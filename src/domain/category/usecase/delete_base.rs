use axum::async_trait;

use crate::{
    domain::category::repository::delete_base::DeleteCategoryRepo, global::errors::CustomError,
};

pub struct DeleteCategoryUsecaseImpl<T>
where
    T: DeleteCategoryRepo,
{
    repository: T,
}

#[async_trait]
pub trait DeleteCategoryUsecase {
    async fn delete_base_category(
        &self,
        user_id: i32,
        base_id: i16,
    ) -> Result<(), Box<CustomError>>;
}

impl<T> DeleteCategoryUsecaseImpl<T>
where
    T: DeleteCategoryRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> DeleteCategoryUsecase for DeleteCategoryUsecaseImpl<T>
where
    T: DeleteCategoryRepo,
{
    async fn delete_base_category(
        &self,
        user_id: i32,
        base_id: i16,
    ) -> Result<(), Box<CustomError>> {
        _delete_base_category(&self.repository, user_id, base_id).await
    }
}

async fn _delete_base_category<T>(
    repository: &T,
    user_id: i32,
    base_id: i16,
) -> Result<(), Box<CustomError>>
where
    T: DeleteCategoryRepo,
{
    repository.delete_base_category(user_id, base_id).await
}
