use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::connect::entity::Connect, global::errors::CustomError};

pub struct GetConnectRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait GetConnectRepo: Send + Sync {
    async fn get_connect_by_name(&self, name: String) -> Result<Connect, Box<CustomError>>;
}

impl GetConnectRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl GetConnectRepo for GetConnectRepoImpl {
    async fn get_connect_by_name(&self, name: String) -> Result<Connect, Box<CustomError>> {
        get_connect_by_name(&self.pool, name).await
    }
}

pub async fn get_connect_by_name(pool: &PgPool, name: String) -> Result<Connect, Box<CustomError>> {
    let result = sqlx::query_as::<_, Connect>(
        "
        SELECT * FROM tb_connect WHERE name = $1
        ",
    )
    .bind(name.as_str())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(GetConnect {}): {:?}", name, &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            sqlx::Error::RowNotFound => CustomError::NotFound("Connect".to_string()),
            _ => CustomError::Unexpected(e.into()),
        };
        Box::new(err)
    })?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool, domain::connect::repository::save::save_connect,
        global::errors::CustomError,
    };

    use super::get_connect_by_name;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_get_connect_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let name = "조회용 커넥트";
        let _ = save_connect(&pool, name.to_string()).await;

        // Act
        let result = get_connect_by_name(&pool, name.to_string()).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_name(), name)
    }

    #[tokio::test]
    async fn check_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let name = "없는 커넥트";

        // Act
        let result = get_connect_by_name(&pool, name.to_string()).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}
