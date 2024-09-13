use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::record::usecase::delete::DeleteRecordUsecase;

pub async fn delete_record<T>(
    Extension(usecase): Extension<Arc<T>>,
    Path(id): Path<i64>,
) -> impl IntoResponse
where
    T: DeleteRecordUsecase,
{
    match usecase.delete_record(id).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "성공"}))).into_response(),
        Err(err) => err.as_ref().into_response(),
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
            async fn delete_record(&self, id: i64) -> Result<(), Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_delete_record_status() {
        // Arrange
        let id = 1i64;
        let mut mock_usecase = MockDeleteRecordUsecaseImpl::new();
        mock_usecase
            .expect_delete_record()
            .with(predicate::eq(id))
            .returning(|i| Ok(()));

        let app = Router::new()
            .route(
                "/api/v1/record/:record_id",
                delete(delete_record::<MockDeleteRecordUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/record/{}", id))
            .body(Body::from(()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_delete_record_body() {
        // Arrange
        let id = 1i64;
        let mut mock_usecase = MockDeleteRecordUsecaseImpl::new();
        mock_usecase
            .expect_delete_record()
            .with(predicate::eq(id))
            .returning(|i| Ok(()));

        let app = Router::new()
            .route(
                "/api/v1/record/:record_id",
                delete(delete_record::<MockDeleteRecordUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/record/{}", id))
            .body(Body::from(()))
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
        assert_eq!(body_json["message"], "성공");
    }

    #[tokio::test]
    async fn check_id_not_found() {
        // Arrange
        let id = -32i64;
        let mut mock_usecase = MockDeleteRecordUsecaseImpl::new();
        mock_usecase
            .expect_delete_record()
            .with(predicate::eq(id))
            .returning(|i| Err(Arc::new(CustomError::NotFound("Record".to_string()))));

        let app = Router::new()
            .route(
                "/api/v1/record/:record_id",
                delete(delete_record::<MockDeleteRecordUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/api/v1/record/{}", id))
            .body(Body::from(()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404);
    }
}
