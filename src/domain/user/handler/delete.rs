use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::user::usecase::delete::DeleteUserUsecase;

pub async fn delete_user<T>(
    Extension(usecase): Extension<Arc<T>>,
    Path(id): Path<i32>,
) -> impl IntoResponse
where
    T: DeleteUserUsecase,
{
    // field validation (값의 범위 등)

    match usecase.delete_user(id).await {
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

    use crate::{domain::user::usecase::delete::DeleteUserUsecase, global::errors::CustomError};

    use super::delete_user;

    mock! {
        DeleteUserUsecaseImpl {}

        #[async_trait]
        impl DeleteUserUsecase for DeleteUserUsecaseImpl {
            async fn delete_user(&self, id: i32) -> Result<(), Arc<CustomError>>;
        }
    }

    fn _create_app(id: i32, ret: Result<(), Arc<CustomError>>) -> Router {
        let mut mock_usecase = MockDeleteUserUsecaseImpl::new();
        mock_usecase
            .expect_delete_user()
            .with(predicate::eq(id))
            .returning(move |_| ret.clone());

        Router::new()
            .route(
                "/api/v1/user/:user_id",
                delete(delete_user::<MockDeleteUserUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
    }

    fn _create_req(id: i32) -> Request {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/user/{}", id))
            .body(Body::from(()))
            .unwrap()
    }

    #[tokio::test]
    async fn check_delete_user_status() {
        // Arrange
        let id = 1;
        let app = _create_app(id, Ok(()));
        let req = _create_req(id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_delete_user_body() {
        // Arrange
        let id = 1;
        let app = _create_app(id, Ok(()));
        let req = _create_req(id);

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
        assert_eq!(body_json["message"], "성공")
    }

    #[tokio::test]
    async fn check_delete_user_not_found() {
        // Arrange
        let id = -32;
        let app = _create_app(id, Err(Arc::new(CustomError::NotFound("User".to_string()))));
        let req = _create_req(id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }
}
