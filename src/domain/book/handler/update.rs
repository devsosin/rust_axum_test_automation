use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::book::{dto::request::EditBook, usecase::update::UpdateBookUsecase};

pub async fn update_book<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Path(book_id): Path<i32>,
    Json(edit_book): Json<EditBook>,
) -> impl IntoResponse
where
    T: UpdateBookUsecase,
{
    match usecase.update_book(user_id, book_id, edit_book).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "성공"}))).into_response(),
        Err(err) => err.as_ref().into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{async_trait, body::Body, extract::Request, routing::patch, Extension, Router};
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::{to_string, Value};
    use tower::ServiceExt;

    use crate::{
        domain::book::{
            dto::request::EditBook, handler::update::update_book,
            usecase::update::UpdateBookUsecase,
        },
        global::errors::CustomError,
    };

    mock! {
        UpdateBookUsecaseImpl {}

        #[async_trait]
        impl UpdateBookUsecase for UpdateBookUsecaseImpl {
            async fn update_book(&self,
                user_id: i32,
                book_id: i32,
                edit_book: EditBook) -> Result<(), Box<CustomError>>;
        }
    }

    fn _get_request(id: i32, json_body: String) -> Request<Body> {
        Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/book/{}", id))
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap()
    }

    fn _get_router(user_id: i32, mock_usecase: MockUpdateBookUsecaseImpl) -> Router {
        let app = Router::new()
            .route(
                "/api/v1/book/:book_id",
                patch(update_book::<MockUpdateBookUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id));

        app
    }

    #[tokio::test]
    async fn check_update_book_status() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let target_book = EditBook::new("수정된 가계부".to_string());
        let json_body = to_string(&target_book).unwrap();

        let mut mock_usecase = MockUpdateBookUsecaseImpl::new();
        mock_usecase
            .expect_update_book()
            .with(
                predicate::eq(user_id),
                predicate::eq(book_id),
                predicate::eq(target_book),
            )
            .returning(|_, _, _| Ok(()));

        let app = _get_router(user_id, mock_usecase);
        let req = _get_request(book_id, json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_update_book_body() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let target_book = EditBook::new("수정된 가계부".to_string());
        let json_body = to_string(&target_book).unwrap();

        let mut mock_usecase = MockUpdateBookUsecaseImpl::new();
        mock_usecase
            .expect_update_book()
            .with(
                predicate::eq(user_id),
                predicate::eq(book_id),
                predicate::eq(target_book),
            )
            .returning(|_, _, _| Ok(()));
        let app = _get_router(user_id, mock_usecase);
        let req = _get_request(book_id, json_body);

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

        let body_json: Value = serde_json::from_str(&body_str).unwrap();

        // Assert
        assert_eq!(body_json["message"], "성공".to_string())
    }

    #[tokio::test]
    async fn check_not_found() {
        // Arrange
        let user_id = 1;
        let no_id = -32;
        let target_book = EditBook::new("없는 가계부".to_string());
        let json_body = to_string(&target_book).unwrap();

        let mut mock_usecase = MockUpdateBookUsecaseImpl::new();
        mock_usecase
            .expect_update_book()
            .with(
                predicate::eq(user_id),
                predicate::eq(no_id),
                predicate::eq(target_book),
            )
            .returning(|_, _, _| Err(Box::new(CustomError::NotFound("Book".to_string()))));

        let app = _get_router(user_id, mock_usecase);
        let req = _get_request(no_id, json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_duplicated() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let target_book = EditBook::new("중복 이름".to_string());
        let json_body = to_string(&target_book).unwrap();

        let mut mock_usecase = MockUpdateBookUsecaseImpl::new();
        mock_usecase
            .expect_update_book()
            .with(
                predicate::eq(user_id),
                predicate::eq(book_id),
                predicate::eq(target_book),
            )
            .returning(|_, _, _| Err(Box::new(CustomError::Duplicated("Book".to_string()))));

        let app = _get_router(user_id, mock_usecase);
        let req = _get_request(book_id, json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }

    #[tokio::test]
    async fn check_no_role() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let target_book = EditBook::new("권한 없는 가계부".to_string());
        let json_body = to_string(&target_book).unwrap();

        let mut mock_usecase = MockUpdateBookUsecaseImpl::new();
        mock_usecase
            .expect_update_book()
            .with(
                predicate::eq(user_id),
                predicate::eq(book_id),
                predicate::eq(target_book),
            )
            .returning(|_, _, _| Err(Box::new(CustomError::Unauthorized("Book".to_string()))));

        let app = _get_router(user_id, mock_usecase);
        let req = _get_request(book_id, json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }
}
