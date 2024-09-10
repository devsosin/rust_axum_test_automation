use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::book::usecase::delete::DeleteBookUsecase;

pub(crate) async fn delete_book<T>(
    Extension(usecase): Extension<Arc<T>>,
    Path(id): Path<i32>,
) -> impl IntoResponse
where
    T: DeleteBookUsecase,
{
    match usecase.delete_book(id).await {
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
            async fn delete_book(&self, id: i32) -> Result<(), Arc<CustomError>>;
        }
    }

    fn get_request(id: i32) -> Request<Body> {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/book/{}", id))
            .body(Body::from(()))
            .unwrap()
    }

    fn get_router(id: i32, ret: Result<(), Arc<CustomError>>) -> Router {
        let mut mock_usecase = MockDeleteBookUsecaseImpl::new();
        mock_usecase
            .expect_delete_book()
            .with(predicate::eq(id))
            .returning(move |_| ret.clone());

        let app = Router::new()
            .route(
                "/api/v1/book/:book_id",
                delete(delete_book::<MockDeleteBookUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        app
    }

    #[tokio::test]
    async fn check_delete_book_status() {
        // Arrange
        let id = 1;
        let app = get_router(id, Ok(()));
        let req = get_request(id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_delete_book_body() {
        // Arrange
        let id = 1;
        let app = get_router(id, Ok(()));
        let req = get_request(id);

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
    async fn check_delete_book_id_not_found() {
        // Arrange
        let id = -32;
        let app = get_router(id, Err(Arc::new(CustomError::NotFound("Book".to_string()))));
        let req = get_request(id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }
}
