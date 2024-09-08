use std::sync::Arc;

use axum::async_trait;

use crate::domain::book::{entity::BookType, repository::GetBookTypeRepo};

pub struct ReadBookTypeUsecaseImpl<T>
where
    T: GetBookTypeRepo,
{
    repository: Arc<T>,
}

#[async_trait]
pub trait ReadBookTypeUsecase: Send + Sync {
    async fn read_book_types(&self) -> Result<Vec<BookType>, String>;
}

impl<T> ReadBookTypeUsecaseImpl<T>
where
    T: GetBookTypeRepo,
{
    pub fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> ReadBookTypeUsecase for ReadBookTypeUsecaseImpl<T>
where
    T: GetBookTypeRepo,
{
    async fn read_book_types(&self) -> Result<Vec<BookType>, String> {
        read_book_types(&*self.repository).await
    }
}

async fn read_book_types<T: GetBookTypeRepo>(repository: &T) -> Result<Vec<BookType>, String> {
    repository.get_book_types().await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::mock;

    use crate::domain::book::{
        entity::BookType, repository::GetBookTypeRepo, usecase::read_type::read_book_types,
    };

    mock! {
        GetBookTypeRepoImpl {}

        #[async_trait]
        impl GetBookTypeRepo for GetBookTypeRepoImpl {
            async fn get_book_types(&self) -> Result<Vec<BookType>, String>;
            async fn get_book_type_by_name(&self, name: &str) -> Result<BookType, String>;
        }
    }

    #[tokio::test]
    async fn check_get_book_types_success() {
        // Arrange
        let mut mock_repo = MockGetBookTypeRepoImpl::new();

        mock_repo.expect_get_book_types().returning(|| {
            Ok(vec![
                BookType::new(1, "개인".to_string()),
                BookType::new(2, "커플".to_string()),
                BookType::new(3, "기업".to_string()),
            ])
        });

        // Act
        let result = read_book_types(&mock_repo).await;
        assert!(result.is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.len(), 3)
    }

    #[tokio::test]
    async fn check_get_book_types_failure() {
        // Arrange

        let mut mock_repo = MockGetBookTypeRepoImpl::new();

        mock_repo
            .expect_get_book_types()
            .returning(|| Err("데이터베이스 연결 실패".to_string()));

        // Act
        let result = read_book_types(&mock_repo).await;

        // Assert
        assert!(result.is_err())
    }
}
