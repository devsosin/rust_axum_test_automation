use std::sync::Arc;

use axum::async_trait;

use crate::{
    domain::record::{dto::request::NewRecord, repository::save::SaveRecordRepo},
    global::errors::CustomError,
};

pub(crate) struct CreateRecordUsecaseImpl<T>
where
    T: SaveRecordRepo,
{
    repository: Arc<T>,
}

#[async_trait]
pub(crate) trait CreateRecordUsecase: Send + Sync {
    async fn create_record(&self, new_record: &NewRecord) -> Result<i64, Arc<CustomError>>;
}

impl<T> CreateRecordUsecaseImpl<T>
where
    T: SaveRecordRepo,
{
    pub(crate) fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> CreateRecordUsecase for CreateRecordUsecaseImpl<T>
where
    T: SaveRecordRepo,
{
    async fn create_record(&self, new_record: &NewRecord) -> Result<i64, Arc<CustomError>> {
        create_record(&*self.repository, new_record).await
    }
}

async fn create_record<T>(repository: &T, new_record: &NewRecord) -> Result<i64, Arc<CustomError>>
where
    T: SaveRecordRepo,
{
    let record = new_record.to_entity();
    let connect_ids = new_record.get_connect_ids();

    repository.validate_connect_ids(&connect_ids).await?;
    repository.save_record(record, connect_ids).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::NaiveDateTime;
    use mockall::{mock, predicate};
    use std::sync::Arc;

    use crate::domain::record::{
        dto::request::NewRecord, entity::Record, repository::save::SaveRecordRepo,
        usecase::create::create_record,
    };
    use crate::global::errors::CustomError;

    mock! {
        SaveRecordRepoImpl {}

        #[async_trait]
        impl SaveRecordRepo for SaveRecordRepoImpl {
            async fn save_record(&self, record: Record, connect_ids: Option<Vec<i32>>) -> Result<i64, Arc<CustomError>>;
            async fn validate_connect_ids(&self, connect_ids: &Option<Vec<i32>>) -> Result<(), Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_create_record_success() {
        // Arrange
        let new_record = NewRecord::new(
            1,
            18,
            16500,
            Some("감자탕".to_string()),
            NaiveDateTime::parse_from_str("2024-09-08 15:30:37", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
            None,
        );

        let mut mock_repo = MockSaveRecordRepoImpl::new();
        mock_repo
            .expect_save_record()
            .with(predicate::eq(new_record.to_entity()), predicate::eq(None))
            .returning(|_, _| Ok(1));
        mock_repo
            .expect_validate_connect_ids()
            .with(predicate::eq(None))
            .returning(|_| Ok(()));

        // Act
        let result = create_record(&mock_repo, &new_record).await;
        assert!(result.is_ok());
        let inserted_id = result.unwrap();

        // Assert
        assert_eq!(inserted_id, 1);
    }

    #[tokio::test]
    async fn check_sub_category_invalid() {
        // Arrange
        let new_record = NewRecord::new(
            1,
            -32,
            15000,
            Some("감자탕".to_string()),
            NaiveDateTime::parse_from_str("2024-09-08 15:30:37", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
            None,
        );

        // tb_connect_record

        let mut mock_repo = MockSaveRecordRepoImpl::new();
        mock_repo
            .expect_save_record()
            .with(predicate::eq(new_record.to_entity()), predicate::eq(None))
            .returning(|_, _| Err(Arc::new(CustomError::NotFound("Category".to_string()))));
        mock_repo
            .expect_validate_connect_ids()
            .with(predicate::eq(None))
            .returning(|_| Ok(()));

        // Act
        let result = create_record(&mock_repo, &new_record).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_asset_id_not_found() {
        // Arrange
        let new_connections = Some(vec![1, 2]);
        let new_record = NewRecord::new(
            1,
            18,
            15000,
            Some("감자탕".to_string()),
            NaiveDateTime::parse_from_str("2024-09-08 15:30:37", "%Y-%m-%d %H:%M:%S").unwrap(),
            Some(-32),
            new_connections.clone(),
        );

        let mut mock_repo = MockSaveRecordRepoImpl::new();
        mock_repo
            .expect_save_record()
            .with(
                predicate::eq(new_record.to_entity()),
                predicate::eq(new_connections.clone()),
            )
            .returning(|_, _| Err(Arc::new(CustomError::NotFound("Asset".to_string()))));
        mock_repo
            .expect_validate_connect_ids()
            .with(predicate::eq(Some(vec![1, 2])))
            .returning(|_| Ok(()));

        // Act
        let result = create_record(&mock_repo, &new_record).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_no_role_in_book() {
        todo!()

        // Arrange

        // Act

        // Assert
    }
}
