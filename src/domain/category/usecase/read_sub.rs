use axum::async_trait;

use crate::{
    domain::category::{entity::SubCategory, repository::get_sub::GetCategoryRepo},
    global::errors::CustomError,
};

pub struct ReadCategoryUsecaseImpl<T>
where
    T: GetCategoryRepo,
{
    repository: T,
}

#[async_trait]
pub trait ReadCategoryUsecase: Send + Sync {
    async fn read_sub_category(
        &self,
        user_id: i32,
        base_id: i16,
    ) -> Result<Vec<SubCategory>, Box<CustomError>>;
}

impl<T> ReadCategoryUsecaseImpl<T>
where
    T: GetCategoryRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> ReadCategoryUsecase for ReadCategoryUsecaseImpl<T>
where
    T: GetCategoryRepo,
{
    async fn read_sub_category(
        &self,
        user_id: i32,
        base_id: i16,
    ) -> Result<Vec<SubCategory>, Box<CustomError>> {
        _read_sub_category(&self.repository, user_id, base_id).await
    }
}

async fn _read_sub_category<T>(
    repository: &T,
    user_id: i32,
    base_id: i16,
) -> Result<Vec<SubCategory>, Box<CustomError>>
where
    T: GetCategoryRepo,
{
    repository.get_list(user_id, base_id).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::{
        domain::category::{entity::SubCategory, repository::get_sub::GetCategoryRepo},
        global::errors::CustomError,
    };

    use super::_read_sub_category;

    mock! {
        GetCategoryRepoImpl {}

        #[async_trait]
        impl GetCategoryRepo for GetCategoryRepoImpl {
            async fn get_list(
                &self,
                user_id: i32,
                base_id: i16,
            ) -> Result<Vec<SubCategory>, Box<CustomError>>;

        }
    }

    #[tokio::test]
    async fn check_read_sub_category_success() {
        // Arrange
        let user_id = 1;
        let base_id = 1;

        let mut mock_repo = MockGetCategoryRepoImpl::new();
        mock_repo
            .expect_get_list()
            .with(predicate::eq(user_id), predicate::eq(base_id))
            .returning(|_, i| {
                Ok(vec![
                    SubCategory::new(i, "테스트 카테고리".to_string()).id(1)
                ])
            });

        // Act
        let result = _read_sub_category(&mock_repo, user_id, base_id).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        assert_eq!(result.unwrap().len(), 1)
    }
}
