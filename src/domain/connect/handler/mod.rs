use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use create::create_connect;
use read::read_connect;
use sqlx::PgPool;

use super::{
    repository::{get::GetConnectRepoImpl, save::SaveConnectRepoImpl},
    usecase::{create::CreateConnectUsecaseImpl, read::ReadConnectUsecaseImpl},
};

pub(super) mod create;
pub(super) mod read;

pub fn create_router(pool: &Arc<PgPool>) -> Router {
    let repository = SaveConnectRepoImpl::new(&pool);
    let usecase = CreateConnectUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/",
            post(create_connect::<CreateConnectUsecaseImpl<SaveConnectRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn read_router(pool: &Arc<PgPool>) -> Router {
    let repository = GetConnectRepoImpl::new(&pool);
    let usecase = ReadConnectUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/",
            get(read_connect::<ReadConnectUsecaseImpl<GetConnectRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
