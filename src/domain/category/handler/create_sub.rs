use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::category::{
    dto::request::NewSubCategory, usecase::create_sub::CreateCategoryUsecase,
};

pub async fn create_sub_category<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Json(new_sub): Json<NewSubCategory>,
) -> impl IntoResponse
where
    T: CreateCategoryUsecase,
{
    match usecase.create_sub_category(user_id, new_sub).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({"message": "성공", "sub_category_id": id})),
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
        domain::category::{
            dto::request::NewSubCategory, usecase::create_sub::CreateCategoryUsecase,
        },
        global::errors::CustomError,
    };

    use super::create_sub_category;

    mock! {
        CreateCategoryUsecaseImpl {}

        #[async_trait]
        impl CreateCategoryUsecase for CreateCategoryUsecaseImpl {
            async fn create_sub_category(
                &self,
                user_id: i32,
                new_sub: NewSubCategory,
            ) -> Result<i32, Box<CustomError>>;

        }
    }

    fn _create_app(user_id: i32, mock_usecase: MockCreateCategoryUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/category/sub",
                post(create_sub_category::<MockCreateCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id))
    }

    fn _create_req(new_sub: &NewSubCategory) -> Request {
        Request::builder()
            .method("POST")
            .uri("/api/v1/category/sub")
            .header("content-type", "application/json")
            .body(Body::from(to_string(new_sub).unwrap()))
            .unwrap()
    }

    #[tokio::test]
    async fn check_create_sub_status() {
        // Arrange
        let user_id = 1;
        let new_sub = NewSubCategory::new(1, "테스팅".to_string());

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_sub_category()
            .with(predicate::eq(user_id), predicate::eq(new_sub.clone()))
            .returning(|_, _| Ok(1));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_sub);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 201)
    }

    #[tokio::test]
    async fn check_create_base_body() {
        // Arrange
        let user_id = 1;
        let new_sub = NewSubCategory::new(1, "테스팅".to_string());

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_sub_category()
            .with(predicate::eq(user_id), predicate::eq(new_sub.clone()))
            .returning(|_, _| Ok(1));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_sub);

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
        assert_eq!(body_json["sub_category_id"], 1)
    }

    #[tokio::test]
    async fn check_not_found() {
        // Arrange
        let user_id = 1;
        let new_sub = NewSubCategory::new(1, "테스팅".to_string());

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_sub_category()
            .with(predicate::eq(user_id), predicate::eq(new_sub.clone()))
            .returning(|_, _| Err(Box::new(CustomError::NotFound("Base".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_sub);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_unauthorized() {
        // Arrange
        let user_id = 2;
        let new_sub = NewSubCategory::new(1, "테스팅".to_string());

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_sub_category()
            .with(predicate::eq(user_id), predicate::eq(new_sub.clone()))
            .returning(|_, _| Err(Box::new(CustomError::Unauthorized("BookRole".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_sub);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }

    #[tokio::test]
    async fn check_duplicated() {
        // Arrange
        let user_id = 1;
        let new_sub = NewSubCategory::new(1, "중복된 카테고리".to_string());

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_sub_category()
            .with(predicate::eq(user_id), predicate::eq(new_sub.clone()))
            .returning(|_, _| Err(Box::new(CustomError::Duplicated("SubCategory".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_sub);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
