use axum::async_trait;

use crate::{domain::book::repository::delete::DeleteBookRepo, global::errors::CustomError};

pub struct DeleteBookUsecaseImpl<T>
where
    T: DeleteBookRepo,
{
    repository: T,
}

#[async_trait]
pub trait DeleteBookUsecase: Send + Sync {
    async fn delete_book(&self, user_id: i32, book_id: i32) -> Result<(), Box<CustomError>>;
}

impl<T> DeleteBookUsecaseImpl<T>
where
    T: DeleteBookRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> DeleteBookUsecase for DeleteBookUsecaseImpl<T>
where
    T: DeleteBookRepo,
{
    async fn delete_book(&self, user_id: i32, book_id: i32) -> Result<(), Box<CustomError>> {
        delete_book(&self.repository, user_id, book_id).await
    }
}

async fn delete_book<T: DeleteBookRepo>(
    repository: &T,
    user_id: i32,
    book_id: i32,
) -> Result<(), Box<CustomError>> {
    repository.delete_book(user_id, book_id).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::book::{repository::delete::DeleteBookRepo, usecase::delete::delete_book};
    use crate::global::errors::CustomError;

    mock! {
        DeleteBookRepoImpl {}

        #[async_trait]
        impl DeleteBookRepo for DeleteBookRepoImpl {
            async fn delete_book(&self, user_id: i32, book_id: i32) -> Result<(), Box<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_delete_book_success() {
        // Arrange
        let mut mock_repo = MockDeleteBookRepoImpl::new();

        let user_id = 1;
        let target_id = 1;
        mock_repo
            .expect_delete_book()
            .with(predicate::eq(user_id), predicate::eq(target_id))
            .returning(|_, _| Ok(()));

        // Act
        let result = delete_book(&mock_repo, user_id, target_id).await;

        // Assert
        assert!(result.is_ok());
    }
}
