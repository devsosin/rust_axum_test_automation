use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::domain::book::entity::BookType;

pub struct GetBookTypeRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait GetBookTypeRepo: Send + Sync {
    async fn get_book_types(&self) -> Result<Vec<BookType>, String>;
    async fn get_book_type_by_name(&self, name: &str) -> Result<BookType, String>;
}

impl GetBookTypeRepoImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GetBookTypeRepo for GetBookTypeRepoImpl {
    async fn get_book_types(&self) -> Result<Vec<BookType>, String> {
        get_book_types(&self.pool).await
    }
    async fn get_book_type_by_name(&self, name: &str) -> Result<BookType, String> {
        get_book_type_by_name(&self.pool, name).await
    }
}

pub async fn get_book_types(pool: &PgPool) -> Result<Vec<BookType>, String> {
    let rows: Vec<BookType> = sqlx::query_as::<_, BookType>(
        r#"
    SELECT * FROM tb_book_type
    "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(GetBookTypes): {:?}", e);
        tracing::error!("{:?}", err_msg);

        err_msg
    })?;

    Ok(rows)
}

pub async fn get_book_type_by_name(pool: &PgPool, name: &str) -> Result<BookType, String> {
    let row = sqlx::query_as::<_, BookType>("SELECT * FROM tb_book_type WHERE name = $1")
        .bind(name)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            // row not found error
            let err_msg = format!("Error(GetBookTypeByName): {:?}", e);
            tracing::error!("{:?}", err_msg);
            // err_msg
            "없는 카테고리입니다.".to_string()
        })?;

    Ok(row)
}

#[cfg(test)]
mod tests {

    use sqlx::Acquire;

    use crate::{
        config::database::create_connection_pool,
        domain::book::{entity::BookType, repository::get_book_type::get_book_types},
    };

    use super::get_book_type_by_name;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false);
    }

    #[tokio::test]
    async fn check_get_book_types_success() {
        // Arrange
        let pool = create_connection_pool().await;

        // Act
        let result = get_book_types(&pool).await;
        assert!(result.is_ok());
        let result = result.unwrap();

        // Assert
        assert_eq!(result.len(), 3); // 개인, 커플 기업
    }

    #[tokio::test]
    async fn check_get_book_type_success() {
        // Arrange
        let pool = create_connection_pool().await;
        let mut conn = pool.acquire().await.unwrap();
        let transaction = conn.begin().await.unwrap();

        let name = "개인";

        // Act
        let result = get_book_type_by_name(&pool, name).await;
        // 결과 제대로 받아왔는지 체크
        assert!(result.is_ok());
        let book_type = result.unwrap();
        let type_id = book_type.get_id();

        // Assert
        let row = sqlx::query_as::<_, BookType>("SELECT * FROM tb_book_type WHERE name = $1")
            .bind(name)
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                // Error: RowNotFound
                let err_message = format!("Error: {:?}", e);
                tracing::error!("{:?}", err_message);
                err_message
            })
            .unwrap();

        assert_eq!(row.get_id(), type_id);

        transaction.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn check_book_type_not_found() {
        // Arrange
        let pool = create_connection_pool().await;
        let mut conn = pool.acquire().await.unwrap();
        let transaction = conn.begin().await.unwrap();

        let name = "없는 이름";

        // Act
        let result = get_book_type_by_name(&pool, name).await;

        // Assert: 존재하지 않는 것은 Err(RowNotFound) 반환
        assert!(result.is_err());

        transaction.rollback().await.unwrap();
    }
}
