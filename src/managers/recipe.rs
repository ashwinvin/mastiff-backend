use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_compression::tokio::bufread::GzipDecoder;
use config::{Config, ConfigError, File};
use eyre::Result;
use serde::Deserialize;
use tokio::{fs, io::AsyncBufRead, sync::RwLock};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};
use tokio_tar::Archive;
use tracing::instrument;

use super::docker::{DockerManager, Image, ImageSource};

#[derive(Debug, Deserialize)]
pub enum ImageType {
    /// The public docker image name.
    Registry(String),
    /// Uses the recipe name as the image name.
    Local,
}

#[derive(Debug, Deserialize)]
pub struct Recipe {
    /// Name of the recipe
    pub name: String,
    /// The version ID of the recipe. This is used to check if there are any
    /// containers using unupdated recipes.
    pub version: String,
    /// Whether the image is pulled or built locally
    pub image: ImageType,
    /// The string which indicates that the process has done starting.
    pub process_started_indicator: String,
    /// The string which indicates that the process has done exiting. If left
    /// `None`, the container will be marked stopped when docker registers it as stopped.
    pub process_ended_indicator: Option<String>,
    /// The command to send when a stop request is given. If left `None`, SIGTERM
    /// is send.
    pub process_stop_cmd: Option<String>,
    /// Minimum number of ports required for the container.
    pub min_ports: usize,
    /// The path where the config file must be stored in the container relative to
    /// `/home/container`, the recipe must contain the config file with the correct
    /// name if this is set.
    pub config_path: Option<PathBuf>,
}

impl Recipe {
    fn parse(path: &str) -> Result<Self> {
        Ok(Config::builder()
            .add_source(File::with_name(path))
            .build()?
            .try_deserialize()?)
    }
}

#[derive(Debug, Clone)]
pub struct RecipeManager {
    recipe_directory: PathBuf,
    docker_manager: Arc<DockerManager>,
}

impl RecipeManager {
    pub fn new<P: Into<PathBuf>>(recipe_directory: P, docker_manager: Arc<DockerManager>) -> Self {
        Self {
            docker_manager,
            recipe_directory: recipe_directory.into(),
        }
    }

    /// Parse and build the recipe.
    #[instrument(skip(self), level = "debug")]
    pub async fn build_recipe(&self, recipe_path: &Path) -> Result<()> {
        let recipe_config = Recipe::parse(recipe_path.join("recipe.toml").to_str().unwrap())?;

        match recipe_config.image {
            ImageType::Registry(name) => {
                self.docker_manager
                    .get_image(Image::new_registry(name), true)
                    .await?
            }
            ImageType::Local => {
                self.docker_manager
                    .get_image(Image::new_local(recipe_path), true)
                    .await?
            }
        };

        Ok(())
    }

    /// Decompress a `tar.gz` file from a byte stream to a specified path.
    /// The tar should not contain the root dir.
    #[instrument(skip(file_stream, self), level = "debug")]
    pub async fn decompress_files(
        &self,
        recipe_name: &str,
        file_stream: impl AsyncBufRead + Unpin + Send,
    ) -> Result<PathBuf> {
        let recipe_path = self.recipe_directory.join(recipe_name);
        let decoder = GzipDecoder::new(file_stream);
        let mut unarchiver = Archive::new(decoder);

        let mut entries = unarchiver.entries()?;

        while let Some(file) = entries.next().await {
            let mut file = file?;
            file.unpack_in(&recipe_path).await?;
        }

        Ok(recipe_path)
    }

    /// Lists the recipes that are present in the recipe directory
    /// Note: This **doesn't** parse the recipe.toml in each directory to check
    /// if they are valid recipes.
    #[instrument(skip(self), level = "debug")]
    pub async fn list_recipes(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        tracing::debug!(
            "Searching for recipes in {0}",
            self.recipe_directory.display()
        );

        let mut file_list = ReadDirStream::new(fs::read_dir(&self.recipe_directory).await?);

        while let Some(Ok(file)) = file_list.next().await {
            if file.file_type().await.unwrap().is_dir() {
                files.push(file.file_name().into_string().unwrap());
            }
        }

        Ok(files)
    }

    /// Deletes a recipe if its present in the store.
    #[instrument(skip(self), level = "debug")]
    pub async fn delete_recipe(&self, recipe_name: &str) -> Result<()> {
        let recipes = self.list_recipes().await?;
        tracing::debug!("Found recipes: {recipes:?}");

        if recipes.iter().any(|el| el == recipe_name) {
            fs::remove_dir_all(self.recipe_directory.join(recipe_name)).await?;
        }
        // Try cleaning up any danglin images
        self.docker_manager.delete_image(recipe_name).await?;
        Ok(())
    }
}
