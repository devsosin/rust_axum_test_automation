use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::record::{dto::request::NewRecord, usecase::create::CreateRecordUsecase};

pub async fn create_record<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Json(new_record): Json<NewRecord>,
) -> impl IntoResponse
where
    T: CreateRecordUsecase,
{
    match usecase.create_record(user_id, new_record).await {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({"message": "성공", "record_id": id})),
        )
            .into_response(),
        Err(err) => err.as_ref().into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{async_trait, extract::Request, routing::post, Extension, Router};
    use chrono::NaiveDateTime;
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::{to_string, Value};
    use tower::ServiceExt;

    use super::create_record;
    use crate::{
        domain::record::{dto::request::NewRecord, usecase::create::CreateRecordUsecase},
        global::errors::CustomError,
    };

    mock! {
        CreateRecordUsecaseImpl {}

        #[async_trait]
        impl CreateRecordUsecase for CreateRecordUsecaseImpl {
            async fn create_record(&self, user_id: i32, new_record: NewRecord) -> Result<i64, Box<CustomError>>;
        }
    }

    fn _create_app(user_id: i32, mock_usecase: MockCreateRecordUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/record",
                post(create_record::<MockCreateRecordUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id))
    }
    fn _create_req(new_record: &NewRecord) -> Request<String> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/record")
            .header("content-type", "application/json")
            .body(to_string(&new_record).unwrap())
            .unwrap()
    }

    #[tokio::test]
    async fn check_create_record_status() {
        // Arrange
        let user_id = 1;
        let new_record = NewRecord::new(
            1,
            18,
            15200,
            Some("감자탕".to_string()),
            NaiveDateTime::parse_from_str("2024-09-08 18:39:27", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
            None,
        );

        let mut mock_usecase = MockCreateRecordUsecaseImpl::new();
        mock_usecase
            .expect_create_record()
            .with(predicate::eq(user_id), predicate::eq(new_record.clone()))
            .returning(|_, _| Ok(1));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_record);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 201);
    }

    #[tokio::test]
    async fn check_create_record_body() {
        // Arrange
        let user_id = 1;
        let new_record = NewRecord::new(
            1,
            18,
            15200,
            Some("순대국".to_string()),
            NaiveDateTime::parse_from_str("2024-09-08 15:37:48", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
            Some(vec![1]),
        );

        let mut mock_usecase = MockCreateRecordUsecaseImpl::new();
        mock_usecase
            .expect_create_record()
            .with(predicate::eq(user_id), predicate::eq(new_record.clone()))
            .returning(|_, _| Ok(1));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_record);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        let body = response.into_body();

        let body_bytes = body
            .collect()
            .await
            .expect("failed to read body")
            .to_bytes();

        let body_str =
            String::from_utf8(body_bytes.to_vec()).expect("failed to convert body to string");

        let body_json: Value = serde_json::from_str(&body_str).expect("failed to parse JSON");

        assert_eq!(body_json["record_id"], 1);
    }

    #[tokio::test]
    async fn check_category_not_found() {
        // Arrange
        let user_id = 1;
        let new_record = NewRecord::new(
            1,
            -32,
            15200,
            None,
            NaiveDateTime::parse_from_str("2024-09-07 15:30:28", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
            None,
        );

        let mut mock_usecase = MockCreateRecordUsecaseImpl::new();
        mock_usecase
            .expect_create_record()
            .with(predicate::eq(user_id), predicate::eq(new_record.clone()))
            .returning(|_, _| Err(Box::new(CustomError::NotFound("Category".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_record);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_no_role() {
        let user_id = 1;
        let new_record = NewRecord::new(
            1,
            1,
            15200,
            None,
            NaiveDateTime::parse_from_str("2024-09-07 15:30:28", "%Y-%m-%d %H:%M:%S").unwrap(),
            None,
            None,
        );

        let mut mock_usecase = MockCreateRecordUsecaseImpl::new();
        mock_usecase
            .expect_create_record()
            .with(predicate::eq(user_id), predicate::eq(new_record.clone()))
            .returning(|_, _| {
                Err(Box::new(CustomError::Unauthorized(
                    "RecordRole".to_string(),
                )))
            });

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(&new_record);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }
}
