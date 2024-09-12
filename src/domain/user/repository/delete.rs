use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::global::errors::CustomError;

pub(crate) struct DeleteUserRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub(crate) trait DeleteUserRepo: Send + Sync {
    async fn delete_user(&self, id: i32) -> Result<(), Arc<CustomError>>;
}

impl DeleteUserRepoImpl {
    pub(crate) fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl DeleteUserRepo for DeleteUserRepoImpl {
    async fn delete_user(&self, id: i32) -> Result<(), Arc<CustomError>> {
        _delete_user(&self.pool, id).await
    }
}

async fn _delete_user(pool: &PgPool, id: i32) -> Result<(), Arc<CustomError>> {
    let result =
        sqlx::query("UPDATE tb_user SET is_active = FALSE, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                let err_msg = format!("Error(DeleteUser {}): {:?}", id, &e);
                tracing::error!("{}", err_msg);

                let err = match e {
                    sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                    _ => CustomError::Unexpected(e.into()),
                };
                Arc::new(err)
            })?;

    if result.rows_affected() == 0 {
        return Err(Arc::new(CustomError::NotFound("User".to_string())));
    }

    Ok(())
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

    use super::_delete_user;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_delete_user_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user = User::new(
            "deltest1234@test.test".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "deltest1234@test.test".to_string(),
            "email".to_string(),
        );

        let new_id = save_user(&pool, user).await.unwrap();

        // Act
        let result = _delete_user(&pool, new_id).await;
        assert!(result.map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let user = get_by_id(&pool, new_id).await.unwrap();

        assert_eq!(user.get_is_active(), false)
    }

    #[tokio::test]
    async fn check_user_not_found() {
        // Arrange
        let pool = create_connection_pool().await;
        let id = -32;

        // Act
        let result = _delete_user(&pool, id).await;

        // Assert
        assert!(result.is_err())
    }
}
