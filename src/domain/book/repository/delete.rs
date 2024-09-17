use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::global::{constants::DeleteResult, errors::CustomError};

pub struct DeleteBookRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait DeleteBookRepo: Send + Sync {
    async fn delete_book(&self, user_id: i32, book_id: i32) -> Result<(), Box<CustomError>>;
}

impl DeleteBookRepoImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeleteBookRepo for DeleteBookRepoImpl {
    async fn delete_book(&self, user_id: i32, book_id: i32) -> Result<(), Box<CustomError>> {
        delete_book(&self.pool, user_id, book_id).await
    }
}

async fn delete_book(pool: &PgPool, user_id: i32, book_id: i32) -> Result<(), Box<CustomError>> {
    let result = sqlx::query_as::<_, DeleteResult>(
        "
        WITH BookExists AS (
            SELECT id
            FROM tb_book
            WHERE id = $2
        ),
        AuthorityCheck AS (
            SELECT EXISTS (
                SELECT 1 
                FROM BookExists AS be
                JOIN tb_user_book_role AS br ON br.book_id = be.id
                WHERE br.user_id = $1 AND role = 'owner'
            ) AS is_authorized
        ),
        DeleteRole AS (
            DELETE FROM tb_user_book_role
            WHERE user_id = $1 AND book_id = $2
                AND (SELECT is_authorized FROM AuthorityCheck) = true
        ),
        DeleteBook AS (
            DELETE FROM tb_book 
            WHERE id = $2
                AND (SELECT is_authorized FROM AuthorityCheck) = true
            RETURNING id
        )
        SELECT 
            EXISTS (SELECT 1 FROM BookExists) AS is_exist,
            (SELECT is_authorized FROM AuthorityCheck) AS is_authorized,
            (SELECT COUNT(*) FROM DeleteBook) AS delete_count;
        ",
    )
    .bind(user_id)
    .bind(book_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(DeleteBook {}): {:?}", book_id, &e);
        tracing::error!("{}", err_msg);

        let err = match e {
            // database error
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            // internal server error
            _ => CustomError::Unexpected(e.into()),
        };
        Box::new(err)
    })?;

    if !result.get_exist() {
        return Err(Box::new(CustomError::NotFound("Book".to_string())));
    } else if !result.get_authorized() {
        return Err(Box::new(CustomError::Unauthorized("BookRole".to_string())));
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{
        config::database::create_connection_pool,
        domain::book::{
            entity::Book,
            repository::{delete::delete_book, get_book::get_book, save::save_book},
        },
        global::errors::CustomError,
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
        let book = Book::new("삭제용 가계부".to_string(), 1);
        let user_id = 1;

        let target_id = save_book(&pool, book, user_id).await.unwrap();

        // Act
        let result = delete_book(&pool, user_id, target_id).await;
        assert!(result.map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let row = get_book(&pool, user_id, target_id).await;
        assert!(row.is_err())
    }

    #[tokio::test]
    async fn check_book_not_found() {
        // Arrange
        let pool = create_connection_pool().await;
        let user_id = 1;
        let book_id = -32;

        // Act
        let result = delete_book(&pool, user_id, book_id).await;

        // Assert
        assert!(result.as_ref().is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_delete_book_no_role() {
        // Arrange
        let pool = create_connection_pool().await;
        // ref) init.sql
        let user_id = 2;
        let book_id = 1;

        // Act
        let result = delete_book(&pool, user_id, book_id).await;

        // Assert
        assert!(result.as_ref().is_err());
        println!("{:?}", result.as_ref().err());
        let err_type = match *result.err().unwrap() {
            CustomError::Unauthorized(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}
