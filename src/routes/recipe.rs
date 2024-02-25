use std::{path::PathBuf, sync::Arc};

use axum::{
    debug_handler,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use eyre::{bail, eyre, Result};
use tokio::sync::OnceCell;
use tokio_stream::StreamExt;
use tokio_util::io::StreamReader;
use tracing::instrument;

use super::AppError;
use crate::managers::recipe::RecipeManager;

#[debug_handler]
pub async fn upload_recipe(
    State(recipe_manager): State<Arc<RecipeManager>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut recipe_name = None;
    let mut file_reader = OnceCell::new();

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        // The file name check is a must.
        if field.name().is_some() && field.file_name().is_none() {
            let field_name = field.name().unwrap();
            if field_name == "name" {
                recipe_name = Some(field.text().await.unwrap());
                continue;
            }
        } else if field.file_name().is_some() {
            let file_stream = field.chunk().await?.unwrap();

            file_reader.set(file_stream)?;
        }
    }

    if let (Some(recipe_name), Some(file_reader)) = (recipe_name, file_reader.get_mut()) {
        tracing::debug!("Registering recipe: {recipe_name}");
        let recipe_path = recipe_manager
            .decompress_files(&recipe_name, file_reader as &[u8])
            .await?;

        recipe_manager.build_recipe(&recipe_path).await?;

        return Ok(StatusCode::CREATED);
    }
    return Err(eyre!("Missing compressed recipe file.").into());
}

#[instrument(skip(recipe_manager), level = "debug")]
pub async fn delete_recipe(
    Path(recipe_name): Path<String>,
    State(recipe_manager): State<Arc<RecipeManager>>,
) -> Result<StatusCode, AppError> {
    recipe_manager.delete_recipe(&recipe_name).await?;
    Ok(StatusCode::NO_CONTENT)
}
