use std::sync::Arc;

use axum::async_trait;

use crate::{domain::user::repository::delete::DeleteUserRepo, global::errors::CustomError};

pub(crate) struct DeleteUserUsecaseImpl<T>
where
    T: DeleteUserRepo,
{
    repository: Arc<T>,
}

#[async_trait]
pub(crate) trait DeleteUserUsecase: Send + Sync {
    async fn delete_user(&self, id: i32) -> Result<(), Arc<CustomError>>;
}

impl<T> DeleteUserUsecaseImpl<T>
where
    T: DeleteUserRepo,
{
    pub(crate) fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> DeleteUserUsecase for DeleteUserUsecaseImpl<T>
where
    T: DeleteUserRepo,
{
    async fn delete_user(&self, id: i32) -> Result<(), Arc<CustomError>> {
        _delete_user(&*self.repository, id).await
    }
}

async fn _delete_user<T>(repository: &T, id: i32) -> Result<(), Arc<CustomError>>
where
    T: DeleteUserRepo,
{
    repository.delete_user(id).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::{domain::user::repository::delete::DeleteUserRepo, global::errors::CustomError};

    use super::_delete_user;

    mock! {
        DeleteUserRepoImpl {}

        #[async_trait]
        impl DeleteUserRepo for DeleteUserRepoImpl {
            async fn delete_user(&self, id: i32) -> Result<(), Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_delete_user_success() {
        // Arrange
        let id = 1;
        let mut mock_repo = MockDeleteUserRepoImpl::new();
        mock_repo
            .expect_delete_user()
            .with(predicate::eq(id))
            .returning(|_| Ok(()));

        // Act
        let result = _delete_user(&mock_repo, id).await;

        // Assert
        assert!(result.map_err(|e| println!("{:?}", e)).is_ok());
    }

    #[tokio::test]
    async fn check_delete_user_not_found() {
        // Arrange
        let id = -32;
        let mut mock_repo = MockDeleteUserRepoImpl::new();
        mock_repo
            .expect_delete_user()
            .with(predicate::eq(id))
            .returning(|_| Err(Arc::new(CustomError::NotFound("User".to_string()))));

        // Act
        let result = _delete_user(&mock_repo, id).await;

        // Assert
        assert!(result.is_err())
    }
}
