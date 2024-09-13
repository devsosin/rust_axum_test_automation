use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::global::errors::CustomError;

pub struct DeleteRecordRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait DeleteRecordRepo: Send + Sync {
    async fn delete_record(&self, id: i64) -> Result<(), Arc<CustomError>>;
}

impl DeleteRecordRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl DeleteRecordRepo for DeleteRecordRepoImpl {
    async fn delete_record(&self, id: i64) -> Result<(), Arc<CustomError>> {
        delete_record(&self.pool, id).await
    }
}

async fn delete_record(pool: &PgPool, id: i64) -> Result<(), Arc<CustomError>> {
    let result = sqlx::query("DELETE FROM tb_record WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(DeleteRecord {}): {:?}", id, &e);
            tracing::error!("{}", err_msg);

            let err = match e {
                sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                _ => CustomError::Unexpected(e.into()),
            };
            Arc::new(err)
        })?;

    if result.rows_affected() == 0 {
        return Err(Arc::new(CustomError::NotFound("Record".to_string())));
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

        let record = Record::new(
            1,
            18, // 식비
            16300,
            NaiveDateTime::parse_from_str("2024-09-08 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
        );

        let new_id = save_record(&pool, record, None).await.unwrap();

        // Act
        let result = delete_record(&pool, new_id).await;
        assert!(result.is_ok());

        // Assert
        let row = get_by_id(&pool, new_id).await;
        assert!(row.is_err())
    }

    #[tokio::test]
    async fn check_delete_record_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let no_id = -32i64;

        // Act
        let result = delete_record(&pool, no_id).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn check_delete_record_no_role() {
        todo!()

        // Arrange

        // Act

        // Assert
    }
}
