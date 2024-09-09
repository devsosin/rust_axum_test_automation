use std::sync::Arc;

use axum::async_trait;

use crate::domain::record::{entity::Record, repository::get_record::GetRecordRepo};

pub(crate) struct ReadRecordUsecaseImpl<T>
where
    T: GetRecordRepo,
{
    repository: Arc<T>,
}

#[async_trait]
pub(crate) trait ReadRecordUsecase: Send + Sync {
    async fn read_records(&self) -> Result<Vec<Record>, String>;
    async fn read_record(&self, id: i64) -> Result<Record, String>;
}

impl<T> ReadRecordUsecaseImpl<T>
where
    T: GetRecordRepo,
{
    pub(crate) fn new(repository: Arc<T>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> ReadRecordUsecase for ReadRecordUsecaseImpl<T>
where
    T: GetRecordRepo,
{
    async fn read_records(&self) -> Result<Vec<Record>, String> {
        read_records(&*self.repository).await
    }

    async fn read_record(&self, id: i64) -> Result<Record, String> {
        read_record(&*self.repository, id).await
    }
}

async fn read_records<T>(repository: &T) -> Result<Vec<Record>, String>
where
    T: GetRecordRepo,
{
    repository.get_list().await
}

async fn read_record<T>(repository: &T, id: i64) -> Result<Record, String>
where
    T: GetRecordRepo,
{
    repository.get_by_id(id).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::NaiveDateTime;
    use mockall::{mock, predicate};

    use crate::domain::record::{
        entity::Record,
        repository::get_record::GetRecordRepo,
        usecase::read::{read_record, read_records},
    };

    mock! {
        GetRecordRepoImpl {}

        #[async_trait]
        impl GetRecordRepo for GetRecordRepoImpl {
            async fn get_list(&self) -> Result<Vec<Record>, String>;
            async fn get_by_id(&self, id: i64) -> Result<Record, String>;
        }
    }

    #[tokio::test]
    async fn check_read_records_success() {
        // Arrange
        let mut mock_repo = MockGetRecordRepoImpl::new();
        mock_repo.expect_get_list().returning(|| {
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
        let result = read_records::<MockGetRecordRepoImpl>(&mock_repo).await;
        assert!(result.clone().map_err(|e| println!("{}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn check_read_records_failure() {
        todo!()

        // Arrange

        // Act

        // Assert
    }

    #[tokio::test]
    async fn check_read_record_success() {
        // Arrange
        let id = 1;

        let mut mock_repo = MockGetRecordRepoImpl::new();
        mock_repo
            .expect_get_by_id()
            .with(predicate::eq(id))
            .returning(|i| {
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
        let result = read_record(&mock_repo, id).await;
        assert!(result.clone().map_err(|e| println!("{}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_id(), id);
    }

    #[tokio::test]
    async fn check_read_record_not_found() {
        // Arrange
        let id = -32;
        let mut mock_repo = MockGetRecordRepoImpl::new();
        mock_repo
            .expect_get_by_id()
            .with(predicate::eq(id))
            .returning(|i| Err(format!("id: {} not found", i)));

        // Act
        let result = read_record(&mock_repo, id).await;

        // Assert
        assert!(result.is_err())
    }
}
