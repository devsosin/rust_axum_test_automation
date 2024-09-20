use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::{
    domain::category::{
        dto::request::EditBaseCategory, usecase::update_base::UpdateCategoryUsecase,
    },
    global::errors::CustomError,
};

pub async fn update_base_category<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Path(base_id): Path<i16>,
    Json(edit_base): Json<EditBaseCategory>,
) -> impl IntoResponse
where
    T: UpdateCategoryUsecase,
{
    if let Some(v) = edit_base.get_name() {
        if v.is_empty() {
            return CustomError::ValidationError("FieldEmpty".to_string()).into_response();
        }
    }
    if let Some(v) = edit_base.get_color() {
        if v.is_empty() {
            return CustomError::ValidationError("FieldEmpty".to_string()).into_response();
        }
    }

    match usecase
        .update_base_category(user_id, base_id, edit_base)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "성공"}))).into_response(),
        Err(err) => err.into_response(),
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

    use crate::{
        domain::category::{
            dto::request::EditBaseCategory, usecase::update_base::UpdateCategoryUsecase,
        },
        global::errors::CustomError,
    };

    use super::update_base_category;

    mock! {
        UpdateCategoryUsecaseImpl {}

        #[async_trait]
        impl UpdateCategoryUsecase for UpdateCategoryUsecaseImpl {
            async fn update_base_category(
                &self,
                user_id: i32,
                base_id: i16,
                edit_base: EditBaseCategory,
            ) -> Result<(), Box<CustomError>>;
        }
    }

    fn _create_app(user_id: i32, mock_usecase: MockUpdateCategoryUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/category/base/:base_id",
                patch(update_base_category::<MockUpdateCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id))
    }

    fn _create_req(base_id: i16, edit_base: &EditBaseCategory) -> Request {
        Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/category/base/{}", base_id))
            .header("content-type", "application/json")
            .body(Body::from(to_string(&edit_base).unwrap()))
            .unwrap()
    }

    #[tokio::test]
    async fn check_update_base_category_status() {
        // Arrange
        let user_id = 1;
        let base_id = 1;
        let name = Some("name".to_string());
        let color = None;
        let edit_base = EditBaseCategory::new(name, color);

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_base_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(base_id),
                predicate::eq(edit_base.clone()),
            )
            .returning(|_, _, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id, &edit_base);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_update_base_category_body() {
        // Arrange
        let user_id = 1;
        let base_id = 1;
        let name = Some("name".to_string());
        let color = None;
        let edit_base = EditBaseCategory::new(name, color);

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_base_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(base_id),
                predicate::eq(edit_base.clone()),
            )
            .returning(|_, _, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id, &edit_base);

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
    async fn check_field_is_empty() {
        // Arrange
        let user_id = 1;
        let base_id = 1;
        let name = Some("".to_string());
        let color = None;
        let edit_base = EditBaseCategory::new(name, color);

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_base_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(base_id),
                predicate::eq(edit_base.clone()),
            )
            .returning(|_, _, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id, &edit_base);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }

    #[tokio::test]
    async fn check_not_found() {
        // Arrange
        let user_id = 1;
        let base_id = 1;
        let name = None;
        let color = Some("112334".to_string());
        let edit_base = EditBaseCategory::new(name, color);

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_base_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(base_id),
                predicate::eq(edit_base.clone()),
            )
            .returning(|_, _, _| Err(Box::new(CustomError::NotFound("BaseCategory".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id, &edit_base);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_unauthorized() {
        // Arrange
        let user_id = 1;
        let base_id = 1;
        let name = None;
        let color = Some("112334".to_string());
        let edit_base = EditBaseCategory::new(name, color);

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_base_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(base_id),
                predicate::eq(edit_base.clone()),
            )
            .returning(|_, _, _| {
                Err(Box::new(CustomError::Unauthorized(
                    "BaseCategoryRole".to_string(),
                )))
            });

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id, &edit_base);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }

    #[tokio::test]
    async fn check_duplicated() {
        // Arrange
        let user_id = 1;
        let base_id = 1;
        let name = Some("중복된 이름".to_string());
        let color = None;
        let edit_base = EditBaseCategory::new(name, color);

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_base_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(base_id),
                predicate::eq(edit_base.clone()),
            )
            .returning(|_, _, _| {
                Err(Box::new(CustomError::Duplicated(
                    "BaseCategory".to_string(),
                )))
            });

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(base_id, &edit_base);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
