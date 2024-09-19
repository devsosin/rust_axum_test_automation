use std::sync::Arc;

use axum::{routing::post, Extension, Router};
use create_base::create_base_category;
use create_sub::create_sub_category;
use sqlx::PgPool;

use super::{
    repository::{
        save_base::SaveCategoryRepoImpl as SaveBaseCategoryRepoImpl,
        save_sub::SaveCategoryRepoImpl as SaveSubCategoryRepoImpl,
    },
    usecase::{
        create_base::CreateCategoryUsecaseImpl as CreateBaseCateogryUsecaseImpl,
        create_sub::CreateCategoryUsecaseImpl as CreateSubCateogryUsecaseImpl,
    },
};

mod create_base;
mod create_sub;

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
