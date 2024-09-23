use std::sync::Arc;

use axum::async_trait;
use sqlx::{PgPool, Row};

use crate::global::errors::CustomError;

pub struct SaveConnectRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait SaveConnectRepo: Send + Sync {
    async fn save_connect(&self, name: String) -> Result<i32, Box<CustomError>>;
}

impl SaveConnectRepoImpl {
    pub fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl SaveConnectRepo for SaveConnectRepoImpl {
    async fn save_connect(&self, name: String) -> Result<i32, Box<CustomError>> {
        save_connect(&self.pool, name).await
    }
}

pub async fn save_connect(pool: &PgPool, name: String) -> Result<i32, Box<CustomError>> {
    let result = sqlx::query(
        "
        INSERT INTO tb_connect (name) 
        VALUES ($1)
        ON CONFLICT (name) DO NOTHING
        RETURNING id;
        ",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(SaveConnect): {:?}", &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Box::new(err)
    })?;

    if result.is_none() {
        return Err(Box::new(CustomError::Duplicated("Connect".to_string())));
    }

    Ok(result.unwrap().get("id"))
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool, domain::connect::entity::Connect,
        global::errors::CustomError,
    };

    use super::save_connect;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_save_connect_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let name = "새로운 커넥트";

        // Act
        let result = save_connect(&pool, name.to_string()).await;
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_ok());
        let connect_id = result.unwrap();

        // Assert
        let row = sqlx::query_as::<_, Connect>("SELECT * FROM tb_connect WHERE id = $1")
            .bind(connect_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(row.get_name(), name)
    }

    #[tokio::test]
    async fn check_duplicated() {
        // Arrange
        let pool = create_connection_pool().await;

        let name = "중복되는 커넥트";
        let _ = save_connect(&pool, name.to_string()).await;

        // Act
        let result = save_connect(&pool, name.to_string()).await;

        // Assert
        assert!(result.is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::Duplicated(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}
