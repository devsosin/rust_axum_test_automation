use axum::async_trait;

use crate::{
    domain::category::{entity::BaseCategory, repository::get_base::GetCategoryRepo},
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
    async fn read_base_category(
        &self,
        user_id: i32,
        book_id: i32,
        is_record: bool,
    ) -> Result<Vec<BaseCategory>, Box<CustomError>>;
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
    async fn read_base_category(
        &self,
        user_id: i32,
        book_id: i32,
        is_record: bool,
    ) -> Result<Vec<BaseCategory>, Box<CustomError>> {
        _read_base_category(&self.repository, user_id, book_id, is_record).await
    }
}

async fn _read_base_category<T>(
    repository: &T,
    user_id: i32,
    book_id: i32,
    is_record: bool,
) -> Result<Vec<BaseCategory>, Box<CustomError>>
where
    T: GetCategoryRepo,
{
    repository.get_list(user_id, book_id, is_record).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::{
        domain::category::{entity::BaseCategory, repository::get_base::GetCategoryRepo},
        global::errors::CustomError,
    };

    use super::_read_base_category;

    mock! {
        GetCategoryRepoImpl {}

        #[async_trait]
        impl GetCategoryRepo for GetCategoryRepoImpl {
            async fn get_list(
                &self,
                user_id: i32,
                book_id: i32,
                is_record: bool,
            ) -> Result<Vec<BaseCategory>, Box<CustomError>>;

        }
    }

    #[tokio::test]
    async fn check_read_base_category_success() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let is_record = true;

        let mut mock_repo = MockGetCategoryRepoImpl::new();
        mock_repo
            .expect_get_list()
            .with(
                predicate::eq(user_id),
                predicate::eq(book_id),
                predicate::eq(is_record),
            )
            .returning(|_, i, r| {
                Ok(vec![BaseCategory::new(
                    1,
                    i,
                    r,
                    false,
                    "테스트 카테고리".to_string(),
                    "112233".to_string(),
                )
                .id(1)])
            });

        // Act
        let result = _read_base_category(&mock_repo, user_id, book_id, is_record).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        assert_eq!(result.unwrap().len(), 1)
    }
}
