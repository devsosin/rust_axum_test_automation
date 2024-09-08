use std::sync::Arc;

use axum::async_trait;
use sqlx::{PgPool, Row};

pub struct SaveBookRepoImpl {
    pool: Arc<PgPool>,
}

#[async_trait]
pub trait SaveBookRepo: Send + Sync {
    async fn save_book(&self, name: &str, book_type: &str) -> Result<i32, String>;
}

impl SaveBookRepoImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SaveBookRepo for SaveBookRepoImpl {
    async fn save_book(&self, name: &str, book_type: &str) -> Result<i32, String> {
        save_book(&self.pool, name, book_type).await
    }
}

pub async fn save_book(pool: &PgPool, name: &str, book_type: &str) -> Result<i32, String> {
    // 한 유저 내에서는 같은 이름의 가계부 생성 불가
    // type_id sub_query
    let row = sqlx::query(
        r#"
        WITH type_check AS (
            SELECT id
            FROM tb_book_type
            WHERE name = $2
        )
        INSERT INTO tb_book (name, type_id)
        SELECT $1, id FROM type_check
        RETURNING id;
        "#,
    )
    .bind(name)
    .bind(book_type)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Error: Inserting new book: {:?}", e);
        let err_message = format!("가계부 생성 중 오류가 발생했습니다. {:?}", e);
        err_message
    })?;

    let id: i32 = row.get("id");

    Ok(id)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sqlx::postgres::PgPoolOptions;
    use sqlx::Acquire;

    use crate::config::database::create_connection_pool;
    use crate::domain::book::dto::request::NewBook;
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
        let mut conn = pool.acquire().await.unwrap();
        let transaction = conn.begin().await.unwrap();

        let name = "새 가계부";
        let book_type = "개인";

        // Act: 메서드 호출을 통한 DB에 데이터 삽입
        let result = save_book(&pool, name, book_type).await;
        let inserted_id = result.map_err(|e| println!("{}", e)).unwrap();
        // assert!(result.is_ok()); // 삽입 성공 여부 확인

        // Assert: DB에서 직접 조회하여 검증
        // let inserted_id = result.unwrap();
        let row = sqlx::query_as::<_, Book>("SELECT id, name, type_id FROM tb_book WHERE id = $1")
            .bind(inserted_id)
            .fetch_one(&pool)
            .await
            .map_err(|err| err.to_string())
            .unwrap();

        // 삽입된 데이터의 필드값 확인
        assert_eq!(name, row.get_name());

        // 상태변화 방지를 위한 롤백
        transaction.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn check_create_book_fail_with_type() {
        // Arrange
        let pool = create_connection_pool().await;
        let mut conn = pool.acquire().await.unwrap();
        let transaction = conn.begin().await.unwrap();
        let new_book = NewBook::new("새 가계부".to_string(), "없는 타입".to_string());
        // Act
        let result = save_book(&pool, new_book.get_name(), new_book.get_book_type()).await;

        // Assert

        assert!(result.map_err(|e| println!("{:?}", e)).is_err());

        transaction.rollback().await.unwrap();
    }

    // 중복데이터 삽입 테스트케이스
    async fn check_create_book_failure() {
        // user정보 같이 삽입 -> role 추가
        // user-role table
    }
}
