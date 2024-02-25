use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, post},
    Router,
};

use crate::managers::{recipe::RecipeManager, Managers};

pub mod recipe;

// TODO: Implement concrete error types.
pub struct AppError(eyre::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<eyre::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub fn initialise_routes(managers: Managers) -> Router {
    let recipe_routes = Router::new()
        .route("/recipes/upload", post(recipe::upload_recipe))
        .route("/recipes/:name", delete(recipe::delete_recipe));

    Router::new().merge(recipe_routes).with_state(managers)
}
