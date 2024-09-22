use std::sync::Arc;

use axum::{
    routing::{delete, get, patch, post},
    Extension, Router,
};
use delete_base::delete_base_category;
use delete_sub::delete_sub_category;
use read_base::read_base_category;
use read_sub::read_sub_category;
use sqlx::PgPool;

use create_base::create_base_category;
use create_sub::create_sub_category;
use update_base::update_base_category;
use update_sub::update_sub_category;

use super::{
    repository::{
        delete_base::DeleteCategoryRepoImpl as DeleteBaseCategoryRepoImpl,
        delete_sub::DeleteCategoryRepoImpl as DeleteSubCategoryRepoImpl,
        get_base::GetCategoryRepoImpl as GetBaseCategoryRepoImpl,
        get_sub::GetCategoryRepoImpl as GetSubCategoryRepoImpl,
        save_base::SaveCategoryRepoImpl as SaveBaseCategoryRepoImpl,
        save_sub::SaveCategoryRepoImpl as SaveSubCategoryRepoImpl,
        update_base::UpdateCategoryRepoImpl as UpdateBaseCategoryRepoImpl,
        update_sub::UpdateCategoryRepoImpl as UpdateSubCategoryRepoImpl,
    },
    usecase::{
        create_base::CreateCategoryUsecaseImpl as CreateBaseCateogryUsecaseImpl,
        create_sub::CreateCategoryUsecaseImpl as CreateSubCateogryUsecaseImpl,
        delete_base::DeleteCategoryUsecaseImpl as DeleteBaseCategoryUsecaseImpl,
        delete_sub::DeleteCategoryUsecaseImpl as DeleteSubCategoryUsecaseImpl,
        read_base::ReadCategoryUsecaseImpl as ReadBaseCategoryUsecaseImpl,
        read_sub::ReadCategoryUsecaseImpl as ReadSubCategoryUsecaseImpl,
        update_base::UpdateCategoryUsecaseImpl as UpdateBaseCategoryUsecaseImpl,
        update_sub::UpdateCategoryUsecaseImpl as UpdateSubCategoryUsecaseImpl,
    },
};

mod create_base;
mod create_sub;
mod delete_base;
mod delete_sub;
mod read_base;
mod read_sub;
mod update_base;
mod update_sub;

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

pub fn update_base_router(pool: &Arc<PgPool>) -> Router {
    let repository = UpdateBaseCategoryRepoImpl::new(&pool);
    let usecase = UpdateBaseCategoryUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/base/:base_id",
            patch(
                update_base_category::<UpdateBaseCategoryUsecaseImpl<UpdateBaseCategoryRepoImpl>>,
            ),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn update_sub_router(pool: &Arc<PgPool>) -> Router {
    let repository = UpdateSubCategoryRepoImpl::new(&pool);
    let usecase = UpdateSubCategoryUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/sub/:sub_id",
            patch(update_sub_category::<UpdateSubCategoryUsecaseImpl<UpdateSubCategoryRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn delete_base_router(pool: &Arc<PgPool>) -> Router {
    let repository = DeleteBaseCategoryRepoImpl::new(&pool);
    let usecase = DeleteBaseCategoryUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/base/:base_id",
            delete(
                delete_base_category::<DeleteBaseCategoryUsecaseImpl<DeleteBaseCategoryRepoImpl>>,
            ),
        )
        .layer(Extension(Arc::new(usecase)))
}

pub fn delete_sub_router(pool: &Arc<PgPool>) -> Router {
    let repository = DeleteSubCategoryRepoImpl::new(&pool);
    let usecase = DeleteSubCategoryUsecaseImpl::new(repository);

    Router::new()
        .route(
            "/sub/:sub_id",
            delete(delete_sub_category::<DeleteSubCategoryUsecaseImpl<DeleteSubCategoryRepoImpl>>),
        )
        .layer(Extension(Arc::new(usecase)))
}
