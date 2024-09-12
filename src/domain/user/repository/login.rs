use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::user::entity::User, global::errors::CustomError};

pub(crate) struct LoginUserRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub(crate) trait LoginUserRepo: Send + Sync {
    async fn get_by_username(&self, username: &str) -> Result<User, Arc<CustomError>>;
}

impl LoginUserRepoImpl {
    pub(crate) fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl LoginUserRepo for LoginUserRepoImpl {
    async fn get_by_username(&self, username: &str) -> Result<User, Arc<CustomError>> {
        _get_by_username(&self.pool, username).await
    }
}

async fn _get_by_username(pool: &PgPool, username: &str) -> Result<User, Arc<CustomError>> {
    let row = sqlx::query_as::<_, User>("SELECT * FROM tb_user WHERE username = $1")
        .bind(username)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(GetByUsername {}): {:?}", username, &e);
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
        domain::user::{
            entity::User,
            repository::{get_user::get_by_id, save::save_user},
        },
    };

    use super::_get_by_username;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_get_by_username_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let username = "login_test@test.test";

        let user = User::new(
            username.to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "test1234@test.test".to_string(),
            "email".to_string(),
        );

        let new_id = save_user(&pool, user).await.unwrap();

        // Act
        let result = _get_by_username(&pool, username).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let result = result.unwrap();

        // Assert
        let user = get_by_id(&pool, new_id).await.unwrap();

        assert_eq!(user.get_id(), result.get_id())
    }

    #[tokio::test]
    async fn check_username_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let username = "not_found_username@test.test";

        // Act
        let result = _get_by_username(&pool, username).await;

        // Assert
        assert!(result.is_err())
    }
}
