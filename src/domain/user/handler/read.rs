use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::user::usecase::read::ReadUserUsecase;

pub(crate) async fn read_user<T>(
    Extension(usecase): Extension<Arc<T>>,
    Path(id): Path<i32>,
) -> impl IntoResponse
where
    T: ReadUserUsecase,
{
    match usecase.read_user(id).await {
        Ok(user_info) => (StatusCode::OK, Json(json!(user_info))).into_response(),
        Err(e) => e.as_ref().into_response(),
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
        domain::user::{
            dto::{request::LoginType, response::UserInfo},
            usecase::read::ReadUserUsecase,
        },
        global::errors::CustomError,
    };

    use super::read_user;

    mock! {
        ReadUserUsecaseImpl {}

        #[async_trait]
        impl ReadUserUsecase for ReadUserUsecaseImpl {
            async fn read_user(&self, id: i32) -> Result<UserInfo, Arc<CustomError>>;
        }
    }

    fn _create_app(id: i32, ret: Result<UserInfo, Arc<CustomError>>) -> Router {
        let mut mock_usecase = MockReadUserUsecaseImpl::new();
        mock_usecase
            .expect_read_user()
            .with(predicate::eq(id))
            .returning(move |_| ret.clone());

        Router::new()
            .route(
                "/api/v1/user/:user_id",
                get(read_user::<MockReadUserUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
    }

    fn _create_req(id: i32) -> Request<Body> {
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/user/{}", id))
            .body(Body::from(()))
            .unwrap()
    }

    #[tokio::test]
    async fn check_read_user_status() {
        // Arrange
        let id = 1;
        let user_info = UserInfo::new(
            id,
            "test1234@test.test".to_string(),
            "nickname".to_string(),
            LoginType::Email,
            Some("010-1234-5678".to_string()),
            None,
        );

        let app = _create_app(id, Ok(user_info));
        let req = _create_req(id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_user_body() {
        // Arrange
        let id = 1;
        let user_email = "test1234@test.test";
        let user_info = UserInfo::new(
            id,
            user_email.to_string(),
            "nickname".to_string(),
            LoginType::Email,
            Some("010-1234-5678".to_string()),
            None,
        );

        let app = _create_app(id, Ok(user_info));
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
        assert_eq!(body_json["id"], id);
        assert_eq!(body_json["email"], user_email)
    }

    #[tokio::test]
    async fn check_not_found() {
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
