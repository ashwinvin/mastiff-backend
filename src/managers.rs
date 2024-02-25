use std::sync::Arc;

use axum::extract::FromRef;

use crate::config::Settings;

pub mod backup;
pub mod docker;
pub mod ftp;
pub mod recipe;

// TODO: Implement `ManagerFactory` which ingests the config and builds all the required managers
#[derive(Clone, Debug)]
pub struct Managers {
    recipe_manager: Arc<recipe::RecipeManager>,
    docker_manager: Arc<docker::DockerManager>,
}

impl Managers {
    pub async fn new(settings: &Settings) -> Self {
        let docker_manager = Arc::new(docker::DockerManager::new().await);

        let recipe_manager = Arc::new(recipe::RecipeManager::new(
            &settings.recipe_directory,
            Arc::clone(&docker_manager),
        ));

        Self {
            recipe_manager,
            docker_manager,
        }
    }
}

impl FromRef<Managers> for Arc<recipe::RecipeManager> {
    fn from_ref(managers: &Managers) -> Arc<recipe::RecipeManager> {
        Arc::clone(&managers.recipe_manager)
    }
}

impl FromRef<Managers> for Arc<docker::DockerManager> {
    fn from_ref(managers: &Managers) -> Arc<docker::DockerManager> {
        Arc::clone(&managers.docker_manager)
    }
}
