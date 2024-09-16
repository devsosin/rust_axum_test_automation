use std::sync::Arc;

use axum::async_trait;
use sqlx::PgPool;

use crate::{domain::book::entity::Book, global::errors::CustomError};

pub struct SaveBookRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait SaveBookRepo: Send + Sync {
    async fn save_book(&self, book: Book, user_id: i32) -> Result<i32, Box<CustomError>>;
}

impl SaveBookRepoImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SaveBookRepo for SaveBookRepoImpl {
    async fn save_book(&self, book: Book, user_id: i32) -> Result<i32, Box<CustomError>> {
        save_book(&self.pool, book, user_id).await
    }
}

#[derive(Debug, sqlx::FromRow)]
struct InsertResult {
    id: Option<i32>,
    duplicated_check: bool,
}

impl InsertResult {
    pub fn get_id(&self) -> Option<i32> {
        self.id
    }
    pub fn get_duplicated(&self) -> bool {
        self.duplicated_check
    }
}

pub async fn save_book(pool: &PgPool, book: Book, user_id: i32) -> Result<i32, Box<CustomError>> {
    let row = sqlx::query_as::<_, InsertResult>(
        r#"
        WITH DuplicateCheck AS (
            SELECT EXISTS (
                SELECT 1
                FROM tb_book AS b
                JOIN tb_user_book_role AS br ON br.book_id = b.id
                WHERE br.user_id = $1 AND b.name = $2
            ) AS is_duplicate
        ),
        InsertBook AS (
            INSERT INTO tb_book (name, type_id)
                SELECT $2, $3
                    FROM DuplicateCheck
                    WHERE is_duplicate = false
            RETURNING id
        ),
        InsertRole AS (
            INSERT INTO tb_user_book_role (user_id, book_id, role)
                SELECT $1, id, 'owner'
                    FROM InsertBook
                    WHERE id IS NOT NULL
        )
        SELECT 
            (SELECT id FROM InsertBook) AS id,
            (SELECT is_duplicate FROM DuplicateCheck) AS duplicated_check;
    "#,
    )
    .bind(user_id)
    .bind(book.get_name())
    .bind(book.get_type_id())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(SaveBook): {:?}", e);
        tracing::error!("{}", err_msg);

        let err = match e {
            // 무결성 제약은 여기에 속함 DatabaseError
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Box::new(err)
    })?;

    if row.get_duplicated() {
        return Err(Box::new(CustomError::Duplicated("Book".to_string())));
    }

    let id: i32 = row.get_id().unwrap();
    Ok(id)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sqlx::postgres::PgPoolOptions;

    use crate::config::database::create_connection_pool;
    use crate::domain::book::entity::Book;
    use crate::domain::book::repository::save::save_book;
    use crate::global::errors::CustomError;

    #[tokio::test]
    async fn check_database_connectivity() {
        // Arrange: 테스트 데이터베이스 설정
        let database_url = std::env::var("DATABASE_URL").expect("set DATABASE_URL env variable");

        // Act: pool 생성
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&database_url)
            .await
            .expect("unable to make connections");

        // Assert: 연결 되어있는지 확인
        assert_eq!(pool.is_closed(), false);
    }

    #[tokio::test]
    async fn check_create_book_success() {
        // Arange: 테스트 데이터베이스 설정, 데이터 준비
        let pool = create_connection_pool().await;
        let user_id = 1; // test user
        let book = Book::new("새 가계부".to_string(), 1);

        // Act: 메서드 호출을 통한 DB에 데이터 삽입
        let result = save_book(&pool, book.clone(), user_id).await;
        let inserted_id = result.map_err(|e| println!("{:?}", e)).unwrap();
        // assert!(result.is_ok()); // 삽입 성공 여부 확인

        // Assert: DB에서 직접 조회하여 검증
        let row = sqlx::query_as::<_, Book>("SELECT id, name, type_id FROM tb_book WHERE id = $1")
            .bind(inserted_id)
            .fetch_one(&pool)
            .await
            .map_err(|e| println!("{:?}", e))
            .unwrap();

        // 삽입된 데이터의 필드값 확인
        assert_eq!(book.get_name(), row.get_name());
    }

    #[tokio::test]
    async fn check_no_type() {
        // Arrange
        let pool = create_connection_pool().await;
        let user_id = 1;
        let book = Book::new("새 가계부22".to_string(), -3);

        // Act
        let result = save_book(&pool, book, user_id).await;

        // Assert
        assert!(result.as_ref().map_err(|e| println!("{:?}", e)).is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::DatabaseError(_) => true,
            _ => false,
        };
        assert!(err_type)
    }

    // 중복데이터 삽입 테스트케이스
    #[tokio::test]
    async fn check_duplicate() {
        let pool = create_connection_pool().await;
        let user_id = 1;
        let book = Book::new("중복 가계부 이름".to_string(), 1);
        let _ = save_book(&pool, book, user_id).await.unwrap();

        let duplicate_book = Book::new("중복 가계부 이름".to_string(), 1);

        // Act
        let result = save_book(&pool, duplicate_book, user_id).await;

        // Assert
        assert!(result.as_ref().is_err());
        let err_type = match *result.err().unwrap() {
            CustomError::Duplicated(_) => true,
            _ => false,
        };
        assert!(err_type)
    }
}
