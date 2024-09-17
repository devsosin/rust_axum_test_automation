use axum::async_trait;

use crate::{
    domain::record::{dto::request::NewRecord, repository::save::SaveRecordRepo},
    global::errors::CustomError,
};

pub struct CreateRecordUsecaseImpl<T>
where
    T: SaveRecordRepo,
{
    repository: T,
}

#[async_trait]
pub trait CreateRecordUsecase: Send + Sync {
    async fn create_record(
        &self,
        user_id: i32,
        new_record: NewRecord,
    ) -> Result<i64, Box<CustomError>>;
}

impl<T> CreateRecordUsecaseImpl<T>
where
    T: SaveRecordRepo,
{
    pub fn new(repository: T) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<T> CreateRecordUsecase for CreateRecordUsecaseImpl<T>
where
    T: SaveRecordRepo,
{
    async fn create_record(
        &self,
        user_id: i32,
        new_record: NewRecord,
    ) -> Result<i64, Box<CustomError>> {
        create_record(&self.repository, user_id, new_record).await
    }
}

async fn create_record<T>(
    repository: &T,
    user_id: i32,
    new_record: NewRecord,
) -> Result<i64, Box<CustomError>>
where
    T: SaveRecordRepo,
{
    let record = new_record.to_entity();
    let connect_ids = new_record.get_connect_ids();

    repository.save_record(user_id, record, connect_ids).await
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::NaiveDateTime;
    use mockall::{mock, predicate};

    use crate::domain::record::{
        dto::request::NewRecord, entity::Record, repository::save::SaveRecordRepo,
        usecase::create::create_record,
    };
    use crate::global::errors::CustomError;

    mock! {
        SaveRecordRepoImpl {}

        #[async_trait]
        impl SaveRecordRepo for SaveRecordRepoImpl {
            async fn save_record(&self, user_id: i32, record: Record, connect_ids: Option<Vec<i32>>) -> Result<i64, Box<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_create_record_success() {
        // Arrange
        let user_id = 1;
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
            .with(
                predicate::eq(user_id),
                predicate::eq(new_record.to_entity()),
                predicate::eq(None),
            )
            .returning(|_, _, _| Ok(1));

        // Act
        let result = create_record(&mock_repo, user_id, new_record).await;
        assert!(result.is_ok());
        let inserted_id = result.unwrap();

        // Assert
        assert_eq!(inserted_id, 1);
    }
}
