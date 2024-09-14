use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use create::create_images;
use read::read_image;
use s3::Bucket;
use sqlx::PgPool;

use super::{
    repository::{get_by_id::GetImageByIdRepoImpl, save::SaveImageRepoImpl},
    usecase::{create::CreateImageUsecaseImpl, read::ReadImageUsecaseImpl},
};

mod create;
mod read;

pub fn create_router(pool: &Arc<PgPool>, bucket: &Arc<Bucket>) -> Router {
    // image usecase -> bucket
    let repository = SaveImageRepoImpl::new(&pool);
    let usecase = CreateImageUsecaseImpl::new(repository, bucket);

    Router::new()
        .route(
            "/",
            post(create_images::<CreateImageUsecaseImpl<SaveImageRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn read_router(pool: &Arc<PgPool>) -> Router {
    let repository = GetImageByIdRepoImpl::new(&pool);
    let usecase = ReadImageUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/:image_id",
            get(read_image::<ReadImageUsecaseImpl<GetImageByIdRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
