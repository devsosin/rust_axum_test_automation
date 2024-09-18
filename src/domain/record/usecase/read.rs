use std::sync::Arc;

use axum::async_trait;

use crate::{
    domain::record::{entity::Record, repository::get_record::GetRecordRepo},
    global::errors::CustomError,
};

pub struct ReadRecordUsecaseImpl<T>
where
    T: GetRecordRepo,
{
    repository: T,
}

#[async_trait]
pub trait ReadRecordUsecase: Send + Sync {
    async fn read_records(&self, user_id: i32) -> Result<Vec<Record>, Arc<CustomError>>;
    async fn read_record(&self, user_id: i32, record_id: i64) -> Result<Record, Arc<CustomError>>;
}

impl<T> ReadRecordUsecaseImpl<T>
where
    T: GetRecordRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> ReadRecordUsecase for ReadRecordUsecaseImpl<T>
where
    T: GetRecordRepo,
{
    async fn read_records(&self, user_id: i32) -> Result<Vec<Record>, Arc<CustomError>> {
        read_records(&self.repository, user_id).await
    }

    async fn read_record(&self, user_id: i32, record_id: i64) -> Result<Record, Arc<CustomError>> {
        read_record(&self.repository, user_id, record_id).await
    }
}

async fn read_records<T>(repository: &T, user_id: i32) -> Result<Vec<Record>, Arc<CustomError>>
where
    T: GetRecordRepo,
{
    repository.get_list(user_id).await
}

async fn read_record<T>(
    repository: &T,
    user_id: i32,
    record_id: i64,
) -> Result<Record, Arc<CustomError>>
where
    T: GetRecordRepo,
{
    repository.get_by_id(user_id, record_id).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::async_trait;
    use chrono::NaiveDateTime;
    use mockall::{mock, predicate};

    use crate::{
        domain::record::{
            entity::Record,
            repository::get_record::GetRecordRepo,
            usecase::read::{read_record, read_records},
        },
        global::errors::CustomError,
    };

    mock! {
        GetRecordRepoImpl {}

        #[async_trait]
        impl GetRecordRepo for GetRecordRepoImpl {
            async fn get_list(&self, user_id: i32) -> Result<Vec<Record>, Arc<CustomError>>;
            async fn get_by_id(&self, user_id: i32, record_id: i64) -> Result<Record, Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_read_records_success() {
        // Arrange
        let user_id = 1;

        let mut mock_repo = MockGetRecordRepoImpl::new();
        mock_repo
            .expect_get_list()
            .with(predicate::eq(user_id))
            .returning(|_| {
                Ok(vec![
                    Record::new(
                        1,
                        18,
                        15000,
                        NaiveDateTime::parse_from_str("2024-09-08 15:30:27", "%Y-%m-%d %H:%M:%S")
                            .unwrap(),
                        None,
                    )
                    .id(Some(1))
                    .build(),
                    Record::new(
                        1,
                        18,
                        15000,
                        NaiveDateTime::parse_from_str("2024-09-08 15:30:27", "%Y-%m-%d %H:%M:%S")
                            .unwrap(),
                        None,
                    )
                    .id(Some(2))
                    .build(),
                    Record::new(
                        1,
                        18,
                        15000,
                        NaiveDateTime::parse_from_str("2024-09-08 15:30:27", "%Y-%m-%d %H:%M:%S")
                            .unwrap(),
                        None,
                    )
                    .id(Some(3))
                    .build(),
                ])
            });

        // Act
        let result = read_records::<MockGetRecordRepoImpl>(&mock_repo, user_id).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn check_read_record_success() {
        // Arrange
        let user_id = 1;
        let record_id = 1;

        let mut mock_repo = MockGetRecordRepoImpl::new();
        mock_repo
            .expect_get_by_id()
            .with(predicate::eq(user_id), predicate::eq(record_id))
            .returning(|_, i| {
                Ok(Record::new(
                    1,
                    18,
                    15200,
                    NaiveDateTime::parse_from_str("2024-09-27 15:30:47", "%Y-%m-%d %H:%M:%S")
                        .unwrap(),
                    None,
                )
                .id(Some(i))
                .build())
            });

        // Act
        let result = read_record(&mock_repo, user_id, record_id).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_id(), record_id);
    }

    #[tokio::test]
    async fn check_read_record_not_found() {
        // Arrange
        let user_id = 1;
        let no_id = -32;
        let mut mock_repo = MockGetRecordRepoImpl::new();
        mock_repo
            .expect_get_by_id()
            .with(predicate::eq(user_id), predicate::eq(no_id))
            .returning(|_, _| Err(Arc::new(CustomError::NotFound("Record".to_string()))));

        // Act
        let result = read_record(&mock_repo, user_id, no_id).await;

        // Assert
        assert!(result.is_err())
    }
}
