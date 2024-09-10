use std::sync::Arc;

use axum::{routing::post, Extension, Router};

use sqlx::PgPool;

mod create;

use create::create_user;

use super::{repository::save::SaveUserRepoImpl, usecase::create::CreateUserUsecaseImpl};

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
