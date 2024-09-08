use std::sync::Arc;

use axum::Router;

use sqlx::PgPool;

use super::handler::{create_router, delete_router, read_router, read_type_router, update_router};

pub fn get_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .merge(create_router(pool.clone()))
        .merge(read_router(pool.clone()))
        .nest("/type", read_type_router(pool.clone()))
        .merge(update_router(pool.clone()))
        .merge(delete_router(pool.clone()))
}
