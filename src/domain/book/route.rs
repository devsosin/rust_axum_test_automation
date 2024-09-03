use axum::{routing::post, Router};

use super::handler::*;

pub fn get_router() -> Router {
    Router::new()
        .route("/", post(create_book))
}