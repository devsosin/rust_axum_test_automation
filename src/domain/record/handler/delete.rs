use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::record::usecase::delete::DeleteRecordUsecase;

pub async fn delete_record<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Path(record_id): Path<i64>,
) -> impl IntoResponse
where
    T: DeleteRecordUsecase,
{
    match usecase.delete_record(user_id, record_id).await {
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
        domain::record::{handler::delete::delete_record, usecase::delete::DeleteRecordUsecase},
        global::errors::CustomError,
    };

    mock! {
        DeleteRecordUsecaseImpl {}

        #[async_trait]
        impl DeleteRecordUsecase for DeleteRecordUsecaseImpl {
            async fn delete_record(&self, user_id: i32, record_id: i64) -> Result<(), Box<CustomError>>;
        }
    }

    fn _create_app(user_id: i32, mock_usecase: MockDeleteRecordUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/record/:record_id",
                delete(delete_record::<MockDeleteRecordUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id))
    }

    fn _create_req(record_id: i64) -> Request {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/record/{}", record_id))
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn check_delete_record_status() {
        // Arrange
        let user_id = 1;
        let record_id = 1i64;
        let mut mock_usecase = MockDeleteRecordUsecaseImpl::new();
        mock_usecase
            .expect_delete_record()
            .with(predicate::eq(user_id), predicate::eq(record_id))
            .returning(|_, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(record_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_delete_record_body() {
        // Arrange
        let user_id = 1;
        let record_id = 1i64;
        let mut mock_usecase = MockDeleteRecordUsecaseImpl::new();
        mock_usecase
            .expect_delete_record()
            .with(predicate::eq(user_id), predicate::eq(record_id))
            .returning(|_, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(record_id);

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
        assert_eq!(body_json["message"], "성공");
    }

    #[tokio::test]
    async fn check_id_not_found() {
        // Arrange
        let user_id = 1;
        let no_id = -32i64;
        let mut mock_usecase = MockDeleteRecordUsecaseImpl::new();
        mock_usecase
            .expect_delete_record()
            .with(predicate::eq(user_id), predicate::eq(no_id))
            .returning(|_, _| Err(Box::new(CustomError::NotFound("Record".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(no_id);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404);
    }
}
