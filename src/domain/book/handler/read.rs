use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::book::usecase::read::ReadBookUsecase;

pub(crate) async fn read_books<T>(Extension(usecase): Extension<Arc<T>>) -> impl IntoResponse
where
    T: ReadBookUsecase,
{
    match usecase.read_books().await {
        Ok(result) => (StatusCode::OK, Json(json!(result))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": e})),
        )
            .into_response(),
    }
}

pub(crate) async fn read_book<T>(
    Extension(usecase): Extension<Arc<T>>,
    Path(book_id): Path<i32>,
) -> impl IntoResponse
where
    T: ReadBookUsecase,
{
    match usecase.read_book(book_id).await {
        Ok(result) => (StatusCode::OK, Json(json!(result))).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({"message": e}))).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{async_trait, body::Body, extract::Request, routing::get, Extension, Router};

    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::domain::book::{
        entity::Book,
        handler::read::{read_book, read_books},
        usecase::read::ReadBookUsecase,
    };

    mock! {
        ReadBookUsecaseImpl {}

        #[async_trait]
        impl ReadBookUsecase for ReadBookUsecaseImpl {
            async fn read_books(&self) -> Result<Vec<Book>, String>;
            async fn read_book(&self, id: i32) -> Result<Book, String>;
        }
    }

    #[tokio::test]
    async fn check_read_books_status() {
        // Arrange
        let mut mock_usecase = MockReadBookUsecaseImpl::new();

        mock_usecase.expect_read_books().returning(|| {
            Ok(vec![
                Book::new(Some(1), "가계부 1".to_string(), 1),
                Book::new(Some(2), "가계부 2".to_string(), 2),
                Book::new(Some(3), "가계부 3".to_string(), 1),
            ])
        });

        let app = Router::new()
            .route("/api/v1/book", get(read_books::<MockReadBookUsecaseImpl>))
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/book")
            .body(Body::from(()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_books_body() {
        // Arrange
        let mut mock_usecase = MockReadBookUsecaseImpl::new();

        mock_usecase.expect_read_books().returning(|| {
            Ok(vec![
                Book::new(Some(1), "가계부 1".to_string(), 1),
                Book::new(Some(2), "가계부 2".to_string(), 2),
                Book::new(Some(3), "가계부 3".to_string(), 1),
            ])
        });

        let app: Router = Router::new()
            .route("/api/v1/book", get(read_books::<MockReadBookUsecaseImpl>))
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/book")
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
            String::from_utf8(body_bytes.to_vec()).expect("failed to convert body to str");

        let body_json: Value = serde_json::from_str(&body_str).expect("failed to parse JSON");

        assert_eq!(body_json[0]["name"], "가계부 1");
    }

    #[tokio::test]
    async fn check_read_books_not_found() {
        // Arrange
        let mut mock_usecase = MockReadBookUsecaseImpl::new();

        mock_usecase
            .expect_read_books()
            .returning(|| Err("에러가 발생했습니다.".to_string()));

        let app: Router = Router::new()
            .route("/api/v1/book", get(read_books::<MockReadBookUsecaseImpl>))
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/book")
            .body(Body::from(()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 500)
    }

    #[tokio::test]
    async fn check_read_book_status() {
        // Arrange
        let id = 1;

        let mut mock_usecase = MockReadBookUsecaseImpl::new();
        mock_usecase
            .expect_read_book()
            .with(predicate::eq(id))
            .returning(|i| Ok(Book::new(Some(i), format!("가계부 {}", i), 1)));

        let app = Router::new()
            .route(
                "/api/v1/book/:book_id",
                get(read_book::<MockReadBookUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/book/1")
            .body(Body::from(()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_book_body() {
        // Arrange
        let id = 1;

        let mut mock_usecase = MockReadBookUsecaseImpl::new();
        mock_usecase
            .expect_read_book()
            .with(predicate::eq(id))
            .returning(|i| Ok(Book::new(Some(i), format!("가계부 {}", i), 1)));

        let app = Router::new()
            .route(
                "/api/v1/book/:book_id",
                get(read_book::<MockReadBookUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/book/1")
            .body(Body::from(()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        let body = response.into_body();

        let body_bytes = body
            .collect()
            .await
            .expect("failed to read body")
            .to_bytes();

        let body_str =
            String::from_utf8(body_bytes.to_vec()).expect("failed to convert body to string");

        let body_json: Value = serde_json::from_str(&body_str).expect("failed to parse JSON");

        // Assert
        assert_eq!(body_json["id"], id);
        assert_eq!(body_json["name"], format!("가계부 {}", id));
    }

    #[tokio::test]
    async fn check_read_book_not_found() {
        // Arrange
        let id = -32;

        let mut mock_usecase = MockReadBookUsecaseImpl::new();
        mock_usecase
            .expect_read_book()
            .with(predicate::eq(id))
            .returning(|_| Err("가계부를 찾지 못했습니다.".to_string()));

        let app = Router::new()
            .route(
                "/api/v1/book/:book_id",
                get(read_book::<MockReadBookUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/book/-32")
            .body(Body::from(()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }
}
