use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::{
    domain::image::{
        dto::request::NewImages, usecase::create::CreateImageUsecase,
        utils::validator::validation_image,
    },
    global::errors::CustomError,
};

pub async fn create_images<T>(
    Extension(usecase): Extension<Arc<T>>,
    Json(images): Json<NewImages>,
) -> impl IntoResponse
where
    T: CreateImageUsecase,
{
    if images.get_file_names().is_empty() {
        return CustomError::ValidationError("Image Empty".to_string()).into_response();
    }
    for image in images.get_file_names() {
        if !validation_image(image) {
            return CustomError::ValidationError("Image Valid".to_string()).into_response();
        }
    }

    match usecase.create_images(images).await {
        Ok(urls) => (StatusCode::CREATED, Json(json!({"data": urls}))).into_response(),
        Err(e) => e.into_response(),
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
        config::aws::get_bucket,
        domain::image::{
            dto::{request::NewImages, response::PresignedUrl},
            usecase::create::CreateImageUsecase,
        },
        global::{errors::CustomError, utils::get_uuid},
    };

    use super::create_images;

    mock! {
        CreateImageUsecaseImpl {}

        #[async_trait]
        impl CreateImageUsecase for CreateImageUsecaseImpl {
            async fn create_images(&self, images: NewImages)
                -> Result<Vec<PresignedUrl>, Box<CustomError>>;
        }
    }

    async fn _create_app(new_images: &NewImages) -> Router {
        let bucket = get_bucket();

        let presigned_url = bucket
            .presign_put(format!("raw/{}.jpg", get_uuid()), 600, None, None)
            .await
            .unwrap();

        let img_len: i32 = new_images.len() as i32;

        let mut mock_usecase = MockCreateImageUsecaseImpl::new();
        mock_usecase
            .expect_create_images()
            .with(predicate::eq(new_images.clone()))
            .returning(move |_| {
                Ok((1..=img_len)
                    .into_iter()
                    .map(|i| PresignedUrl::new(i as i32, presigned_url.clone()))
                    .collect())
            });

        Router::new()
            .route(
                "/api/v1/image",
                post(create_images::<MockCreateImageUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
    }

    fn _create_req(new_images: &NewImages) -> Request {
        Request::builder()
            .method("POST")
            .uri("/api/v1/image")
            .header("content-type", "application/json")
            .body(Body::from(to_string(&new_images).unwrap()))
            .unwrap()
    }

    #[tokio::test]
    async fn check_create_image_success() {
        // Arrange
        let new_images = NewImages::new(vec![
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
        ]);

        let app = _create_app(&new_images).await;
        let req = _create_req(&new_images);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 201)
    }

    #[tokio::test]
    async fn check_create_image_body() {
        // Arrange
        let new_images = NewImages::new(vec![
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
            "test_images.jpg".to_string(),
        ]);

        let app = _create_app(&new_images).await;
        let req = _create_req(&new_images);

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
        assert_eq!(
            body_json["data"].as_array().unwrap().len(),
            new_images.len()
        )
    }

    #[tokio::test]
    async fn check_no_images() {
        // Arrange
        let new_images = NewImages::new(vec![]);

        let app = _create_app(&new_images).await;
        let req = _create_req(&new_images);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }

    #[tokio::test]
    async fn check_content_type_validation() {
        // Arrange
        let new_images = NewImages::new(vec![
            "test_images.png".to_string(),
            "test_images.eif".to_string(),
            "test_images.avi".to_string(),
            "test_images.mp4".to_string(),
            "test_images.zip".to_string(),
        ]);

        let app = _create_app(&new_images).await;
        let req = _create_req(&new_images);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
