use std::sync::Arc;

use axum::{routing::post, Extension, Router};
use create::create_record;
use sqlx::PgPool;

use super::{repository::save::SaveRecordRepoImpl, usecase::create::CreateRecordUsecaseImpl};

mod create;

pub(crate) fn create_router(pool: Arc<PgPool>) -> Router {
    let repository = SaveRecordRepoImpl::new(pool.clone());
    let usecase = CreateRecordUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route(
            "/",
            post(create_record::<CreateRecordUsecaseImpl<SaveRecordRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
