use std::path::Path;

use docker_api::{
    models::ImageBuildChunk,
    opts::{
        ImageBuildOpts, ImageFilter, ImageListOpts, ImagePruneOpts, ImagesPruneFilter, PullOpts,
    },
    Docker,
};
use eyre::{bail, Result};
use tokio_stream::{Stream, StreamExt};
use tracing::instrument;

#[derive(Debug)]
pub struct DockerManager {
    docker: Docker,
}

#[derive(Debug)]
pub struct Image {
    name: String,
    source: ImageSource,
}

#[derive(Debug)]
pub enum ImageSource {
    Local {
        /// Path to the Containerfile.
        path: String,
    },
    Registry,
}

impl Image {
    pub fn new_local(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        Image {
            // TODO: this is ugly
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            source: ImageSource::Local {
                path: path.to_string_lossy().to_string(),
            },
        }
    }

    pub fn new_registry(name: impl Into<String>) -> Self {
        Image {
            name: name.into(),
            source: ImageSource::Registry,
        }
    }
}

impl DockerManager {
    pub async fn new() -> Self {
        // TODO: Add support for windows
        let docker = Docker::unix("/var/run/docker.sock");
        if let Err(_) = docker.ping().await {
            panic!("Could not connect to docker daemon at /var/run/docker.sock")
        }

        DockerManager { docker }
    }

    // Creates or Pulls the image from a dockerfile or from a registryi.
    #[instrument(skip(self), level = "debug")]
    async fn create_image(&self, image_data: Image) -> Result<()> {
        let images = self.docker.images();

        let mut image: Box<
            dyn Stream<Item = Result<ImageBuildChunk, docker_api::Error>> + Unpin + Send,
        > = match image_data.source {
            ImageSource::Local { path } => {
                Box::new(
                    // Docker only requires a directory containing the dockerfile.
                    images.build_par(
                        &ImageBuildOpts::builder(path)
                            // Typo in the library
                            .nocahe(true)
                            .labels([("mastiff.recipe-name", image_data.name)])
                            .build(),
                    ),
                )
            }
            // TODO: Support pulling from custom registries
            ImageSource::Registry => {
                Box::new(images.pull(&PullOpts::builder().image(image_data.name).build()))
            }
        };

        // Only check for any errors, there is pretty much no need for streaming docker logs
        while let Some(data) = image.next().await {
            match data {
                Ok(build_data) => match build_data {
                    ImageBuildChunk::Error { error_detail, .. } => {
                        bail!("Could not build the image: {}", error_detail.message)
                    }
                    _ => continue,
                },
                Err(e) => {
                    bail!("Could not build the image {}", e.to_string())
                }
            }
        }

        Ok(())
    }

    /// Gets the image if present locally else creates it and returns it.
    #[instrument(skip(self), level = "debug")]
    pub async fn get_image(&self, img_details: Image, create: bool) -> Result<docker_api::Image> {
        let images = self.docker.images();

        let image = images.get(&img_details.name);

        // Check if image exists, else create it.
        if image.inspect().await.is_err() {
            tracing::debug!("Could not inspect image: {}", img_details.name);
            if create {
                self.create_image(img_details).await?;
            }
        }
        Ok(image)
    }

    /// Get a list of images which are associated to recipes.
    #[instrument(skip(self), level = "debug", ret(Debug))]
    pub async fn list_images(&self) -> Result<Vec<String>> {
        let images = self.docker.images();

        Ok(images
            .list(
                &ImageListOpts::builder()
                    .filter([ImageFilter::LabelKey("mastiff.recipe-name".to_string())])
                    .build(),
            )
            .await?
            .into_iter()
            .map(|image| image.labels.get("mastiff.recipe-name").unwrap().to_string())
            .collect())
    }

    /// Deletes the image associated with a recipe.
    #[instrument(skip(self), level = "debug")]
    pub async fn delete_image(&self, name: &str) -> Result<()> {
        let images = self.docker.images();
        let pruned = images
            .prune(
                &ImagePruneOpts::builder()
                    .filter([ImagesPruneFilter::Label(
                        "mastiff.recipe-name".to_string(),
                        name.to_string(),
                    )])
                    .build(),
            )
            .await?; // TODO: Should I use prune here?
        tracing::debug!(
            "Deleted {} Images associated with '{name}' recipe",
            pruned.images_deleted.map_or(0, |imgs| imgs.len())
        );
        Ok(())
    }
}
