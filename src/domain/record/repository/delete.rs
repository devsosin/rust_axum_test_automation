use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::global::{constants::DeleteResult, errors::CustomError};

pub struct DeleteRecordRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait DeleteRecordRepo: Send + Sync {
    async fn delete_record(&self, user_id: i32, record_id: i64) -> Result<(), Arc<CustomError>>;
}

impl DeleteRecordRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl DeleteRecordRepo for DeleteRecordRepoImpl {
    async fn delete_record(&self, user_id: i32, record_id: i64) -> Result<(), Arc<CustomError>> {
        delete_record(&self.pool, user_id, record_id).await
    }
}

async fn delete_record(
    pool: &PgPool,
    user_id: i32,
    record_id: i64,
) -> Result<(), Arc<CustomError>> {
    let result = sqlx::query_as::<_, DeleteResult>(
        r"
        WITH RecordExists AS (
            SELECT book_id
            FROM tb_record
            WHERE id = $2
        ),
        AuthorityCheck AS (
            SELECT r.book_id
            FROM RecordExists AS r
            JOIN tb_book AS b ON b.id = r.book_id
            JOIN tb_user_book_role AS br ON b.id = br.book_id
            WHERE br.user_id = $1 AND br.role != 'viewer'
        ),
        DeleteRecord AS (
            DELETE FROM tb_record 
            WHERE id = $2
                AND EXISTS (SELECT 1 FROM AuthorityCheck) = true
            RETURNING id
        )
        SELECT
            EXISTS (SELECT 1 FROM RecordExists) AS is_exist,
            EXISTS (SELECT 1 FROM AuthorityCheck) AS is_authorized,
            (SELECT COUNT(*) FROM DeleteRecord) AS delete_count

        ",
    )
    .bind(user_id)
    .bind(record_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(DeleteRecord {}): {:?}", record_id, &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Arc::new(err)
    })?;

    if !result.get_exist() {
        return Err(Arc::new(CustomError::NotFound("Record".to_string())));
    } else if !result.get_authorized() {
        return Err(Arc::new(CustomError::Unauthorized(
            "RecordRole".to_string(),
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDateTime;

    use crate::{
        config::database::create_connection_pool,
        domain::record::{
            entity::Record,
            repository::{get_record::get_by_id, save::save_record},
        },
        global::errors::CustomError,
    };

    use super::delete_record;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_delete_record_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        let new_id = save_record(&pool, user_id, record, None).await.unwrap();

        // Act
        let result = delete_record(&pool, user_id, new_id).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let row = get_by_id(&pool, user_id, new_id).await;
        assert!(row.is_err())
    }

    #[tokio::test]
    async fn check_record_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let no_id = -32i64;

        // Act
        let result = delete_record(&pool, user_id, no_id).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result);
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_unauthorized() {
        // Arrange
        let pool = create_connection_pool().await;

        // ref) init.sql
        let viewer_id = 2;
        let record_id = 1i64;

        // Act
        let result = delete_record(&pool, viewer_id, record_id).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result);
        let err_type = match *result.err().unwrap() {
            CustomError::Unauthorized(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}
