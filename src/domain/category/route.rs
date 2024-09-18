use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

pub fn get_router(pool: &Arc<PgPool>) -> Router {
    Router::new()
}
