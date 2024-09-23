use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::connect::{dto::request::NewConnect, usecase::create::CreateConnectUsecase};

pub async fn create_connect<T>(
    Extension(usecase): Extension<Arc<T>>,
    Json(new_connect): Json<NewConnect>,
) -> impl IntoResponse
where
    T: CreateConnectUsecase,
{
    match usecase.create_connect(new_connect).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({"message": "성공", "connect_id": id})),
        )
            .into_response(),
        Err(err) => err.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{async_trait, body::Body, extract::Request, routing::post, Extension, Router};
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::{to_string, Value};
    use tower::ServiceExt;

    use crate::{
        domain::connect::{dto::request::NewConnect, usecase::create::CreateConnectUsecase},
        global::errors::CustomError,
    };

    use super::create_connect;

    mock! {
        CreateConnectUsecaseImpl {}

        #[async_trait]
        impl CreateConnectUsecase for CreateConnectUsecaseImpl {
            async fn create_connect(&self, new_connect: NewConnect) -> Result<i32, Box<CustomError>>;
        }
    }

    fn _create_app(mock_usecase: MockCreateConnectUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/connect",
                post(create_connect::<MockCreateConnectUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
    }

    fn _create_req(new_connect: &NewConnect) -> Request {
        Request::builder()
            .method("POST")
            .uri("/api/v1/connect")
            .header("content-type", "application/json")
            .body(Body::from(to_string(new_connect).unwrap()))
            .unwrap()
    }

    #[tokio::test]
    async fn check_create_connect_status() {
        // Arrange
        let new_connect = NewConnect::new("테스트 커넥션".to_string());
        let mut mock_usecase = MockCreateConnectUsecaseImpl::new();
        mock_usecase
            .expect_create_connect()
            .with(predicate::eq(new_connect.clone()))
            .returning(|_| Ok(1));

        let app = _create_app(mock_usecase);
        let req = _create_req(&new_connect);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 201)
    }

    #[tokio::test]
    async fn check_create_connect_body() {
        // Arrange
        let new_connect = NewConnect::new("테스트 커넥션".to_string());
        let mut mock_usecase = MockCreateConnectUsecaseImpl::new();
        mock_usecase
            .expect_create_connect()
            .with(predicate::eq(new_connect.clone()))
            .returning(|_| Ok(1));

        let app = _create_app(mock_usecase);
        let req = _create_req(&new_connect);

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
        assert_eq!(body_json["connect_id"], 1)
    }

    #[tokio::test]
    async fn check_duplicated() {
        // Arrange
        let new_connect = NewConnect::new("중복 커넥션".to_string());
        let mut mock_usecase = MockCreateConnectUsecaseImpl::new();
        mock_usecase
            .expect_create_connect()
            .with(predicate::eq(new_connect.clone()))
            .returning(|_| Err(Box::new(CustomError::Duplicated("Connect".to_string()))));

        let app = _create_app(mock_usecase);
        let req = _create_req(&new_connect);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
