use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::category::{
    dto::request::NewBaseCategory, usecase::create_base::CreateCategoryUsecase,
};

pub async fn create_base_category<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Json(new_base): Json<NewBaseCategory>,
) -> impl IntoResponse
where
    T: CreateCategoryUsecase,
{
    match usecase.create_base_category(user_id, new_base).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({"message": "성공", "base_id": id})),
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
    use sqlx::Error;
    use tower::ServiceExt;

    use crate::{
        domain::category::{
            dto::request::NewBaseCategory, usecase::create_base::CreateCategoryUsecase,
        },
        global::errors::CustomError,
    };

    use super::create_base_category;

    mock! {
        CreateCategoryUsecaseImpl {}

        #[async_trait]
        impl CreateCategoryUsecase for CreateCategoryUsecaseImpl {
            async fn create_base_category(
                &self,
                user_id: i32,
                new_base: NewBaseCategory,
            ) -> Result<i16, Box<CustomError>>;

        }
    }

    fn _create_app(user_id: i32, mock_usecase: MockCreateCategoryUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/category/base",
                post(create_base_category::<MockCreateCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id))
    }

    fn _create_req(new_base: &NewBaseCategory) -> Request {
        Request::builder()
            .method("POST")
            .uri("/api/v1/category/base")
            .header("content-type", "application/json")
            .body(Body::from(to_string(new_base).unwrap()))
            .unwrap()
    }

    #[tokio::test]
    async fn check_create_base_status() {
        // Arrange
        let user_id = 1;
        let new_base = NewBaseCategory::new(
            1,
            1,
            true,
            false,
            "테스팅".to_string(),
            "123456".to_string(),
        );

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_base_category()
            .with(predicate::eq(user_id), predicate::eq(new_base.clone()))
            .returning(|_, _| Ok(1));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_base);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 201)
    }

    #[tokio::test]
    async fn check_create_base_body() {
        // Arrange
        let user_id = 1;
        let new_base = NewBaseCategory::new(
            1,
            1,
            true,
            false,
            "테스팅".to_string(),
            "123456".to_string(),
        );

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_base_category()
            .with(predicate::eq(user_id), predicate::eq(new_base.clone()))
            .returning(|_, _| Ok(1));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_base);

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
        assert_eq!(body_json["base_id"], 1)
    }

    #[tokio::test]
    async fn check_not_found() {
        // Arrange
        let user_id = 1;
        let new_base = NewBaseCategory::new(
            1,
            -3,
            true,
            false,
            "테스팅".to_string(),
            "123456".to_string(),
        );

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_base_category()
            .with(predicate::eq(user_id), predicate::eq(new_base.clone()))
            .returning(|_, _| Err(Box::new(CustomError::NotFound("Book".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_base);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_type_error() {
        // Arrange
        let user_id = 1;
        let new_base = NewBaseCategory::new(
            -3,
            1,
            true,
            false,
            "테스팅".to_string(),
            "123456".to_string(),
        );

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_base_category()
            .with(predicate::eq(user_id), predicate::eq(new_base.clone()))
            .returning(|_, _| Err(Box::new(CustomError::DatabaseError(Error::RowNotFound)))); // 임시 에러

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_base);

        // Act
        let response = app.oneshot(req).await.unwrap();
        println!("{:?}", response);

        // Assert
        assert_eq!(response.status(), 400)
    }

    #[tokio::test]
    async fn check_unauthorized() {
        // Arrange
        let user_id = 2;
        let new_base = NewBaseCategory::new(
            1,
            1,
            true,
            false,
            "테스팅".to_string(),
            "123456".to_string(),
        );

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_base_category()
            .with(predicate::eq(user_id), predicate::eq(new_base.clone()))
            .returning(|_, _| Err(Box::new(CustomError::Unauthorized("BookRole".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_base);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }

    #[tokio::test]
    async fn check_duplicated() {
        // Arrange
        let user_id = 1;
        let new_base = NewBaseCategory::new(
            1,
            1,
            true,
            false,
            "중복된 카테고리 이름".to_string(),
            "123456".to_string(),
        );

        let mut mock_usecase = MockCreateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_create_base_category()
            .with(predicate::eq(user_id), predicate::eq(new_base.clone()))
            .returning(|_, _| {
                Err(Box::new(CustomError::Duplicated(
                    "BaseCategory".to_string(),
                )))
            });

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_base);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
