use axum::{routing::post, Router};

use super::handler::create::create_book;

pub fn get_router() -> Router {
    Router::new()
        .route("/", post(create_book))
}