use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::user::entity::User, global::errors::CustomError};

pub(crate) struct GetUserRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub(crate) trait GetUserRepo: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<User, Arc<CustomError>>;
}

impl GetUserRepoImpl {
    pub(crate) fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GetUserRepo for GetUserRepoImpl {
    async fn get_by_id(&self, id: i32) -> Result<User, Arc<CustomError>> {
        get_by_id(&self.pool, id).await
    }
}

pub(crate) async fn get_by_id(pool: &PgPool, id: i32) -> Result<User, Arc<CustomError>> {
    let row = sqlx::query_as::<_, User>("SELECT * FROM tb_user WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(GetUser {}): {:?}", id, &e);
            tracing::error!("{}", err_msg);

            let err = match e {
                sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                sqlx::Error::RowNotFound => CustomError::NotFound("User".to_string()),
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
        domain::user::{entity::User, repository::save::save_user},
    };

    use super::get_by_id;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_get_user_success() {
        // Arrange
        let pool = create_connection_pool().await;
        let user = User::new(
            "gettest@gettest.test".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "email".to_string(),
        );
        let new_id = save_user(&pool, user.clone()).await.unwrap();

        // Act
        let result = get_by_id(&pool, new_id).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.get_user_email(), user.get_user_email())
    }

    #[tokio::test]
    async fn check_user_not_found() {
        // Arrange
        let pool = create_connection_pool().await;
        let no_id = -32;

        // Act
        let reuslt = get_by_id(&pool, no_id).await;

        // Assert
        assert!(reuslt.is_err())
    }
}
