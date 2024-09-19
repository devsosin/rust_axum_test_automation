use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use read_base::read_base_category;
use read_sub::read_sub_category;
use sqlx::PgPool;

use create_base::create_base_category;
use create_sub::create_sub_category;

use super::{
    repository::{
        get_base::GetCategoryRepoImpl as GetBaseCategoryRepoImpl,
        get_sub::GetCategoryRepoImpl as GetSubCategoryRepoImpl,
        save_base::SaveCategoryRepoImpl as SaveBaseCategoryRepoImpl,
        save_sub::SaveCategoryRepoImpl as SaveSubCategoryRepoImpl,
    },
    usecase::{
        create_base::CreateCategoryUsecaseImpl as CreateBaseCateogryUsecaseImpl,
        create_sub::CreateCategoryUsecaseImpl as CreateSubCateogryUsecaseImpl,
        read_base::ReadCategoryUsecaseImpl as ReadBaseCategoryUsecaseImpl,
        read_sub::ReadCategoryUsecaseImpl as ReadSubCategoryUsecaseImpl,
    },
};

mod create_base;
mod create_sub;
mod read_base;
mod read_sub;

pub fn create_base_router(pool: &Arc<PgPool>) -> Router {
    let repository = SaveBaseCategoryRepoImpl::new(&pool);
    let usecase = CreateBaseCateogryUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/base",
            post(create_base_category::<CreateBaseCateogryUsecaseImpl<SaveBaseCategoryRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn create_sub_router(pool: &Arc<PgPool>) -> Router {
    let repository = SaveSubCategoryRepoImpl::new(&pool);
    let usecase = CreateSubCateogryUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/sub",
            post(create_sub_category::<CreateSubCateogryUsecaseImpl<SaveSubCategoryRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn read_base_router(pool: &Arc<PgPool>) -> Router {
    let repository = GetBaseCategoryRepoImpl::new(&pool);
    let usecase = ReadBaseCategoryUsecaseImpl::new(repository);

    Router::new().route(
        "/base/:book_id",
        get(read_base_category::<ReadBaseCategoryUsecaseImpl<GetBaseCategoryRepoImpl>>)
            .layer(Extension(Arc::new(usecase))),
    )
}

pub fn read_sub_router(pool: &Arc<PgPool>) -> Router {
    let repository = GetSubCategoryRepoImpl::new(&pool);
    let usecase = ReadSubCategoryUsecaseImpl::new(repository);

    Router::new().route(
        "/sub/:base_id",
        get(read_sub_category::<ReadSubCategoryUsecaseImpl<GetSubCategoryRepoImpl>>)
            .layer(Extension(Arc::new(usecase))),
    )
}
