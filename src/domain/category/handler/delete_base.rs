use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::category::usecase::delete_base::DeleteCategoryUsecase;

pub async fn delete_base_category<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Path(base_id): Path<i16>,
) -> impl IntoResponse
where
    T: DeleteCategoryUsecase,
{
    match usecase.delete_base_category(user_id, base_id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "성공"}))).into_response(),
        Err(err) => err.into_response(),
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
        domain::category::usecase::delete_base::DeleteCategoryUsecase, global::errors::CustomError,
    };

    use super::delete_base_category;

    mock! {
        DeleteCategoryUsecaseImpl {}

        #[async_trait]
        impl DeleteCategoryUsecase for DeleteCategoryUsecaseImpl {
            async fn delete_base_category(
                &self,
                user_id: i32,
                base_id: i16,
            ) -> Result<(), Box<CustomError>>;
        }
    }

    fn _create_app(user_id: i32, mock_usecase: MockDeleteCategoryUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/category/base/:base_id",
                delete(delete_base_category::<MockDeleteCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id))
    }

    fn _create_req(base_id: i16) -> Request {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/category/base/{}", base_id))
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn check_delete_base_category_status() {
        // Arrange
        let user_id = 1;
        let base_id = 1;

        let mut mock_usecase = MockDeleteCategoryUsecaseImpl::new();
        mock_usecase
            .expect_delete_base_category()
            .with(predicate::eq(user_id), predicate::eq(base_id))
            .returning(|_, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_delete_base_category_body() {
        // Arrange
        let user_id = 1;
        let base_id = 1;

        let mut mock_usecase = MockDeleteCategoryUsecaseImpl::new();
        mock_usecase
            .expect_delete_base_category()
            .with(predicate::eq(user_id), predicate::eq(base_id))
            .returning(|_, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id);

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
    async fn check_not_found() {
        // Arrange
        let user_id = 1;
        let base_id = -33;

        let mut mock_usecase = MockDeleteCategoryUsecaseImpl::new();
        mock_usecase
            .expect_delete_base_category()
            .with(predicate::eq(user_id), predicate::eq(base_id))
            .returning(|_, _| Err(Box::new(CustomError::NotFound("BaseCategory".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_unauthorized() {
        // Arrange
        let user_id = 1;
        let base_id = -33;

        let mut mock_usecase = MockDeleteCategoryUsecaseImpl::new();
        mock_usecase
            .expect_delete_base_category()
            .with(predicate::eq(user_id), predicate::eq(base_id))
            .returning(|_, _| {
                Err(Box::new(CustomError::Unauthorized(
                    "BaseCategoryRole".to_string(),
                )))
            });

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }
}
