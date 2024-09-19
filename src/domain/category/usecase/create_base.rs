use axum::async_trait;

use crate::{
    domain::category::{dto::request::NewBaseCategory, repository::save_base::SaveCategoryRepo},
    global::errors::CustomError,
};

pub struct CreateCategoryUsecaseImpl<T>
where
    T: SaveCategoryRepo,
{
    repository: T,
}

#[async_trait]
pub trait CreateCategoryUsecase: Send + Sync {
    async fn create_base_category(
        &self,
        user_id: i32,
        new_base: NewBaseCategory,
    ) -> Result<i16, Box<CustomError>>;
}

impl<T> CreateCategoryUsecaseImpl<T>
where
    T: SaveCategoryRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> CreateCategoryUsecase for CreateCategoryUsecaseImpl<T>
where
    T: SaveCategoryRepo,
{
    async fn create_base_category(
        &self,
        user_id: i32,
        new_base: NewBaseCategory,
    ) -> Result<i16, Box<CustomError>> {
        _create_base_category(&self.repository, user_id, new_base).await
    }
}

async fn _create_base_category<T>(
    repository: &T,
    user_id: i32,
    new_base: NewBaseCategory,
) -> Result<i16, Box<CustomError>>
where
    T: SaveCategoryRepo,
{
    repository
        .save_base_category(user_id, new_base.to_entity())
        .await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::category::dto::request::NewBaseCategory;
    use crate::domain::category::{entity::BaseCategory, repository::save_base::SaveCategoryRepo};
    use crate::global::errors::CustomError;

    use super::_create_base_category;

    mock! {
        SaveCategoryRepoImpl {}

        #[async_trait]
        impl SaveCategoryRepo for SaveCategoryRepoImpl {
            async fn save_base_category(
                &self,
                user_id: i32,
                base_category: BaseCategory,
            ) -> Result<i16, Box<CustomError>>;

        }
    }

    #[tokio::test]
    async fn check_create_success() {
        // Arrange
        let user_id = 1;
        let base_category = BaseCategory::new(
            1,
            1,
            true,
            false,
            "테스트 베이스".to_string(),
            "112233".to_string(),
        );

        let mut mock_repo = MockSaveCategoryRepoImpl::new();
        mock_repo
            .expect_save_base_category()
            .with(predicate::eq(user_id), predicate::eq(base_category))
            .returning(|_, _| Ok(1));

        let new_base = NewBaseCategory::new(
            1,
            1,
            true,
            false,
            "테스트 베이스".to_string(),
            "112233".to_string(),
        );

        // Act
        let result = _create_base_category(&mock_repo, user_id, new_base).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        assert_eq!(result.unwrap(), 1);
    }
}
