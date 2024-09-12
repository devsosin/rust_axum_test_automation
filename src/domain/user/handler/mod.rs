use std::sync::Arc;

use axum::{
    routing::{delete, get, patch, post},
    Extension, Router,
};

use sqlx::PgPool;

mod create;
mod delete;
mod login;
mod read;
mod refresh;
mod update;

use create::create_user;
use delete::delete_user;
use login::login;
use read::read_user;
use update::update_user;

use super::{
    repository::{
        delete::DeleteUserRepoImpl, get_user::GetUserRepoImpl, login::LoginUserRepoImpl,
        save::SaveUserRepoImpl, update::UpdateUserRepoImpl,
    },
    usecase::{
        create::CreateUserUsecaseImpl, delete::DeleteUserUsecaseImpl, login::LoginUserUsecaseImpl,
        read::ReadUserUsecaseImpl, refresh::RefreshTokenUsecaseImpl, update::UpdateUserUsecaseImpl,
    },
};

pub fn create_router(pool: &Arc<PgPool>) -> Router {
    let repository = SaveUserRepoImpl::new(&pool);
    let usecase = CreateUserUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/",
            post(create_user::<CreateUserUsecaseImpl<SaveUserRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn read_router(pool: &Arc<PgPool>) -> Router {
    let repository = GetUserRepoImpl::new(&pool);
    let usecase = ReadUserUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/:user_id",
            get(read_user::<ReadUserUsecaseImpl<GetUserRepoImpl>>),
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

pub fn login_router(pool: &Arc<PgPool>) -> Router {
    // login user
    let login_repo = LoginUserRepoImpl::new(pool);
    let save_repo = SaveUserRepoImpl::new(pool);
    let usecase = LoginUserUsecaseImpl::new(login_repo, save_repo);

    Router::new()
        .route(
            "/login",
            post(login::<LoginUserUsecaseImpl<LoginUserRepoImpl, SaveUserRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

// refresh
pub fn refresh_router(pool: &Arc<PgPool>) -> Router {
    let repository = GetUserRepoImpl::new(pool);
    let usecase = RefreshTokenUsecaseImpl::new(repository);

    Router::new().layer(Extension(Arc::new(usecase)))
}
