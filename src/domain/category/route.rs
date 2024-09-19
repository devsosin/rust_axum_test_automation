use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use super::handler::{create_base_router, create_sub_router, read_base_router, read_sub_router};

pub fn get_router(pool: &Arc<PgPool>) -> Router {
    Router::new()
        .merge(create_base_router(&pool))
        .merge(create_sub_router(&pool))
        .merge(read_base_router(&pool))
        .merge(read_sub_router(&pool))
}
