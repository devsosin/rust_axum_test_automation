use std::sync::Arc;

use axum::async_trait;

use crate::{
    domain::record::{dto::request::EditRecord, repository::update::UpdateRecordRepo},
    global::errors::CustomError,
};

pub struct UpdateRecordUsecaseImpl<T>
where
    T: UpdateRecordRepo,
{
    repository: T,
}

#[async_trait]
pub trait UpdateRecordUsecase: Send + Sync {
    async fn update_record(
        &self,
        user_id: i32,
        record_id: i64,
        edit_record: EditRecord,
    ) -> Result<(), Arc<CustomError>>;
}

impl<T> UpdateRecordUsecaseImpl<T>
where
    T: UpdateRecordRepo,
{
    pub fn new(repository: T) -> Self {
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
        user_id: i32,
        record_id: i64,
        edit_record: EditRecord,
    ) -> Result<(), Arc<CustomError>> {
        update_record(&self.repository, user_id, record_id, edit_record).await
    }
}

async fn update_record<T>(
    repository: &T,
    user_id: i32,
    record_id: i64,
    edit_record: EditRecord,
) -> Result<(), Arc<CustomError>>
where
    T: UpdateRecordRepo,
{
    repository
        .update_record(user_id, record_id, edit_record.to_update())
        .await
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
            async fn update_record(&self, user_id: i32, record_id: i64, edit_record: UpdateRecord) -> Result<(), Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_update_record_success() {
        // Arrange
        let edit_record = EditRecord::new(None, Some(15000), Some("NULL".to_string()), None, None);
        let user_id = 1;

        let record_id = 1i64;

        let mut mock_repo = MockUpdateRecordRepoImpl::new();
        mock_repo
            .expect_update_record()
            .with(
                predicate::eq(user_id),
                predicate::eq(record_id),
                predicate::eq(edit_record.clone().to_update()),
            )
            .returning(|_, _, _| Ok(()));

        // Act
        let result = update_record(&mock_repo, user_id, record_id, edit_record).await;

        // Assert
        assert!(result.is_ok())
    }
}
