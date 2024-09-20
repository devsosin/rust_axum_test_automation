use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::{
    domain::category::usecase::update_sub::UpdateCategoryUsecase, global::errors::CustomError,
};

#[derive(Debug, Deserialize)]
pub struct EditSubCategory {
    name: String,
}

pub async fn update_sub_category<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(user_id): Extension<i32>,
    Path(sub_id): Path<i32>,
    Json(edit_sub): Json<EditSubCategory>,
) -> impl IntoResponse
where
    T: UpdateCategoryUsecase,
{
    if (&edit_sub.name).is_empty() {
        return CustomError::ValidationError("FieldEmpty".to_string()).into_response();
    }

    match usecase
        .update_sub_category(user_id, sub_id, edit_sub.name)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "성공"}))).into_response(),
        Err(err) => err.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        async_trait, body::Body, extract::Request, routing::patch, Extension, Json, Router,
    };
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::{json, to_string, Value};
    use tower::ServiceExt;

    use crate::{
        domain::category::usecase::update_sub::UpdateCategoryUsecase, global::errors::CustomError,
    };

    use super::update_sub_category;

    mock! {
        UpdateCategoryUsecaseImpl {}

        #[async_trait]
        impl UpdateCategoryUsecase for UpdateCategoryUsecaseImpl {
            async fn update_sub_category(
                &self,
                user_id: i32,
                sub_id: i32,
                name: String,
            ) -> Result<(), Box<CustomError>>;
        }
    }

    fn _create_app(user_id: i32, mock_usecase: MockUpdateCategoryUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/category/sub/:sub_id",
                patch(update_sub_category::<MockUpdateCategoryUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(user_id))
    }
    fn _create_req(sub_id: i32, name: &str) -> Request {
        Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/category/sub/{}", sub_id))
            .header("content-type", "application/json")
            .body(Body::from(to_string(&json!({"name": name})).unwrap()))
            .unwrap()
    }

    #[tokio::test]
    async fn check_update_sub_category_status() {
        // Arrange
        let user_id = 1;
        let sub_id = 1;
        let name = "수정용 이름";

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_sub_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(sub_id),
                predicate::eq(name.to_string()),
            )
            .returning(|_, _, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(sub_id, name);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_update_sub_category_body() {
        // Arrange
        let user_id = 1;
        let sub_id = 1;
        let name = "수정용 이름";

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_sub_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(sub_id),
                predicate::eq(name.to_string()),
            )
            .returning(|_, _, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(sub_id, name);

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
        let sub_id = 1;
        let name = "";

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_sub_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(sub_id),
                predicate::eq(name.to_string()),
            )
            .returning(|_, _, _| Ok(()));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(sub_id, name);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }

    #[tokio::test]
    async fn check_not_found() {
        // Arrange
        let user_id = 1;
        let sub_id = 1;
        let name = "수정수정";

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_sub_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(sub_id),
                predicate::eq(name.to_string()),
            )
            .returning(|_, _, _| Err(Box::new(CustomError::NotFound("SubCategory".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(sub_id, name);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_unauthorized() {
        // Arrange
        let user_id = 1;
        let sub_id = 1;
        let name = "수정수정";

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_sub_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(sub_id),
                predicate::eq(name.to_string()),
            )
            .returning(|_, _, _| {
                Err(Box::new(CustomError::Unauthorized(
                    "SubCategoryRole".to_string(),
                )))
            });

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(sub_id, name);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }

    #[tokio::test]
    async fn check_duplicated() {
        // Arrange
        let user_id = 1;
        let sub_id = 1;
        let name = "수정수정";

        let mut mock_usecase = MockUpdateCategoryUsecaseImpl::new();
        mock_usecase
            .expect_update_sub_category()
            .with(
                predicate::eq(user_id),
                predicate::eq(sub_id),
                predicate::eq(name.to_string()),
            )
            .returning(|_, _, _| Err(Box::new(CustomError::Duplicated("SubCategory".to_string()))));

        let app = _create_app(user_id, mock_usecase);
        let req = _create_req(sub_id, name);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
