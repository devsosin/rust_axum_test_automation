use std::sync::Arc;

use axum::{
    routing::{get, patch, post},
    Extension, Router,
};
use read::{read_record, read_records};
use sqlx::PgPool;
use update::update_record;

mod create;
mod read;
mod update;

use super::{
    repository::{
        get_record::GetRecordRepoImpl, save::SaveRecordRepoImpl, update::UpdateRecordRepoImpl,
    },
    usecase::{
        create::CreateRecordUsecaseImpl, read::ReadRecordUsecaseImpl,
        update::UpdateRecordUsecaseImpl,
    },
};
use create::create_record;

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

pub(crate) fn read_router(pool: Arc<PgPool>) -> Router {
    let repository = GetRecordRepoImpl::new(pool.clone());
    let usecase = ReadRecordUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route(
            "/",
            get(read_records::<ReadRecordUsecaseImpl<GetRecordRepoImpl>>),
        )
        .route(
            "/:record_id",
            get(read_record::<ReadRecordUsecaseImpl<GetRecordRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub(crate) fn update_router(pool: Arc<PgPool>) -> Router {
    let repository = UpdateRecordRepoImpl::new(pool.clone());
    let usecase = UpdateRecordUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route(
            "/:record_id",
            patch(update_record::<UpdateRecordUsecaseImpl<UpdateRecordRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
