use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::book::usecase::read_type::ReadBookTypeUsecase;

pub(crate) async fn read_book_types<T>(Extension(usecase): Extension<Arc<T>>) -> impl IntoResponse
where
    T: ReadBookTypeUsecase,
{
    match usecase.read_book_types().await {
        Ok(result) => (StatusCode::OK, Json(json!(result))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": e})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{async_trait, body::Body, extract::Request, routing::get, Extension, Router};
    use http_body_util::BodyExt;
    use mockall::mock;
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::domain::book::{
        entity::BookType, handler::read_type::read_book_types,
        usecase::read_type::ReadBookTypeUsecase,
    };

    mock! {
        ReadBookTypeUsecaseImpl {}

        #[async_trait]
        impl ReadBookTypeUsecase for ReadBookTypeUsecaseImpl {
            async fn read_book_types(&self) -> Result<Vec<BookType>, String>;
        }
    }

    #[tokio::test]
    async fn check_read_book_types_status() {
        // Arrange
        let mut mock_usecase = MockReadBookTypeUsecaseImpl::new();
        mock_usecase.expect_read_book_types().returning(|| {
            Ok(vec![
                BookType::new(1, "개인".to_string()),
                BookType::new(2, "커플".to_string()),
                BookType::new(3, "기업".to_string()),
            ])
        });

        let app = Router::new()
            .route(
                "/api/v1/book/type",
                get(read_book_types::<MockReadBookTypeUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/book/type")
            .body(Body::from(()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_book_types_body() {
        // Arrange
        let mut mock_usecase = MockReadBookTypeUsecaseImpl::new();
        mock_usecase.expect_read_book_types().returning(|| {
            Ok(vec![
                BookType::new(1, "개인".to_string()),
                BookType::new(2, "커플".to_string()),
                BookType::new(3, "기업".to_string()),
            ])
        });

        let app = Router::new()
            .route(
                "/api/v1/book/type",
                get(read_book_types::<MockReadBookTypeUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));
        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/book/type")
            .body(Body::from(()))
            .unwrap();

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

        assert_eq!(body_json[0]["id"], 1);
        assert_eq!(body_json[1]["name"], "커플");
        assert_eq!(body_json[2]["id"], 3);
    }
}
