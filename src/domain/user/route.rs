use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use super::handler::{delete_router, read_router, update_router};

pub fn get_router(pool: &Arc<PgPool>) -> Router {
    Router::new()
        .merge(read_router(&pool))
        .merge(update_router(&pool))
        .merge(delete_router(&pool))
}
