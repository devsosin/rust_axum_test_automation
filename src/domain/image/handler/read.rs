use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::image::usecase::read::ReadImageUsecase;

pub async fn read_image<T>(
    Extension(usecase): Extension<Arc<T>>,
    Path(id): Path<i32>,
) -> impl IntoResponse
where
    T: ReadImageUsecase,
{
    match usecase.read_image(id).await {
        Ok(image) => (StatusCode::OK, Json(json!({"data": image}))).into_response(),
        Err(e) => e.into_response(),
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
        domain::image::{entity::Image, usecase::read::ReadImageUsecase},
        global::{errors::CustomError, utils::get_uuid},
    };

    use super::read_image;

    mock! {
        ReadImageUsecaseImpl {}

        #[async_trait]
        impl ReadImageUsecase for ReadImageUsecaseImpl {
            async fn read_image(&self, id: i32) -> Result<Image, Box<CustomError>>;
        }
    }

    fn _create_success_mock(id: i32) -> MockReadImageUsecaseImpl {
        let mut mock_usecase = MockReadImageUsecaseImpl::new();
        mock_usecase
            .expect_read_image()
            .with(predicate::eq(id))
            .returning(|i| {
                Ok(Image::new("test_image.jpg".to_string(), format!("{}.jpg", get_uuid())).id(i))
            });
        mock_usecase
    }

    fn _create_failure_mock(id: i32) -> MockReadImageUsecaseImpl {
        let mut mock_usecase = MockReadImageUsecaseImpl::new();
        mock_usecase
            .expect_read_image()
            .with(predicate::eq(id))
            .returning(move |i| Err(Box::new(CustomError::NotFound("Image".to_string()))));
        mock_usecase
    }

    fn _create_app(id: i32, is_success: bool) -> Router {
        let mut mock_usecase;
        if is_success {
            mock_usecase = _create_success_mock(id);
        } else {
            mock_usecase = _create_failure_mock(id);
        }

        Router::new()
            .route(
                "/api/v1/image/:image_id",
                get(read_image::<MockReadImageUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
    }

    fn _create_req(id: i32) -> Request {
        Request::builder()
            .method("GET")
            .uri(format!("/api/v1/image/{}", id))
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn check_read_image_status() {
        // Arrange
        let id = 1;
        let app = _create_app(id, true);
        let req = _create_req(id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_image_body() {
        // Arrange
        let id = 1;
        let app = _create_app(id, true);
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
        println!("{}", &body_str);
        let body_json: Value = serde_json::from_str(&body_str).expect("failed to parse JSON");

        // Assert
        assert_eq!(body_json["data"]["id"], id)
    }

    #[tokio::test]
    async fn check_read_image_not_found() {
        // Arrange
        let id = -32;
        let app = _create_app(id, false);
        let req = _create_req(id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }
}
