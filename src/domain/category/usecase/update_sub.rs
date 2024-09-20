use axum::async_trait;

use crate::{
    domain::category::repository::update_sub::UpdateCategoryRepo, global::errors::CustomError,
};

pub struct UpdateCategoryUsecaseImpl<T>
where
    T: UpdateCategoryRepo,
{
    repository: T,
}

#[async_trait]
pub trait UpdateCategoryUsecase: Send + Sync {
    async fn update_sub_category(
        &self,
        user_id: i32,
        sub_id: i32,
        name: String,
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
    async fn update_sub_category(
        &self,
        user_id: i32,
        sub_id: i32,
        name: String,
    ) -> Result<(), Box<CustomError>> {
        _update_sub_category(&self.repository, user_id, sub_id, name).await
    }
}

async fn _update_sub_category<T>(
    repository: &T,
    user_id: i32,
    sub_id: i32,
    name: String,
) -> Result<(), Box<CustomError>>
where
    T: UpdateCategoryRepo,
{
    repository.update_sub_category(user_id, sub_id, name).await
}
