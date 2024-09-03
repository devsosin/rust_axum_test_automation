use axum::{http::StatusCode, Extension, Json};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::domain::book::usecase;

use super::dto::request::NewBook;

pub async fn create_book(
    Extension(pool): Extension<PgPool>,
    Json(new_book): Json<NewBook>,
) -> Result<Json<Value>, (StatusCode, String)> {
    tracing::debug!("CALL: Create Book");
    tracing::info!("Create Book : {}", new_book.get_name());

    // book_type 체크 -> &str -> i16

    // book_type 전달
    let result = usecase::create_book(&pool, &new_book, 1)
        .await
        .map_err(|err| {
            (StatusCode::INTERNAL_SERVER_ERROR, err)
    })?;

    // get created book

    tracing::info!("생성 결과 : {}", result);
    Ok(Json(json!({"message": "신규 가계부 생성 완료"})))
}


#[cfg(test)]
mod test {
    use std::{env, time::Duration};

    use axum::{Extension, Json};
    use dotenv::dotenv;
    use sqlx::{postgres::PgPoolOptions, Acquire, PgPool};

    use crate::domain::book::{dto::request::NewBook, handler::create_book};

    #[tokio::test]
    async fn check_database_connectivity() {
        dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("set DATABASE_URL env variable");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(&database_url)
            .await
            .expect("unable to make connections");

        assert_eq!(pool.is_closed(), false);
    }

    async fn create_connection_pool() -> PgPool {
        let database_url = env::var("DATABASE_URL").expect("set DATABASE_URL env variable");
        PgPool::connect(&database_url).await.expect("Unable to connect to database")
    }


    #[tokio::test]
    async fn check_create_book() {
        dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("set DATABASE_URL env variable");

        let pool = create_connection_pool().await;

        // 흠.. 한번 테스트하고 되돌리는 방법 확인해봐야겠는데
        // let mut transaction = pool.begin().await.expect("Failed to create transaction");

        // 예제 입력 생성
        let new_book = NewBook::new("Test Book", "개인");

        let request_body = Json(new_book);

        // create_book 함수 호출
        // let response = create_book(Extension(transaction.acquire().await.unwrap()), request_body).await;

        // 결과 확인
        // assert!(response.is_ok());

        // transaction.rollback().await.expect("Failed to rollback transaction");
    }

}