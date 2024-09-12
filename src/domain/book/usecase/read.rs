use std::sync::Arc;

use axum::async_trait;

use crate::{
    domain::book::{entity::Book, repository::get_book::GetBookRepo},
    global::errors::CustomError,
};

pub(crate) struct ReadBookUsecaseImpl<T>
where
    T: GetBookRepo,
{
    repository: T,
}

#[async_trait]
pub(crate) trait ReadBookUsecase: Send + Sync {
    async fn read_books(&self) -> Result<Vec<Book>, Arc<CustomError>>;
    async fn read_book(&self, id: i32) -> Result<Book, Arc<CustomError>>;
}

impl<T> ReadBookUsecaseImpl<T>
where
    T: GetBookRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> ReadBookUsecase for ReadBookUsecaseImpl<T>
where
    T: GetBookRepo,
{
    async fn read_books(&self) -> Result<Vec<Book>, Arc<CustomError>> {
        // Dereferencing Arc to get to the inner T
        read_books(&self.repository).await
    }
    async fn read_book(&self, id: i32) -> Result<Book, Arc<CustomError>> {
        read_book(&self.repository, id).await
    }
}

async fn read_books<T: GetBookRepo>(repository: &T) -> Result<Vec<Book>, Arc<CustomError>> {
    repository.get_books().await
}

async fn read_book<T: GetBookRepo>(repository: &T, id: i32) -> Result<Book, Arc<CustomError>> {
    repository.get_book(id).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::book::{entity::Book, repository::get_book::GetBookRepo};
    use crate::global::errors::CustomError;

    use super::{ReadBookUsecase, ReadBookUsecaseImpl};

    mock! {
        GetBookRepoImpl {}

        #[async_trait]
        impl GetBookRepo for GetBookRepoImpl {
            async fn get_books(&self) -> Result<Vec<Book>, Arc<CustomError>>;
            async fn get_book(&self, id: i32) -> Result<Book, Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_read_books_success() {
        // Arrange
        let mut mock_repo = MockGetBookRepoImpl::new();

        // 모킹 동작 설정
        mock_repo.expect_get_books().returning(|| Ok(vec![])); // 성공 시 id 1반환

        let usecase = ReadBookUsecaseImpl::<MockGetBookRepoImpl>::new(mock_repo);

        // Act
        let books = usecase.read_books().await;
        assert!(books.is_ok());
        let books = books.unwrap();

        // Assert
        assert_eq!(books.len(), 0);
    }

    #[tokio::test]
    async fn check_read_books_failure() {
        // Arrange
        let mut mock_repo = MockGetBookRepoImpl::new();

        // 발생할 수 있는 에러케이스? 데이터베이스 접속 에러
        mock_repo.expect_get_books().returning(|| {
            Err(Arc::new(CustomError::Unexpected(anyhow::Error::msg(
                "에러 발생",
            ))))
        });

        let usecase = ReadBookUsecaseImpl::<MockGetBookRepoImpl>::new(mock_repo);

        // Act
        let result = usecase.read_books().await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn check_read_book_success() {
        // Arrange
        let mut mock_repo = MockGetBookRepoImpl::new();
        let id = 1;

        mock_repo
            .expect_get_book()
            .with(predicate::eq(id))
            .returning(|i| Ok(Book::new(Some(i), "새 가계부".to_string(), 1)));

        let usecase = ReadBookUsecaseImpl::<MockGetBookRepoImpl>::new(mock_repo);

        // Act
        let result = usecase.read_book(id).await;
        assert!(result.is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_id().unwrap(), id);
    }

    #[tokio::test]
    async fn check_read_book_failure() {
        // Arrange
        let mut mock_repo = MockGetBookRepoImpl::new();
        let id = 1;

        mock_repo
            .expect_get_book()
            .with(predicate::eq(id))
            .returning(|i| {
                Err(Arc::new(CustomError::Unexpected(anyhow::Error::msg(
                    "에러 발생",
                ))))
            });

        let usecase = ReadBookUsecaseImpl::<MockGetBookRepoImpl>::new(mock_repo);

        // Act
        let result = usecase.read_book(id).await;

        // Assert
        assert!(result.is_err())
    }
}
