use std::sync::Arc;

use axum::{extract::Query, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::{domain::connect::usecase::read::ReadConnectUsecase, global::errors::CustomError};

#[derive(Deserialize)]
pub(super) struct Params {
    name: String,
}

pub async fn read_connect<T>(
    Extension(usecase): Extension<Arc<T>>,
    params: Query<Params>,
) -> impl IntoResponse
where
    T: ReadConnectUsecase,
{
    if params.name.as_str().is_empty() {
        return CustomError::ValidationError("Connect".to_string()).into_response();
    };

    match usecase.read_connect(params.name.to_owned()).await {
        Ok(connect) => (
            StatusCode::OK,
            Json(json!({"message": "성공", "data": connect})),
        )
            .into_response(),
        Err(err) => err.into_response(),
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
        domain::connect::{entity::Connect, usecase::read::ReadConnectUsecase},
        global::errors::CustomError,
    };

    use super::read_connect;

    mock! {
        ReadConnectUsecaseImpl {}

        #[async_trait]
        impl ReadConnectUsecase for ReadConnectUsecaseImpl {
            async fn read_connect(&self, name: String) -> Result<Connect, Box<CustomError>>;
        }
    }

    fn _create_app(mock_usecase: MockReadConnectUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/connect",
                get(read_connect::<MockReadConnectUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
    }

    fn _create_req(name: &str) -> Request {
        let encoded_name: String = url::form_urlencoded::byte_serialize(name.as_bytes()).collect();
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/connect?name={}", encoded_name))
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn check_read_connect_status() {
        // Arrange
        let name = "커넥트 이름";

        let mut mock_usecase = MockReadConnectUsecaseImpl::new();
        mock_usecase
            .expect_read_connect()
            .with(predicate::eq(name.to_string()))
            .returning(|n| Ok(Connect::new(n).id(1)));

        let app = _create_app(mock_usecase);
        let req = _create_req(name);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_connect_body() {
        // Arrange
        let name = "커넥트 이름";

        let mut mock_usecase = MockReadConnectUsecaseImpl::new();
        mock_usecase
            .expect_read_connect()
            .with(predicate::eq(name.to_string()))
            .returning(|n| Ok(Connect::new(n).id(1)));

        let app = _create_app(mock_usecase);
        let req = _create_req(name);

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
        assert_eq!(body_json["data"]["id"], 1);
        assert_eq!(body_json["data"]["name"], name);
    }

    #[tokio::test]
    async fn check_not_found() {
        // Arrange
        let name = "없는 커넥트";

        let mut mock_usecase = MockReadConnectUsecaseImpl::new();
        mock_usecase
            .expect_read_connect()
            .with(predicate::eq(name.to_string()))
            .returning(|n| Err(Box::new(CustomError::NotFound("Connect".to_string()))));

        let app = _create_app(mock_usecase);
        let req = _create_req(name);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_name_empty() {
        // Arrange
        let name = "";

        let mut mock_usecase = MockReadConnectUsecaseImpl::new();
        mock_usecase
            .expect_read_connect()
            .with(predicate::eq(name.to_string()))
            .returning(|n| Ok(Connect::new(n).id(1)));

        let app = _create_app(mock_usecase);
        let req = _create_req(name);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
