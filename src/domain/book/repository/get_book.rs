use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::book::entity::Book, global::errors::CustomError};

pub struct GetBookRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait GetBookRepo: Send + Sync {
    async fn get_books(&self) -> Result<Vec<Book>, Arc<CustomError>>;
    async fn get_book(&self, id: i32) -> Result<Book, Arc<CustomError>>;
}

impl GetBookRepoImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GetBookRepo for GetBookRepoImpl {
    async fn get_books(&self) -> Result<Vec<Book>, Arc<CustomError>> {
        get_books(&self.pool).await
    }

    async fn get_book(&self, id: i32) -> Result<Book, Arc<CustomError>> {
        get_book(&self.pool, id).await
    }
}

async fn get_books(pool: &PgPool) -> Result<Vec<Book>, Arc<CustomError>> {
    let books: Vec<Book> = sqlx::query_as::<_, Book>(
        r#"
        SELECT * FROM tb_book
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(GetBooks): {:?}", &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Arc::new(err)
    })?;

    Ok(books)
}

pub async fn get_book(pool: &PgPool, id: i32) -> Result<Book, Arc<CustomError>> {
    let book = sqlx::query_as::<_, Book>("SELECT * FROM tb_book WHERE id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            let err_msg = format!("Error(GetBook {}): {:?}", id, &e);
            tracing::error!("{}", err_msg);

            let err = match e {
                sqlx::Error::Database(_) => CustomError::DatabaseError(e),
                sqlx::Error::RowNotFound => CustomError::NotFound("Book".to_string()),
                _ => CustomError::Unexpected(e.into()),
            };
            Arc::new(err)
        })?;

    Ok(book)
}

#[cfg(test)]
mod tests {
    use crate::{
        config::database::create_connection_pool,
        domain::book::{
            entity::Book,
            repository::get_book::{get_book, get_books},
        },
    };

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange, Act
        let pool = create_connection_pool().await;

        // Assert
        assert_eq!(pool.is_closed(), false);
    }

    // test snippet
    #[tokio::test]
    async fn check_get_books_success() {
        // Arrange
        let pool = create_connection_pool().await;

        // Act
        let result = get_books(&pool).await;
        assert!(result.is_ok());

        // Assert (나중에 user_id로 -> book_role 체크)
        let result = result.unwrap();
        let rows = sqlx::query("SELECT * FROM tb_book")
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(rows.len(), result.len())
    }

    #[tokio::test]
    async fn check_get_book_success() {
        // Arange
        let pool = create_connection_pool().await;
        let id: i32 = 1;

        // Act
        let result = get_book(&pool, id).await;
        assert!(result.is_ok());
        let result = result.unwrap();

        // Assert
        let row = sqlx::query_as::<_, Book>("SELECT * FROM tb_book WHERE ID = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        assert_eq!(result.get_name(), row.get_name());
    }

    #[tokio::test]
    async fn check_get_book_failure() {
        // Arange
        let pool = create_connection_pool().await;
        let id: i32 = -32;

        // Act
        let result = get_book(&pool, id).await;

        // Assert -> row not found error
        assert!(result.is_err());
    }
}
