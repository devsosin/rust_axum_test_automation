use sqlx::PgPool;

use super::super::dto::request::NewBook;

pub async fn save_book(pool: &PgPool, new_book: &NewBook, type_id: i16) -> Result<bool, String> {
    // 한 유저 내에서는 같은 이름의 가계부 생성 불가
    sqlx::query(
        r#"
        INSERT INTO tb_book (name, type_id) VALUES
        ($1, $2)
        "#,
    )
    .bind(new_book.get_name())
    .bind(type_id)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Error: Inserting new book: {:?}", e);
        let err_message = format!("가계부 생성 중 오류가 발생했습니다.");
        err_message
    })?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sqlx::{postgres::PgPoolOptions, PgPool};

    use crate::domain::book::dto::request::NewBook;
    use super::save_book;
    use super::super::Book;

    // database connect 체크
    #[tokio::test]
    async fn check_database_connectivity() {
        // dotenv::from_filename(".env.test").ok();
        let database_url = "postgres://test:test1234@localhost:5455/test_db"; // std::env::var("DATABASE_URL").expect("set DATABASE_URL env variable");
        
        println!("{:?}", &database_url);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&database_url)
            .await
            .expect("unable to make connections");

        assert_eq!(pool.is_closed(), false);
    }

    async fn create_connection_pool() -> PgPool {
        // dotenv::from_filename(".env.test").ok();
        // let database_url = std::env::var("DATABASE_URL").expect("set DATABASE_URL env variable");
        let database_url = "postgres://test:test1234@localhost:5455/test_db";

        PgPool::connect(&database_url).await.expect("Unable to connect to database")
    }

    // database와 적절하게 통신하는지 (삽입 후 체크 시 데이터 존재여부, 삭제 여부 등)
    #[tokio::test]
    async fn check_create_book() {
        let pool = create_connection_pool().await;

        // given
        let new_book = NewBook::new("새 가계부".to_string(), "개인".to_string());
        let _ = save_book(&pool, &new_book, 1).await;

        // when
        let book = sqlx::query_as::<_, Book>(
            "SELECT id, name, type_id FROM tb_book WHERE id = $1"
        )
        .bind(1i64)
        .fetch_one(&pool)
        .await
        .map_err(|err| err.to_string())
        .unwrap();

        // then
        assert_eq!("새 가계부", book.get_name());
        assert_eq!(1, book.get_type_id());
    }
    
    // query가 옳은지 검증, 다양한 데이터 타입 체크


}