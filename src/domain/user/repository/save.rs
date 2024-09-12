use std::sync::Arc;

use axum::async_trait;
use sqlx::{PgPool, Row};

use crate::{domain::user::entity::User, global::errors::CustomError};

pub(crate) struct SaveUserRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub(crate) trait SaveUserRepo: Send + Sync {
    async fn save_user(&self, user: User) -> Result<i32, Arc<CustomError>>;
}

impl SaveUserRepoImpl {
    pub(crate) fn new(pool: &Arc<PgPool>) -> Self {
        Self { pool: pool.clone() }
    }
}

#[async_trait]
impl SaveUserRepo for SaveUserRepoImpl {
    async fn save_user(&self, user: User) -> Result<i32, Arc<CustomError>> {
        save_user(&self.pool, user).await
    }
}

pub(crate) async fn save_user(pool: &PgPool, user: User) -> Result<i32, Arc<CustomError>> {
    let result = sqlx::query(
        "INSERT INTO tb_user (username, password, nickname, phone, 
                                        login_type, email, access_token) 
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (username) DO NOTHING
            RETURNING id;
    ",
    )
    .bind(user.get_username())
    .bind(user.get_password())
    .bind(user.get_nickname())
    .bind(user.get_phone())
    .bind(user.get_login_type().to_string())
    .bind(user.get_email())
    .bind(user.get_access_token())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(SaveUser): {:?}", &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            sqlx::Error::RowNotFound => CustomError::Duplicated("User".to_string()),
            _ => CustomError::Unexpected(e.into()),
        };
        Arc::new(err)
    })?;

    let id = result.get("id");
    Ok(id)
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::user::{entity::User, repository::save::save_user},
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_save_user_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let user = User::new(
            "test1234@test.test".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "test1234@test.test".to_string(),
            "email".to_string(),
        );

        // Act
        let result = save_user(&pool, user).await;
        assert!(result.clone().map_err(|e| println!("{:?}", e)).is_ok());
        let inserted_id = result.unwrap();

        // Assert
        let row = sqlx::query_as::<_, User>("SELECT * FROM tb_user WHERE id = $1")
            .bind(inserted_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        assert_eq!(row.get_id().unwrap(), inserted_id)
    }

    #[tokio::test]
    async fn check_save_user_email_duplicated() {
        // Arrange
        let pool = create_connection_pool().await;
        let user1 = User::new(
            "test_dupl@test.test".to_string(),
            "test_password".to_string(),
            "nickname".to_string(),
            "test@test.test".to_string(),
            "email".to_string(),
        );
        save_user(&pool, user1).await.unwrap();

        let user2 = User::new(
            "test_dupl@test.test".to_string(),
            "test_password".to_string(),
            "duplicate_user".to_string(),
            "test@test.test".to_string(),
            "email".to_string(),
        );

        // Act
        let result = save_user(&pool, user2).await;

        // Assert
        assert!(result.is_err())
    }
}
