use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};

use read::read_user;
use sqlx::PgPool;

mod create;
mod read;

use create::create_user;

use super::{
    repository::{get_user::GetUserRepoImpl, save::SaveUserRepoImpl},
    usecase::{create::CreateUserUsecaseImpl, read::ReadUserUsecaseImpl},
};

pub fn create_router(pool: Arc<PgPool>) -> Router {
    let repository = SaveUserRepoImpl::new(pool.clone());
    let usecase = CreateUserUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route(
            "/",
            post(create_user::<CreateUserUsecaseImpl<SaveUserRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn read_router(pool: Arc<PgPool>) -> Router {
    let repository = GetUserRepoImpl::new(pool.clone());
    let usecase = ReadUserUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route(
            "/:user_id",
            get(read_user::<ReadUserUsecaseImpl<GetUserRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
