use std::sync::Arc;

use axum::{
    routing::{delete, get, patch, post},
    Extension, Router,
};

use delete::delete_user;
use sqlx::PgPool;

mod create;
mod delete;
mod read;
mod update;

use create::create_user;
use read::read_user;
use update::update_user;

use super::{
    repository::{
        delete::DeleteUserRepoImpl, get_user::GetUserRepoImpl, save::SaveUserRepoImpl,
        update::UpdateUserRepoImpl,
    },
    usecase::{
        create::CreateUserUsecaseImpl, delete::DeleteUserUsecaseImpl, read::ReadUserUsecaseImpl,
        update::UpdateUserUsecaseImpl,
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

pub fn delete_router(pool: Arc<PgPool>) -> Router {
    let repository = DeleteUserRepoImpl::new(pool.clone());
    let usecase = DeleteUserUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route(
            "/:user_id",
            delete(delete_user::<DeleteUserUsecaseImpl<DeleteUserRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
