use std::sync::Arc;

use axum::{routing::post, Extension, Router};
use sqlx::PgPool;

use super::{
    handler::create::create_book, repository::BookRepositoryImpl, usecase::BookUsecaseImpl,
};

pub fn get_router(pool: Arc<PgPool>) -> Router {
    let repository = Arc::new(BookRepositoryImpl::new(pool));
    let usecase = Arc::new(BookUsecaseImpl::new(repository));

    Router::new()
        .route("/", post(create_book))
        .layer(Extension(usecase))
}
