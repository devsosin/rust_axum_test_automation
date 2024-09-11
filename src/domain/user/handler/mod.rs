use std::sync::Arc;

use axum::{
    routing::{get, patch, post},
    Extension, Router,
};

use sqlx::PgPool;

mod create;
mod read;
mod update;

use create::create_user;
use read::read_user;
use update::update_user;

use super::{
    repository::{get_user::GetUserRepoImpl, save::SaveUserRepoImpl, update::UpdateUserRepoImpl},
    usecase::{
        create::CreateUserUsecaseImpl, read::ReadUserUsecaseImpl, update::UpdateUserUsecaseImpl,
    },
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

pub fn update_router(pool: Arc<PgPool>) -> Router {
    let repository = UpdateUserRepoImpl::new(pool.clone());
    let usecase = UpdateUserUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route(
            "/:user_id",
            patch(update_user::<UpdateUserUsecaseImpl<UpdateUserRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
