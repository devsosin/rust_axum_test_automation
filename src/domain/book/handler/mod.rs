use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};

use sqlx::PgPool;

pub(super) mod create;
pub(super) mod read;
pub(super) mod read_type;

use create::create_book;
use read::{read_book, read_books};
use read_type::read_book_types;

use super::{
    repository::{get_book::GetBookRepoImpl, save::SaveBookRepoImpl, GetBookTypeRepoImpl},
    usecase::{CreateBookUsecaseImpl, ReadBookTypeUsecaseImpl, ReadBookUsecaseImpl},
};

pub fn create_router(pool: Arc<PgPool>) -> Router {
    let repository = SaveBookRepoImpl::new(pool.clone());

    let usecase = CreateBookUsecaseImpl::new(Arc::new(repository));

    Router::new().route(
        "/",
        post(create_book::<CreateBookUsecaseImpl<SaveBookRepoImpl>>)
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

pub fn read_type_router(pool: Arc<PgPool>) -> Router {
    let repository = GetBookTypeRepoImpl::new(pool);

    let usecase = ReadBookTypeUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route(
            "/",
            get(read_book_types::<ReadBookTypeUsecaseImpl<GetBookTypeRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
