use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{
    domain::book::entity::BookUpdate,
    global::{constants::UpdateResult, errors::CustomError},
};

pub struct UpdateBookRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait UpdateBookRepo: Send + Sync {
    async fn update_book(&self, book_update: BookUpdate) -> Result<(), Box<CustomError>>;
}

impl UpdateBookRepoImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UpdateBookRepo for UpdateBookRepoImpl {
    async fn update_book(&self, book_update: BookUpdate) -> Result<(), Box<CustomError>> {
        update_book(&self.pool, book_update).await
    }
}

pub async fn update_book(pool: &PgPool, book_update: BookUpdate) -> Result<(), Box<CustomError>> {
    let result = sqlx::query_as::<_, UpdateResult>(
        r"
        WITH BookExists AS (
            SELECT id, name
            FROM tb_book
            WHERE id = $2
        ),
        AuthorityCheck AS (
            SELECT be.id AS book_id, be.name, br.user_id
            FROM BookExists AS be
            JOIN tb_user_book_role AS br ON br.book_id = be.id
            WHERE br.user_id = $1 AND br.role != 'viewer'
        ),
        -- 아 여기서 duplicate check하려면 book_exist랑 별개로 체크해야되네
        DuplicateCheck AS (
            SELECT EXISTS (
                SELECT 1
                FROM tb_user_book_role AS br
                JOIN tb_book AS b ON b.id = br.book_id
                WHERE b.name = $3 AND br.book_id != $2
            ) AS is_duplicate
        ),
        UpdateBook AS (
            UPDATE tb_book SET name = $3
            WHERE id = $2 
                AND (SELECT is_duplicate FROM DuplicateCheck) = false
                AND EXISTS (SELECT book_id FROM AuthorityCheck) = true
            RETURNING id
        )
        SELECT 
            EXISTS (SELECT 1 FROM BookExists) AS exists_check,
            EXISTS (SELECT 1 FROM AuthorityCheck) AS authorized_check,
            (SELECT is_duplicate FROM DuplicateCheck) AS duplicated_check,
            (SELECT COUNT(*) FROM UpdateBook) AS update_count;
    ",
    )
    .bind(book_update.get_user_id())
    .bind(book_update.get_book_id())
    .bind(book_update.get_name())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(Update {}): {:?}", book_update.get_book_id(), e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Box::new(err)
    })?;

    if !result.get_exist() {
        return Err(Box::new(CustomError::NotFound("Book".to_string())));
    } else if !result.get_authorized() {
        return Err(Box::new(CustomError::Unauthorized("BookRole".to_string())));
    } else if result.get_duplicated() {
        return Err(Box::new(CustomError::Duplicated("Book".to_string())));
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{
        config::database::create_connection_pool,
        domain::book::{
            entity::{Book, BookUpdate},
            repository::{get_book::get_book, save::save_book, update::update_book},
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
    async fn check_book_update_success() {
        // Arrange
        let pool = create_connection_pool().await;
        let user_id = 1;
        let book = Book::new("수정용 가계부".to_string(), 1);

        let inserted_id = save_book(&pool, book.clone(), user_id).await.unwrap();
        let target_name = "변경 가계부";

        let book_update = BookUpdate::new(user_id, inserted_id, target_name.to_string());

        // Act
        let result = update_book(&pool, book_update).await;
        assert!(result.map_err(|e| println!("{:?}", e)).is_ok());

        // Assert
        let row = get_book(&pool, user_id, inserted_id).await.unwrap();

        assert_eq!(row.get_name(), target_name);
    }

    #[tokio::test]
    async fn check_book_update_id_not_found() {
        // Arrange
        let pool = create_connection_pool().await;

        let user_id = 1;
        let target_id = -32;
        let target_name = "변경되지 않는 가계부";

        let book_update = BookUpdate::new(user_id, target_id, target_name.to_string());

        // Act
        let result = update_book(&pool, book_update).await;

        // Assert
        assert!(result.is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::NotFound(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_book_update_duplicate_name() {
        // Arrange
        let pool = create_connection_pool().await;
        let user_id = 1;
        let target_name = "중복체크 가계부";
        let book = Book::new(target_name.to_string(), 1);
        let _ = save_book(&pool, book.clone(), user_id).await.unwrap();

        let book = Book::new("수정시도용 가계부".to_string(), 1);
        let inserted_id = save_book(&pool, book.clone(), user_id).await.unwrap();
        let book_update = BookUpdate::new(user_id, inserted_id, target_name.to_string());

        // Act
        let result = update_book(&pool, book_update).await;

        // Assert
        // 오류가 나야되는데 왜 안나지?
        println!("{:?}", result);
        assert!(result.as_ref().is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::Duplicated(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    #[tokio::test]
    async fn check_user_has_no_role() {
        // Arrange
        // ref) init.sql
        let pool = create_connection_pool().await;
        let user_id = 2;
        let book_id = 1;
        let target_name = "권한 부족";
        let book_update = BookUpdate::new(user_id, book_id, target_name.to_string());

        // Act
        let result = update_book(&pool, book_update).await;

        // Assert
        assert!(result.as_ref().is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::Unauthorized(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}
