use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::book::{dto::request::EditBook, usecase::update::UpdateBookUsecase};

pub(crate) async fn update_book<T>(
    Extension(usecase): Extension<Arc<T>>,
    Path(id): Path<i32>,
    Json(edit_book): Json<EditBook>,
) -> impl IntoResponse
where
    T: UpdateBookUsecase,
{
    match usecase.update_book(id, &edit_book).await {
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
            async fn update_book(&self, id: i32, edit_book: &EditBook) -> Result<(), Arc<CustomError>>;
        }
    }

    pub fn get_request(id: i32, json_body: String) -> Request<Body> {
        Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/book/{}", id))
            .header("content-type", "application/json")
            .body(Body::from(json_body))
            .unwrap()
    }

    pub fn get_router(id: i32, edit_book: EditBook, ret: Result<(), Arc<CustomError>>) -> Router {
        let mut mock_usecase = MockUpdateBookUsecaseImpl::new();
        mock_usecase
            .expect_update_book()
            .with(predicate::eq(id), predicate::eq(edit_book))
            .returning(move |_, _| ret.clone());

        let app = Router::new()
            .route(
                "/api/v1/book/:book_id",
                patch(update_book::<MockUpdateBookUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        app
    }

    #[tokio::test]
    async fn check_update_book_status() {
        // Arrange
        let id = 1;
        let target_book = EditBook::new("수정된 가계부".to_string());
        let json_body = to_string(&target_book).unwrap();

        let app = get_router(id, target_book.clone(), Ok(()));
        let req = get_request(id, json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_update_book_body() {
        // Arrange
        let id = 1;
        let target_book = EditBook::new("수정된 가계부".to_string());
        let json_body = to_string(&target_book).unwrap();

        let app = get_router(id, target_book.clone(), Ok(()));
        let req = get_request(id, json_body);

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
    async fn check_update_book_not_found() {
        // Arrange
        let no_id = -32;
        let target_book = EditBook::new("없는 가계부".to_string());
        let json_body = to_string(&target_book).unwrap();

        let app = get_router(
            no_id,
            target_book.clone(),
            Err(Arc::new(CustomError::NotFound("Book".to_string()))),
        );
        let req = get_request(no_id, json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_update_book_duplicate() {
        // Arrange
        let id = 1;
        let target_book = EditBook::new("중복 이름".to_string());
        let json_body = to_string(&target_book).unwrap();

        let app = get_router(
            id,
            target_book.clone(),
            Err(Arc::new(CustomError::Duplicated("Book".to_string()))),
        );
        let req = get_request(id, json_body);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
