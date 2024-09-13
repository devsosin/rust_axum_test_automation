use std::sync::Arc;

use axum::{
    routing::{delete, get, patch},
    Extension, Router,
};

use sqlx::PgPool;

mod delete;
mod read;
mod update;

use delete::delete_user;
use read::read_user;
use update::update_user;

use super::{
    repository::{
        delete::DeleteUserRepoImpl, get_by_id::GetUserByIdRepoImpl, update::UpdateUserRepoImpl,
    },
    usecase::{
        delete::DeleteUserUsecaseImpl, read::ReadUserUsecaseImpl, update::UpdateUserUsecaseImpl,
    },
};

pub fn read_router(pool: &Arc<PgPool>) -> Router {
    let repository = GetUserByIdRepoImpl::new(&pool);
    let usecase = ReadUserUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/:user_id",
            get(read_user::<ReadUserUsecaseImpl<GetUserByIdRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn update_router(pool: &Arc<PgPool>) -> Router {
    let repository = UpdateUserRepoImpl::new(&pool);
    let usecase = UpdateUserUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/:user_id",
            patch(update_user::<UpdateUserUsecaseImpl<UpdateUserRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn delete_router(pool: &Arc<PgPool>) -> Router {
    let repository = DeleteUserRepoImpl::new(&pool);
    let usecase = DeleteUserUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/:user_id",
            delete(delete_user::<DeleteUserUsecaseImpl<DeleteUserRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
