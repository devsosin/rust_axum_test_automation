use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use super::handler::{create_router, read_router};

pub fn get_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .merge(create_router(pool.clone()))
        .merge(read_router(pool.clone()))
}
