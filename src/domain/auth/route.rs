use std::sync::Arc;

use axum::{Extension, Router};
use sqlx::PgPool;

use crate::config::jwt::AuthConfig;

use super::handler::{login_router, refresh_router, signup_router};

pub fn get_router(pool: &Arc<PgPool>, auth_config: &Arc<AuthConfig>) -> Router {
    Router::new()
        .merge(login_router(&pool))
        .merge(refresh_router(&pool))
        .merge(signup_router(&pool))
        .layer(Extension(auth_config.clone()))
}
