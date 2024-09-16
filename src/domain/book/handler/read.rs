use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::book::usecase::read::ReadBookUsecase;

pub async fn read_books<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
) -> impl IntoResponse
where
    T: ReadBookUsecase,
{
    match usecase.read_books(user_id).await {
        Ok(result) => (StatusCode::OK, Json(json!(result))).into_response(),
        Err(err) => err.as_ref().into_response(),
    }
}

pub async fn read_book<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Path(book_id): Path<i32>,
) -> impl IntoResponse
where
    T: ReadBookUsecase,
{
    match usecase.read_book(user_id, book_id).await {
        Ok(result) => (StatusCode::OK, Json(json!(result))).into_response(),
        Err(err) => err.as_ref().into_response(),
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

    use crate::{
        domain::book::{
            entity::Book,
            handler::read::{read_book, read_books},
            usecase::read::ReadBookUsecase,
        },
        global::errors::CustomError,
    };

    mock! {
        ReadBookUsecaseImpl {}

        #[async_trait]
        impl ReadBookUsecase for ReadBookUsecaseImpl {
            async fn read_books(&self, user_id:i32) -> Result<Vec<Book>, Box<CustomError>>;
            async fn read_book(&self, user_id: i32, book_id: i32) -> Result<Book, Box<CustomError>>;
        }
    }

    fn _create_app(user_id: i32, usecase: MockReadBookUsecaseImpl) -> Router {
        Router::new()
            .route("/api/v1/book", get(read_books::<MockReadBookUsecaseImpl>))
            .route(
                "/api/v1/book/:book_id",
                get(read_book::<MockReadBookUsecaseImpl>),
            )
            .layer(Extension(Arc::new(usecase)))
            .layer(Extension(user_id))
    }

    fn _create_books_req() -> Request {
        Request::builder()
            .method("GET")
            .uri("/api/v1/book")
            .body(Body::empty())
            .unwrap()
    }
    fn _create_book_req(book_id: i32) -> Request {
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/book/{}", book_id))
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn check_read_books_status() {
        // Arrange
        let user_id = 1;
        let books = vec![
            Book::new("가계부 1".to_string(), 1).id(1),
            Book::new("가계부 2".to_string(), 2).id(2),
            Book::new("가계부 3".to_string(), 1).id(3),
        ];

        let mut mock_usecase = MockReadBookUsecaseImpl::new();
        mock_usecase
            .expect_read_books()
            .with(predicate::eq(user_id))
            .returning(move |_| Ok(books.clone()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_books_req();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_books_body() {
        // Arrange
        let user_id = 1;
        let books = vec![
            Book::new("가계부 1".to_string(), 1).id(1),
            Book::new("가계부 2".to_string(), 2).id(2),
            Book::new("가계부 3".to_string(), 1).id(3),
        ];
        let ret_books = books.clone();

        let mut mock_usecase = MockReadBookUsecaseImpl::new();
        mock_usecase
            .expect_read_books()
            .with(predicate::eq(user_id))
            .returning(move |_| Ok(ret_books.clone()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_books_req();

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

        assert_eq!(body_json[0]["name"], books[0].get_name());
    }

    #[tokio::test]
    async fn check_read_books_not_found() {
        // Arrange
        let user_id = -2;
        let mut mock_usecase = MockReadBookUsecaseImpl::new();

        mock_usecase
            .expect_read_books()
            .with(predicate::eq(user_id))
            .returning(|_| Err(Box::new(CustomError::NotFound("Book".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_books_req();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_read_book_status() {
        // Arrange
        let user_id = 1;
        let book_id = 1;

        let mut mock_usecase = MockReadBookUsecaseImpl::new();
        mock_usecase
            .expect_read_book()
            .with(predicate::eq(user_id), predicate::eq(book_id))
            .returning(|_, i| Ok(Book::new(format!("가계부 {}", i), 1).id(i)));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_book_req(book_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_book_body() {
        // Arrange
        let user_id = 1;
        let book_id = 1;

        let mut mock_usecase = MockReadBookUsecaseImpl::new();
        mock_usecase
            .expect_read_book()
            .with(predicate::eq(user_id), predicate::eq(book_id))
            .returning(|_, i| Ok(Book::new(format!("가계부 {}", i), 1).id(i)));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_book_req(book_id);

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
        assert_eq!(body_json["id"], book_id);
        assert_eq!(body_json["name"], format!("가계부 {}", book_id));
    }

    #[tokio::test]
    async fn check_read_book_not_found() {
        // Arrange
        let user_id = -32;
        let book_id = 5;

        let mut mock_usecase = MockReadBookUsecaseImpl::new();
        mock_usecase
            .expect_read_book()
            .with(predicate::eq(user_id), predicate::eq(book_id))
            .returning(|_, _| Err(Box::new(CustomError::NotFound("Book".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_book_req(book_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }
}
