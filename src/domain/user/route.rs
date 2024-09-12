use std::sync::Arc;

use axum::{Extension, Router};
use sqlx::PgPool;

use crate::config::jwt::AuthConfig;

use super::handler::{
    create_router, delete_router, login_router, read_router, refresh_router, update_router,
};

pub fn get_public_router(pool: &Arc<PgPool>, auth_config: &Arc<AuthConfig>) -> Router {
    Router::new()
        .merge(create_router(&pool))
        .merge(login_router(&pool))
        .merge(refresh_router(&pool))
        .layer(Extension(auth_config.clone()))
}

pub fn get_private_router(pool: &Arc<PgPool>) -> Router {
    Router::new()
        .merge(read_router(&pool))
        .merge(update_router(&pool))
        .merge(delete_router(&pool))
}
