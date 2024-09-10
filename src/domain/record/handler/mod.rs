use std::sync::Arc;

use axum::{
    routing::{delete, get, patch, post},
    Extension, Router,
};

use sqlx::PgPool;

mod create;
mod delete;
mod read;
mod update;

use create::create_record;
use delete::delete_record;
use read::{read_record, read_records};
use update::update_record;

use super::{
    repository::{
        delete::DeleteRecordRepoImpl, get_record::GetRecordRepoImpl, save::SaveRecordRepoImpl,
        update::UpdateRecordRepoImpl,
    },
    usecase::{
        create::CreateRecordUsecaseImpl, delete::DeleteRecordUsecaseImpl,
        read::ReadRecordUsecaseImpl, update::UpdateRecordUsecaseImpl,
    },
};

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

pub(crate) fn delete_router(pool: Arc<PgPool>) -> Router {
    let repository = DeleteRecordRepoImpl::new(pool.clone());
    let usecase = DeleteRecordUsecaseImpl::new(Arc::new(repository));

    Router::new()
        .route(
            "/:record_id",
            delete(delete_record::<DeleteRecordUsecaseImpl<DeleteRecordRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
