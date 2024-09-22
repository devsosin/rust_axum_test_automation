use axum::async_trait;

use crate::{
    domain::category::repository::delete_sub::DeleteCategoryRepo, global::errors::CustomError,
};

pub struct DeleteCategoryUsecaseImpl<T>
where
    T: DeleteCategoryRepo,
{
    repository: T,
}

#[async_trait]
pub trait DeleteCategoryUsecase {
    async fn delete_sub_category(&self, user_id: i32, sub_id: i32) -> Result<(), Box<CustomError>>;
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
    async fn delete_sub_category(&self, user_id: i32, sub_id: i32) -> Result<(), Box<CustomError>> {
        _delete_sub_category(&self.repository, user_id, sub_id).await
    }
}

async fn _delete_sub_category<T>(
    repository: &T,
    user_id: i32,
    sub_id: i32,
) -> Result<(), Box<CustomError>>
where
    T: DeleteCategoryRepo,
{
    repository.delete_sub_category(user_id, sub_id).await
}
