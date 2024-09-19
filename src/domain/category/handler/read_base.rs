use std::sync::Arc;

use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    Extension, Json,
};
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::domain::category::usecase::read_base::ReadCategoryUsecase;

#[derive(Deserialize)]
pub(super) struct Params {
    is_record: Option<bool>,
}

pub async fn read_base_category<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Path(book_id): Path<i32>,
    params: Query<Params>,
) -> impl IntoResponse
where
    T: ReadCategoryUsecase,
{
    let is_record = params.is_record.unwrap_or_else(|| true);

    match usecase
        .read_base_category(user_id, book_id, is_record)
        .await
    {
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
        domain::category::{entity::BaseCategory, usecase::read_base::ReadCategoryUsecase},
        global::errors::CustomError,
    };

    use super::read_base_category;

    mock! {
        ReadCategoryUsecaseImpl {}

        #[async_trait]
        impl ReadCategoryUsecase for ReadCategoryUsecaseImpl {
            async fn read_base_category(
                &self,
                user_id: i32,
                book_id: i32,
                is_record: bool,
            ) -> Result<Vec<BaseCategory>, Box<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_read_base_category_status() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let is_record = true;

        let mut mock_usecase = MockReadCategoryUsecaseImpl::new();
        mock_usecase
            .expect_read_base_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(book_id),
                predicate::eq(is_record),
            )
            .returning(|_, i, r| {
                Ok(vec![BaseCategory::new(
                    1,
                    i,
                    r,
                    true,
                    "테스트 베이스 카테고리".to_string(),
                    "112233".to_string(),
                )])
            });

        let app = Router::new()
            .route(
                "/api/v1/category/base/:book_id",
                get(read_base_category::<MockReadCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id));

        let params = format!("?is_record={}", is_record);
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/category/base/{}{}", book_id, params))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();
        println!("{:?}", &response);

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_read_base_category_body() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let is_record = true;

        let mut mock_usecase = MockReadCategoryUsecaseImpl::new();
        mock_usecase
            .expect_read_base_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(book_id),
                predicate::eq(is_record),
            )
            .returning(|_, i, r| {
                Ok(vec![BaseCategory::new(
                    1,
                    i,
                    r,
                    true,
                    "테스트 베이스 카테고리".to_string(),
                    "112233".to_string(),
                )])
            });

        let app = Router::new()
            .route(
                "/api/v1/category/base/:book_id",
                get(read_base_category::<MockReadCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id));

        let params = format!("?is_record={}", is_record);
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/category/base/{}{}", book_id, params))
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
        println!("{}", &body_str);
        let body_json: Value = serde_json::from_str(&body_str).expect("failed to parse JSON");

        // Assert
        assert_eq!(body_json["data"].as_array().unwrap().len(), 1)
    }

    #[tokio::test]
    async fn check_query_is_not_present() {
        // Arrange
        let user_id = 1;
        let book_id = 1;
        let is_record = true;

        let mut mock_usecase = MockReadCategoryUsecaseImpl::new();
        mock_usecase
            .expect_read_base_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(book_id),
                predicate::eq(is_record),
            )
            .returning(|_, i, r| {
                Ok(vec![BaseCategory::new(
                    1,
                    i,
                    r,
                    true,
                    "테스트 베이스 카테고리".to_string(),
                    "112233".to_string(),
                )])
            });

        let app = Router::new()
            .route(
                "/api/v1/category/base/:book_id",
                get(read_base_category::<MockReadCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id));

        let params = "";
        let req = Request::builder()
            .method("GET")
            .uri(format!("/api/v1/category/base/{}{}", book_id, params))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }
}
