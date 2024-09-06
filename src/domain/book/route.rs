use std::sync::Arc;

use axum::{routing::post, Extension, Router};
use sqlx::PgPool;

use super::{
    handler::create::create_book,
    repository::{BookRepositoryImpl, BookTypeRepositoryImpl},
    usecase::BookUsecaseImpl,
};

pub fn get_router(pool: Arc<PgPool>) -> Router {
    let book_repo = Arc::new(BookRepositoryImpl::new(pool.clone()));
    let type_repo = Arc::new(BookTypeRepositoryImpl::new(pool));

    let usecase = Arc::new(BookUsecaseImpl::new(book_repo, type_repo));

    Router::new()
        .route(
            "/",
            post(create_book::<BookUsecaseImpl<BookRepositoryImpl, BookTypeRepositoryImpl>>),
        )
        .layer(Extension(usecase))
}
