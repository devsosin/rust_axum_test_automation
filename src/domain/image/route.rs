use std::sync::Arc;

use axum::Router;
use s3::Bucket;
use sqlx::PgPool;

use super::handler::{create_router, read_router};

pub fn get_router(pool: &Arc<PgPool>, bucket: &Arc<Bucket>) -> Router {
    Router::new()
        .merge(create_router(&pool, &bucket))
        .merge(read_router(&pool))
}
