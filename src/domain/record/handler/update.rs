use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::domain::record::{dto::request::EditRecord, usecase::update::UpdateRecordUsecase};

pub(crate) async fn update_record<T>(
    Extension(usecase): Extension<Arc<T>>,
    Path(id): Path<i64>,
    Json(edit_record): Json<EditRecord>,
) -> impl IntoResponse
where
    T: UpdateRecordUsecase,
{
    match usecase.update_record(id, edit_record).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "성공"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"message": e})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{async_trait, body::Body, extract::Request, routing::patch, Extension, Router};
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::{to_string, Value};
    use tower::ServiceExt;

    use crate::domain::record::{dto::request::EditRecord, usecase::update::UpdateRecordUsecase};

    use super::update_record;

    mock! {
        UpdateRecordUsecaseImpl {}

        #[async_trait]
        impl UpdateRecordUsecase for UpdateRecordUsecaseImpl {
            async fn update_record(&self, id: i64, edit_record: EditRecord) -> Result<(), String>;
        }
    }

    #[tokio::test]
    async fn check_update_record_status() {
        // Arrange
        let id = 1;
        let edit_record = EditRecord::new(None, Some(15000), Some("NULL".to_string()), None, None);

        let mut mock_usecase = MockUpdateRecordUsecaseImpl::new();
        mock_usecase
            .expect_update_record()
            .with(predicate::eq(id), predicate::eq(edit_record.clone()))
            .returning(|_, _| Ok(()));

        let app = Router::new()
            .route(
                "/api/v1/record/:record_id",
                patch(update_record::<MockUpdateRecordUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/record/{}", id))
            .header("content-type", "application/json")
            .body(Body::from(to_string(&edit_record).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_update_record_body() {
        // Arrange
        let id = 1;
        let edit_record = EditRecord::new(None, Some(15000), Some("NULL".to_string()), None, None);

        let mut mock_usecase = MockUpdateRecordUsecaseImpl::new();
        mock_usecase
            .expect_update_record()
            .with(predicate::eq(id), predicate::eq(edit_record.clone()))
            .returning(|_, _| Ok(()));

        let app = Router::new()
            .route(
                "/api/v1/record/:record_id",
                patch(update_record::<MockUpdateRecordUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/record/{}", id))
            .header("content-type", "application/json")
            .body(Body::from(to_string(&edit_record).unwrap()))
            .unwrap();

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

        assert_eq!(body_json["message"], "성공")
    }

    #[tokio::test]
    async fn check_update_record_not_found() {
        // Arrange
        let id = -32;
        let edit_record = EditRecord::new(None, Some(15000), Some("NULL".to_string()), None, None);

        let mut mock_usecase = MockUpdateRecordUsecaseImpl::new();
        mock_usecase
            .expect_update_record()
            .with(predicate::eq(id), predicate::eq(edit_record.clone()))
            .returning(|_, _| Err("id not found".to_string()));

        let app = Router::new()
            .route(
                "/api/v1/record/:record_id",
                patch(update_record::<MockUpdateRecordUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/record/{}", id))
            .header("content-type", "application/json")
            .body(Body::from(to_string(&edit_record).unwrap()))
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }
}
