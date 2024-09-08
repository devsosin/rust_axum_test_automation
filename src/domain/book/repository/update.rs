use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

pub struct UpdateBookRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait UpdateBookRepo: Send + Sync {
    async fn update_book(&self, id: i32, name: &str) -> Result<(), String>;
}

impl UpdateBookRepoImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UpdateBookRepo for UpdateBookRepoImpl {
    async fn update_book(&self, id: i32, name: &str) -> Result<(), String> {
        update_book(&self.pool, id, name).await
    }
}

pub async fn update_book(pool: &PgPool, id: i32, name: &str) -> Result<(), String> {
    // WITH name duplicate
    let result = sqlx::query("UPDATE tb_book SET name = $2 WHERE id = $1 ")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(Update{}): {:?}", id, e);
            tracing::error!("{}", &err_msg);
            err_msg
        })?;

    if result.rows_affected() == 0 {
        return Err("target id Not Found".to_string());
    };

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{
        config::database::create_connection_pool,
        domain::book::repository::{get_book::get_book, save::save_book, update::update_book},
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_book_update_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let inserted_id = save_book(&pool, "수정용 가계부", "개인").await.unwrap();
        let target_name = "변경 가계부";

        // Act
        let response = update_book(&pool, inserted_id, target_name).await;
        assert!(response.is_ok());

        // Assert
        let row = get_book(&pool, inserted_id).await.unwrap();

        assert_ne!(row.get_name(), "수정용 가계부");
    }

    #[tokio::test]
    async fn check_book_update_id_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let target_id = 999;
        let target_name = "변경되지 않는 가계부";

        // Act
        let result = update_book(&pool, target_id, &target_name).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_book_update_duplicate_name() {
        // 같은 계정 내 중복 이름이 있을 경우 fail -> user, role 추가 후 작업

        // Arrange

        // Act

        // Assert
    }
}
