use std::sync::Arc;

use axum::async_trait;

use crate::domain::book::repository::delete::DeleteBookRepo;

pub struct DeleteBookUsecaseImpl<T>
where
    T: DeleteBookRepo,
{
    repository: Arc<T>,
}

#[async_trait]
pub trait DeleteBookUsecase: Send + Sync {
    async fn delete_book(&self, id: i32) -> Result<(), String>;
}

impl<T> DeleteBookUsecaseImpl<T>
where
    T: DeleteBookRepo,
{
    pub fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> DeleteBookUsecase for DeleteBookUsecaseImpl<T>
where
    T: DeleteBookRepo,
{
    async fn delete_book(&self, id: i32) -> Result<(), String> {
        todo!()
    }
}

async fn delete_book<T: DeleteBookRepo>(repository: &T, id: i32) -> Result<(), String> {
    repository.delete_book(id).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::domain::book::{repository::delete::DeleteBookRepo, usecase::delete::delete_book};

    mock! {
        DeleteBookRepoImpl {}

        #[async_trait]
        impl DeleteBookRepo for DeleteBookRepoImpl {
            async fn delete_book(&self, id: i32) -> Result<(), String>;
        }
    }

    #[tokio::test]
    async fn check_delete_book_success() {
        // Arrange
        let mut mock_repo = MockDeleteBookRepoImpl::new();

        let target_id = 1;
        mock_repo
            .expect_delete_book()
            .with(predicate::eq(target_id))
            .returning(|_| Ok(()));

        // Act
        let result: Result<(), String> = delete_book(&mock_repo, target_id).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn check_delete_book_id_not_found() {
        // Arrange
        let target_id = -32;
        let mut mock_repo = MockDeleteBookRepoImpl::new();
        mock_repo
            .expect_delete_book()
            .with(predicate::eq(target_id))
            .returning(|_| Err("존재하지 않는 id 입니다.".to_string()));

        // Act
        let result = delete_book(&mock_repo, target_id).await;

        // Assert
        assert!(result.is_err())
    }
}
