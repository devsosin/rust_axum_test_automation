use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::category::usecase::read_sub::ReadCategoryUsecase;

pub async fn read_sub_category<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Path(base_id): Path<i16>,
) -> impl IntoResponse
where
    T: ReadCategoryUsecase,
{
    match usecase.read_sub_category(user_id, base_id).await {
        Ok(data) => (
            StatusCode::OK,
            Json(json!({"message": "성공", "data": data})),
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
        domain::category::{entity::SubCategory, usecase::read_sub::ReadCategoryUsecase},
        global::errors::CustomError,
    };

    use super::read_sub_category;

    mock! {
        ReadCategoryUsecaseImpl {}

        #[async_trait]
        impl ReadCategoryUsecase for ReadCategoryUsecaseImpl {
            async fn read_sub_category(
                &self,
                user_id: i32,
                base_id: i16,
            ) -> Result<Vec<SubCategory>, Box<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_read_sub_category_status() {
        // Arrange
        let user_id = 1;
        let base_id = 1;

        let mut mock_usecase = MockReadCategoryUsecaseImpl::new();
        mock_usecase
            .expect_read_sub_category()
            .with(predicate::eq(user_id), predicate::eq(base_id))
            .returning(|_, i| {
                Ok(vec![SubCategory::new(
                    i,
                    "테스트 서브 카테고리".to_string(),
                )
                .id(1)])
            });

        let app = Router::new()
            .route(
                "/api/v1/category/sub/:book_id",
                get(read_sub_category::<MockReadCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id));

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/category/sub/{}", base_id))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_sub_category_body() {
        // Arrange
        let user_id = 1;
        let base_id = 1;

        let mut mock_usecase = MockReadCategoryUsecaseImpl::new();
        mock_usecase
            .expect_read_sub_category()
            .with(predicate::eq(user_id), predicate::eq(base_id))
            .returning(|_, i| {
                Ok(vec![SubCategory::new(
                    i,
                    "테스트 서브 카테고리".to_string(),
                )
                .id(1)])
            });

        let app = Router::new()
            .route(
                "/api/v1/category/sub/:book_id",
                get(read_sub_category::<MockReadCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id));

        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/category/sub/{}", base_id))
            .body(Body::empty())
            .unwrap();

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
        assert_eq!(body_json["data"].as_array().unwrap().len(), 1)
    }
}
