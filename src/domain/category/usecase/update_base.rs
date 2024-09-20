use axum::async_trait;

use crate::{
    domain::category::{
        dto::request::EditBaseCategory, repository::update_base::UpdateCategoryRepo,
    },
    global::errors::CustomError,
};

pub struct UpdateCategoryUsecaseImpl<T>
where
    T: UpdateCategoryRepo,
{
    repository: T,
}

#[async_trait]
pub trait UpdateCategoryUsecase: Send + Sync {
    async fn update_base_category(
        &self,
        user_id: i32,
        base_id: i16,
        edit_base: EditBaseCategory,
    ) -> Result<(), Box<CustomError>>;
}

impl<T> UpdateCategoryUsecaseImpl<T>
where
    T: UpdateCategoryRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> UpdateCategoryUsecase for UpdateCategoryUsecaseImpl<T>
where
    T: UpdateCategoryRepo,
{
    async fn update_base_category(
        &self,
        user_id: i32,
        base_id: i16,
        edit_base: EditBaseCategory,
    ) -> Result<(), Box<CustomError>> {
        _update_base_category(&self.repository, user_id, base_id, edit_base).await
    }
}

async fn _update_base_category<T>(
    repository: &T,
    user_id: i32,
    base_id: i16,
    edit_base: EditBaseCategory,
) -> Result<(), Box<CustomError>>
where
    T: UpdateCategoryRepo,
{
    repository
        .update_base_category(user_id, base_id, edit_base.to_update())
        .await
}
