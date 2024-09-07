use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use create::create_book;
use read::{read_book, read_books};
use sqlx::PgPool;

use super::{
    repository::{get_book::GetBookRepoImpl, save::SaveBookRepoImpl, GetBookTypeRepoImpl},
    usecase::{CreateBookUsecaseImpl, ReadBookUsecaseImpl},
};

pub(super) mod create;
pub(super) mod read;

pub fn create_router(pool: Arc<PgPool>) -> Router {
    let book_repo = SaveBookRepoImpl::new(pool.clone());
    let type_repo = GetBookTypeRepoImpl::new(pool.clone());

    let usecase = CreateBookUsecaseImpl::new(Arc::new(book_repo), Arc::new(type_repo));

    Router::new().route(
        "/",
        post(create_book::<CreateBookUsecaseImpl<SaveBookRepoImpl, GetBookTypeRepoImpl>>)
            .layer(Extension(Arc::new(usecase))),
    )
}

pub fn read_router(pool: Arc<PgPool>) -> Router {
    let repository = GetBookRepoImpl::new(pool);

    let usecase = ReadBookUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route("/", get(read_books::<ReadBookUsecaseImpl<GetBookRepoImpl>>))
        .route(
            "/:book_id",
            get(read_book::<ReadBookUsecaseImpl<GetBookRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
