use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::book::usecase::delete::DeleteBookUsecase;

pub async fn delete_book<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Path(book_id): Path<i32>,
) -> impl IntoResponse
where
    T: DeleteBookUsecase,
{
    match usecase.delete_book(user_id, book_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "성공"}))).into_response(),
        Err(err) => err.as_ref().into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{async_trait, body::Body, extract::Request, routing::delete, Extension, Router};
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::{
        domain::book::{handler::delete::delete_book, usecase::delete::DeleteBookUsecase},
        global::errors::CustomError,
    };

    mock! {
        DeleteBookUsecaseImpl {}

        #[async_trait]
        impl DeleteBookUsecase for DeleteBookUsecaseImpl {
            async fn delete_book(&self, user_id: i32, book_id: i32) -> Result<(), Box<CustomError>>;
        }
    }

    fn get_request(id: i32) -> Request<Body> {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/book/{}", id))
            .body(Body::from(()))
            .unwrap()
    }

    fn get_router(user_id: i32, mock_usecase: MockDeleteBookUsecaseImpl) -> Router {
        let app = Router::new()
            .route(
                "/api/v1/book/:book_id",
                delete(delete_book::<MockDeleteBookUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id));

        app
    }

    #[tokio::test]
    async fn check_delete_book_status() {
        // Arrange
        let user_id = 1;
        let book_id = 1;

        let mut mock_usecase = MockDeleteBookUsecaseImpl::new();
        mock_usecase
            .expect_delete_book()
            .with(predicate::eq(user_id), predicate::eq(book_id))
            .returning(move |_, _| Ok(()));
        let app = get_router(user_id, mock_usecase);
        let req = get_request(book_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_delete_book_body() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let mut mock_usecase = MockDeleteBookUsecaseImpl::new();
        mock_usecase
            .expect_delete_book()
            .with(predicate::eq(user_id), predicate::eq(book_id))
            .returning(move |_, _| Ok(()));
        let app = get_router(user_id, mock_usecase);
        let req = get_request(book_id);

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

        assert_eq!(body_json["message"], "성공")
    }

    #[tokio::test]
    async fn check_book_not_found() {
        // Arrange
        let user_id = 1;
        let book_id = -32;
        let mut mock_usecase = MockDeleteBookUsecaseImpl::new();
        mock_usecase
            .expect_delete_book()
            .with(predicate::eq(user_id), predicate::eq(book_id))
            .returning(move |_, _| Err(Box::new(CustomError::NotFound("Book".to_string()))));
        let app = get_router(user_id, mock_usecase);
        let req = get_request(book_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_no_role() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let mut mock_usecase = MockDeleteBookUsecaseImpl::new();
        mock_usecase
            .expect_delete_book()
            .with(predicate::eq(user_id), predicate::eq(book_id))
            .returning(move |_, _| {
                Err(Box::new(CustomError::Unauthorized("BookRole".to_string())))
            });
        let app = get_router(user_id, mock_usecase);
        let req = get_request(book_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }
}
