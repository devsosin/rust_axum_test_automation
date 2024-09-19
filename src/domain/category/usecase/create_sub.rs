use axum::async_trait;

use crate::{
    domain::category::{dto::request::NewSubCategory, repository::save_sub::SaveCategoryRepo},
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
    async fn create_sub_category(
        &self,
        user_id: i32,
        new_sub: NewSubCategory,
    ) -> Result<i32, Box<CustomError>>;
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
    async fn create_sub_category(
        &self,
        user_id: i32,
        new_sub: NewSubCategory,
    ) -> Result<i32, Box<CustomError>> {
        _create_sub_category(&self.repository, user_id, new_sub).await
    }
}

async fn _create_sub_category<T>(
    repository: &T,
    user_id: i32,
    new_sub: NewSubCategory,
) -> Result<i32, Box<CustomError>>
where
    T: SaveCategoryRepo,
{
    repository
        .save_sub_category(user_id, new_sub.to_entity())
        .await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::category::dto::request::NewSubCategory;
    use crate::domain::category::{entity::SubCategory, repository::save_sub::SaveCategoryRepo};
    use crate::global::errors::CustomError;

    use super::_create_sub_category;

    mock! {
        SaveCategoryRepoImpl {}

        #[async_trait]
        impl SaveCategoryRepo for SaveCategoryRepoImpl {
            async fn save_sub_category(
                &self,
                user_id: i32,
                sub_category: SubCategory,
            ) -> Result<i32, Box<CustomError>>;

        }
    }

    #[tokio::test]
    async fn check_create_success() {
        // Arrange
        let user_id = 1;
        let sub_category = SubCategory::new(1, "테스트 서브".to_string());

        let mut mock_repo = MockSaveCategoryRepoImpl::new();
        mock_repo
            .expect_save_sub_category()
            .with(predicate::eq(user_id), predicate::eq(sub_category))
            .returning(|_, _| Ok(1));

        let new_sub = NewSubCategory::new(1, "테스트 서브".to_string());

        // Act
        let result = _create_sub_category(&mock_repo, user_id, new_sub).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        assert_eq!(result.unwrap(), 1);
    }
}
