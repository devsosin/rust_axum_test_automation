use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use serde_json::json;

use crate::domain::book::{dto::request::NewBook, usecase::CreateBookUsecase};

pub async fn create_book<T>(
    Extension(usecase): Extension<Arc<T>>,
    Json(new_book): Json<NewBook>,
) -> impl IntoResponse
where
    T: CreateBookUsecase,
{
    tracing::debug!("CALL: Create Book");
    tracing::info!("Create Book : {}", new_book.get_name());

    match usecase.create_book(&new_book).await {
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
        async_trait,
        body::Body,
        http::{Method, Request},
        routing::post,
        Extension, Router,
    };

    use mockall::{mock, predicate};
    use serde_json::{to_string, Value};

    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::domain::book::{
        dto::request::NewBook, handler::create::create_book, usecase::CreateBookUsecase,
    };

    mock! {
        CreateBookUsecaseImpl {}

        #[async_trait]
        impl CreateBookUsecase for CreateBookUsecaseImpl {
            async fn create_book(&self, new_book: &NewBook) -> Result<i32, String>;
        }
    }

    fn create_req(body: String) -> Request<Body> {
        let req = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/book")
            .header("content-type", "application/json;charset=utf-8")
            .body(Body::from(body))
            .unwrap();
        req
    }

    fn create_app(new_book: &NewBook, ret: Result<i32, String>) -> Router {
        let mut usecase = MockCreateBookUsecaseImpl::new();
        usecase
            .expect_create_book()
            .with(predicate::eq(new_book.clone()))
            .returning(move |_| ret.clone());

        let app = Router::new()
            .route(
                "/api/v1/book",
                post(create_book::<MockCreateBookUsecaseImpl>),
            )
            .layer(Extension(Arc::new(usecase)));

        app
    }

    #[tokio::test]
    async fn check_create_book_status() {
        // Arrange
        let new_book = NewBook::new("테스트 가계부".to_string(), "개인".to_string());
        let json_body = to_string(&new_book).unwrap();

        let app = create_app(&new_book, Ok(1));
        let req = create_req(json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 201);
    }

    #[tokio::test]
    async fn check_create_book_body() {
        // Arrange
        let new_book = NewBook::new("테스트 가계부".to_string(), "개인".to_string());
        let json_body = to_string(&new_book).unwrap();

        let app = create_app(&new_book, Ok(1));
        let req = create_req(json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        let body = response.into_body();

        let body_bytes = body
            .collect()
            .await
            .expect("failed to read body")
            .to_bytes();

        let body_str =
            String::from_utf8(body_bytes.to_vec()).expect("failed to convert body to string");

        let body_json: Value = serde_json::from_str(&body_str).expect("failed to parse JSON");

        assert_eq!(body_json["book_id"], 1);
    }

    #[tokio::test]
    async fn check_create_book_failure_no_book_type() {
        let new_book = NewBook::new("테스트 가계부".to_string(), "없는 카테".to_string());
        let json_body = to_string(&new_book).unwrap();

        let app = create_app(&new_book, Err("없는 카테고리입니다.".to_string()));
        let req = create_req(json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_create_book_failure_duplicate() {
        let new_book = NewBook::new("테스트 가계부".to_string(), "개인".to_string());
        let json_body = to_string(&new_book).unwrap();

        let app = create_app(&new_book, Err("중복된 가계부입니다.".to_string()));
        let req = create_req(json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 421)
    }
}
