use std::sync::Arc;

use axum::async_trait;
use sqlx::{PgPool, Row};

use crate::{domain::book::entity::Book, global::errors::CustomError};

pub(crate) struct SaveBookRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub(crate) trait SaveBookRepo: Send + Sync {
    async fn save_book(&self, book: Book) -> Result<i32, Arc<CustomError>>;
}

impl SaveBookRepoImpl {
    pub(crate) fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SaveBookRepo for SaveBookRepoImpl {
    async fn save_book(&self, book: Book) -> Result<i32, Arc<CustomError>> {
        save_book(&self.pool, book).await
    }
}

pub(crate) async fn save_book(pool: &PgPool, book: Book) -> Result<i32, Arc<CustomError>> {
    // 한 유저 내에서는 같은 이름의 가계부 생성 불가
    // type_id sub_query
    let row = sqlx::query(
        r#"
        INSERT INTO tb_book (name, type_id)
        VALUES ($1, $2)
        RETURNING id;
        "#,
    )
    .bind(book.get_name())
    .bind(book.get_type_id())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let err_msg = format!("Error(SaveBook): {:?}", e);
        tracing::error!("{}", err_msg);

        let err = match e {
            sqlx::Error::Database(_) => CustomError::DatabaseError(e),
            _ => CustomError::Unexpected(e.into()),
        };
        Arc::new(err)
    })?;

    let id: i32 = row.get("id");

    Ok(id)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sqlx::postgres::PgPoolOptions;

    use crate::config::database::create_connection_pool;
    use crate::domain::book::entity::Book;
    use crate::domain::book::repository::save::save_book;

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

        let book = Book::new(None, "새 가계부".to_string(), 1);

        // Act: 메서드 호출을 통한 DB에 데이터 삽입
        let result = save_book(&pool, book.clone()).await;
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
    async fn check_create_book_fail_with_type() {
        // Arrange
        let pool = create_connection_pool().await;
        let book = Book::new(None, "새 가계부".to_string(), -3);

        // Act
        let result = save_book(&pool, book).await;

        // Assert
        assert!(result.map_err(|e| println!("{:?}", e)).is_err());
    }

    // 중복데이터 삽입 테스트케이스
    async fn check_create_book_failure() {
        // user정보 같이 삽입 -> role 추가
        // user-role table
    }
}
