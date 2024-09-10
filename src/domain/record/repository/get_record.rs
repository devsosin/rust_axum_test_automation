use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::record::entity::Record, global::errors::CustomError};

pub(crate) struct GetRecordRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub(crate) trait GetRecordRepo: Send + Sync {
    async fn get_list(&self) -> Result<Vec<Record>, Arc<CustomError>>;
    async fn get_by_id(&self, id: i64) -> Result<Record, Arc<CustomError>>;
}

impl GetRecordRepoImpl {
    pub(crate) fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GetRecordRepo for GetRecordRepoImpl {
    async fn get_list(&self) -> Result<Vec<Record>, Arc<CustomError>> {
        get_list(&self.pool).await
    }
    async fn get_by_id(&self, id: i64) -> Result<Record, Arc<CustomError>> {
        get_by_id(&self.pool, id).await
    }
}

async fn get_list(pool: &PgPool) -> Result<Vec<Record>, Arc<CustomError>> {
    let rows = sqlx::query_as::<_, Record>("SELECT * FROM tb_record")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(GetRecords): {:?}", &e);
            tracing::error!("{}", err_msg);

            let err = match e {
                sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                _ => CustomError::Unexpected(e.into()),
            };
            Arc::new(err)
        })?;

    Ok(rows)
}

pub(crate) async fn get_by_id(pool: &PgPool, id: i64) -> Result<Record, Arc<CustomError>> {
    let row = sqlx::query_as::<_, Record>("SELECT * FROM tb_record WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(GetRecord {}): {:?}", id, &e);
            tracing::error!("{}", err_msg);

            let err = match e {
                sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                sqlx::Error::RowNotFound => CustomError::NotFound("Record".to_string()),
                _ => CustomError::Unexpected(e.into()),
            };
            Arc::new(err)
        })?;

    Ok(row)
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::record::{
            entity::Record,
            repository::get_record::{get_by_id, get_list},
        },
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_get_list_success() {
        // Arrange
        let pool = create_connection_pool().await;

        // Act
        let result = get_list(&pool).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        let rows = sqlx::query_as::<_, Record>("SELECT * FROM tb_record")
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(result.len(), rows.len());
    }

    #[tokio::test]
    async fn check_get_list_failure() {
        // Arrange
        todo!()

        // Act

        // Assert
    }

    #[tokio::test]
    async fn check_get_by_id_success() {
        // Arrange
        let pool = create_connection_pool().await;
        let id = 1i64;

        // Act
        let result = get_by_id(&pool, id).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        let row = sqlx::query_as::<_, Record>("SELECT * FROM tb_record WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.get_amount(), row.get_amount())
    }

    #[tokio::test]
    async fn check_get_by_id_not_found() {
        // Arrange
        let pool = create_connection_pool().await;
        let id = -32i64;

        // Act
        let result = get_by_id(&pool, id).await;

        // Assert
        assert!(result.is_err())
    }
}
