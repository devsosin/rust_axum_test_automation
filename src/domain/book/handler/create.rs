use std::sync::Arc;

use axum::{http::StatusCode, Extension, Json};
use serde_json::{json, Value};

use crate::domain::book::usecase::BookUsecase;

use super::super::dto::request::NewBook;

pub async fn create_book(
    Extension(usecase): Extension<Arc<BookUsecase>>,
    Json(new_book): Json<NewBook>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    tracing::debug!("CALL: Create Book");
    tracing::info!("Create Book : {}", new_book.get_name());

    // book_type 체크 -> &str -> i16
    let book_type: i16 = 1;

    // book_type 전달
    let result = usecase.create_book(&new_book, book_type)
        .await
        .map_err(|err| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"message": err})))
    })?;

    // get created book

    tracing::info!("생성 결과 : {}", result);
    
    Ok(Json(json!({"message": "신규 가계부 생성 완료"})))
}


#[cfg(test)]
mod tests {
    use std::{env, sync::Arc, time::Duration};

    use axum::{body::Body, http::{Method, Request}, routing::post, Extension, Router};
    
    use serde_json::Value;
    use sqlx::{postgres::PgPoolOptions, PgPool};

    use tower::ServiceExt;

    use http_body_util::BodyExt;

    use crate::domain::book::{handler::create::create_book, repository::BookRepositoryImpl, usecase::BookUsecase};

    // 여기서는 값 처리
    #[tokio::test]
    async fn check_database_connectivity() {
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
    async fn check_create_book_route() {
        let pool = create_connection_pool().await;

        let repository = Arc::new(BookRepositoryImpl::new(Arc::new(pool)));
        let usecase = Arc::new(BookUsecase::new(repository));
        
        let app = Router::new()
            .route("/api/v1/book", post(create_book))
            .layer(Extension(usecase));

        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/book")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{
                    "name": "테스트 가계부",
                    "book_type": "개인"
                }"#,
            ))
            .unwrap();

        let response = app
            .oneshot(req)
            .await
            .unwrap();

        let (parts, body) = response.into_parts();
        
        // 이건 메시지 확인용 (정상적으로 파싱되는지 체크)
        // handler에선 input 검증, repsonse 검증
        let body_bytes = body.collect().await.expect("failed to read body").to_bytes();

        let body_str = String::from_utf8(body_bytes.to_vec())
            .expect("Failed to convert body to string");

        let body: Value = serde_json::from_str(&body_str)
            .expect("Failed to parse JSON");

        println!("{:?}", body);

        assert_eq!(parts.status, 200);
    }

}