use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use serde_json::json;

use crate::domain::book::{
    dto::request::NewBook,
    usecase::{BookUsecase, BookUsecaseImpl},
};

pub async fn create_book(
    Extension(usecase): Extension<Arc<BookUsecaseImpl>>,
    Json(new_book): Json<NewBook>,
) -> impl IntoResponse {
    tracing::debug!("CALL: Create Book");
    tracing::info!("Create Book : {}", new_book.get_name());

    // book_type 체크 -> &str -> i16
    let type_id: i16 = 1;

    // book_type 전달
    match usecase.create_book(&new_book, type_id).await {
        Ok(id) => {
            tracing::info!("Created: {}", id);
            (
                StatusCode::CREATED,
                Json(json!({"message": "신규 가계부 생성 완료", "book_id": id})),
            )
                .into_response()
        }
        Err(err) => {
            tracing::error!("Error: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"message": err})).into_response(),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        http::{Method, Request},
        routing::post,
        Extension, Router,
    };

    use serde_json::Value;

    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::config;
    use crate::domain::book::{
        handler::create::create_book, repository::BookRepositoryImpl, usecase::BookUsecaseImpl,
    };

    #[tokio::test]
    async fn check_create_book_route() {
        // usecase mockall
        let pool = config::database::create_connection_pool().await;

        let repository = Arc::new(BookRepositoryImpl::new(Arc::new(pool)));
        let usecase = Arc::new(BookUsecaseImpl::new(repository));

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

        let response = app.oneshot(req).await.unwrap();

        let (parts, body) = response.into_parts();

        // 이건 메시지 확인용 (정상적으로 파싱되는지 체크)
        // handler에선 input 검증, repsonse 검증
        let body_bytes = body
            .collect()
            .await
            .expect("failed to read body")
            .to_bytes();

        let body_str =
            String::from_utf8(body_bytes.to_vec()).expect("Failed to convert body to string");

        let body: Value = serde_json::from_str(&body_str).expect("Failed to parse JSON");

        println!("{:?}", body);

        assert_eq!(parts.status, 201);
    }
}
