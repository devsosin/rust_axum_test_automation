use std::sync::Arc;

use axum::async_trait;

use crate::{domain::record::repository::delete::DeleteRecordRepo, global::errors::CustomError};

pub struct DeleteRecordUsecaseImpl<T>
where
    T: DeleteRecordRepo,
{
    repository: T,
}

#[async_trait]
pub trait DeleteRecordUsecase: Send + Sync {
    async fn delete_record(&self, user_id: i32, record_id: i64) -> Result<(), Arc<CustomError>>;
}

impl<T> DeleteRecordUsecaseImpl<T>
where
    T: DeleteRecordRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> DeleteRecordUsecase for DeleteRecordUsecaseImpl<T>
where
    T: DeleteRecordRepo,
{
    async fn delete_record(&self, user_id: i32, record_id: i64) -> Result<(), Arc<CustomError>> {
        delete_record(&self.repository, user_id, record_id).await
    }
}

async fn delete_record<T>(
    repository: &T,
    user_id: i32,
    record_id: i64,
) -> Result<(), Arc<CustomError>>
where
    T: DeleteRecordRepo,
{
    repository.delete_record(user_id, record_id).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::{
        domain::record::repository::delete::DeleteRecordRepo, global::errors::CustomError,
    };

    use super::delete_record;

    mock! {
        DeleteRecordRepoImpl {}

        #[async_trait]
        impl DeleteRecordRepo for DeleteRecordRepoImpl{
            async fn delete_record(&self, user_id: i32, record_id: i64) -> Result<(), Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_delete_record_success() {
        // Arrange
        let user_id = 1;
        let record_id = 1;

        let mut mock_repo = MockDeleteRecordRepoImpl::new();
        mock_repo
            .expect_delete_record()
            .with(predicate::eq(user_id), predicate::eq(record_id))
            .returning(|_, _| Ok(()));

        // Act
        let result = delete_record(&mock_repo, user_id, record_id).await;

        // Assert
        assert!(result.is_ok())
    }
}
