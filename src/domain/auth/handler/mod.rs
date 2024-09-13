use std::sync::Arc;

use axum::{routing::post, Extension, Router};
use sqlx::PgPool;

mod login;
mod refresh;
mod signup;

use login::login;
use refresh::refresh_token;
use signup::signup;

use crate::domain::user::repository::{
    get_by_id::GetUserByIdRepoImpl, get_by_username::GetUserByUsernameRepoImpl,
    save::SaveUserRepoImpl,
};

use super::usecase::{
    login::LoginUserUsecaseImpl, refresh::RefreshTokenUsecaseImpl, signup::SignupUserUsecaseImpl,
};

pub fn signup_router(pool: &Arc<PgPool>) -> Router {
    let repository = SaveUserRepoImpl::new(&pool);
    let usecase = SignupUserUsecaseImpl::new(repository);

    Router::new()
        .route("/", post(signup::<SignupUserUsecaseImpl<SaveUserRepoImpl>>))
        .layer(Extension(Arc::new(usecase)))
}

pub fn login_router(pool: &Arc<PgPool>) -> Router {
    // login user
    let get_repo = GetUserByUsernameRepoImpl::new(pool);
    let save_repo = SaveUserRepoImpl::new(pool);
    let usecase = LoginUserUsecaseImpl::new(get_repo, save_repo);

    Router::new()
        .route(
            "/login",
            post(login::<LoginUserUsecaseImpl<GetUserByUsernameRepoImpl, SaveUserRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

// refresh
pub fn refresh_router(pool: &Arc<PgPool>) -> Router {
    let repository = GetUserByIdRepoImpl::new(pool);
    let usecase = RefreshTokenUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/refresh",
            post(refresh_token::<RefreshTokenUsecaseImpl<GetUserByIdRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
