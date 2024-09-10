use std::sync::Arc;

use axum::async_trait;

use crate::{
    domain::record::{dto::request::EditRecord, repository::update::UpdateRecordRepo},
    global::errors::CustomError,
};

pub(crate) struct UpdateRecordUsecaseImpl<T>
where
    T: UpdateRecordRepo,
{
    repository: Arc<T>,
}

#[async_trait]
pub trait UpdateRecordUsecase: Send + Sync {
    async fn update_record(&self, id: i64, edit_record: EditRecord)
        -> Result<(), Arc<CustomError>>;
}

impl<T> UpdateRecordUsecaseImpl<T>
where
    T: UpdateRecordRepo,
{
    pub(crate) fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> UpdateRecordUsecase for UpdateRecordUsecaseImpl<T>
where
    T: UpdateRecordRepo,
{
    async fn update_record(
        &self,
        id: i64,
        edit_record: EditRecord,
    ) -> Result<(), Arc<CustomError>> {
        update_record(&*self.repository, id, edit_record).await
    }
}

async fn update_record<T>(
    repository: &T,
    id: i64,
    edit_record: EditRecord,
) -> Result<(), Arc<CustomError>>
where
    T: UpdateRecordRepo,
{
    repository.update_record(id, edit_record.to_update()).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use mockall::{mock, predicate};

    use crate::{
        domain::record::{
            dto::request::EditRecord, entity::UpdateRecord, repository::update::UpdateRecordRepo,
        },
        global::errors::CustomError,
    };

    use super::update_record;

    mock! {
        UpdateRecordRepoImpl {}

        #[async_trait]
        impl UpdateRecordRepo for UpdateRecordRepoImpl {
            async fn update_record(&self, id: i64, edit_record: UpdateRecord) -> Result<(), Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_update_record_success() {
        // Arrange
        let edit_record = EditRecord::new(None, Some(15000), Some("NULL".to_string()), None, None);

        let id = 1;

        let mut mock_repo = MockUpdateRecordRepoImpl::new();
        mock_repo
            .expect_update_record()
            .with(
                predicate::eq(id),
                predicate::eq(edit_record.clone().to_update()),
            )
            .returning(|_, _| Ok(()));

        // Act
        let result = update_record(&mock_repo, id, edit_record).await;

        // Assert
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn check_update_record_failure() {
        // Arrange
        let edit_record = EditRecord::new(None, Some(15000), Some("NULL".to_string()), None, None);

        let id = -32;

        let mut mock_repo = MockUpdateRecordRepoImpl::new();
        mock_repo
            .expect_update_record()
            .with(
                predicate::eq(id),
                predicate::eq(edit_record.clone().to_update()),
            )
            .returning(|_, _| Err(Arc::new(CustomError::NotFound("Record".to_string()))));

        // Act
        let result = update_record(&mock_repo, id, edit_record).await;

        // Assert
        assert!(result.is_err())
    }
}
