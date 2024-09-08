use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

pub(crate) struct DeleteBookRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub(crate) trait DeleteBookRepo: Send + Sync {
    async fn delete_book(&self, id: i32) -> Result<(), String>;
}

impl DeleteBookRepoImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeleteBookRepo for DeleteBookRepoImpl {
    async fn delete_book(&self, id: i32) -> Result<(), String> {
        delete_book(&self.pool, id).await
    }
}

async fn delete_book(pool: &PgPool, id: i32) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM tb_book WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            // database error or internal server error
            let err_msg = format!("Delete(Book{}): {:?}", id, e);
            tracing::error!(err_msg);
            err_msg
        })?;

    if result.rows_affected() == 0 {
        // not found error
        return Err("target id not found".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{
        config::database::create_connection_pool,
        domain::book::repository::{delete::delete_book, get_book::get_book, save::save_book},
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false)
    }

    #[tokio::test]
    async fn check_delete_book_success() {
        // Arrange
        let pool = create_connection_pool().await;

        let target_id = save_book(&pool, "삭제용 가계부", "개인").await.unwrap();

        // Act
        let result = delete_book(&pool, target_id).await;
        assert!(result
            .map_err(|e| println!("delete book error: {:?}", e))
            .is_ok());

        // Assert
        let row = get_book(&pool, target_id).await;
        assert!(row.is_err())
    }

    #[tokio::test]
    async fn check_delete_book_id_not_found() {
        // Arrange
        let pool = create_connection_pool().await;
        let id = -32;

        // Act
        let result = delete_book(&pool, id).await;

        // Assert
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn check_delete_book_no_role() {
        // 권한 부족
        todo!()

        // Arrange

        // Act

        // Assert
    }
}
