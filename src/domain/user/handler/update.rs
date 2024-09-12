use std::sync::Arc;

use axum::{extract::Path, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::{
    domain::user::{
        dto::request::EditUser, usecase::update::UpdateUserUsecase,
        utils::validator::validation_phone,
    },
    global::errors::CustomError,
};

pub async fn update_user<T>(
    Extension(usecase): Extension<Arc<T>>,
    Path(id): Path<i32>,
    Json(edit_user): Json<EditUser>,
) -> impl IntoResponse
where
    T: UpdateUserUsecase,
{
    if let Some(phone) = edit_user.get_phone() {
        if !validation_phone(phone) {
            return CustomError::ValidationError("User".to_string()).into_response();
        }
    };

    match usecase.update_user(id, edit_user).await {
        Ok(_) => (StatusCode::OK, Json(json!({"message": "성공"}))).into_response(),
        Err(err) => err.as_ref().into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{async_trait, extract::Request, routing::patch, Extension, Router};
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::{to_string, Value};
    use tower::ServiceExt;

    use crate::{
        domain::user::{
            dto::request::{EditPassword, EditUser},
            usecase::update::UpdateUserUsecase,
        },
        global::errors::CustomError,
    };

    use super::update_user;

    mock! {
        UpdateUserUsecaseImpl {}

        #[async_trait]
        impl UpdateUserUsecase for UpdateUserUsecaseImpl {
            async fn update_user(&self, id: i32, edit_user: EditUser) -> Result<(), Arc<CustomError>>;
        }
    }

    fn _create_app(id: i32, edit_user: &EditUser, ret: Result<(), Arc<CustomError>>) -> Router {
        let mut mock_usecase = MockUpdateUserUsecaseImpl::new();
        mock_usecase
            .expect_update_user()
            .with(predicate::eq(id), predicate::eq(edit_user.clone()))
            .returning(move |_, _| ret.clone());

        Router::new()
            .route(
                "/api/v1/user/:user_id",
                patch(update_user::<MockUpdateUserUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
    }

    fn _create_req(id: i32, edit_user: &EditUser) -> Request<String> {
        Request::builder()
            .method("PATCH")
            .uri(format!("/api/v1/user/{}", id))
            .header("content-type", "application/json")
            .body(to_string(edit_user).unwrap())
            .unwrap()
    }

    #[tokio::test]
    async fn check_update_user_status() {
        // Arrange
        let id = 1;
        let edit_user = EditUser::new(
            None,
            Some(EditPassword::new(
                "new_password".to_string(),
                "original_password".to_string(),
            )),
            Some("010-2345-6789".to_string()),
            None,
        );

        let app = _create_app(id, &edit_user, Ok(()));
        let req = _create_req(id, &edit_user);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_update_user_body() {
        // Arrange
        let id = 1;
        let edit_user = EditUser::new(
            None,
            Some(EditPassword::new(
                "new_password".to_string(),
                "original_password".to_string(),
            )),
            Some("010-2345-6789".to_string()),
            None,
        );

        let app = _create_app(id, &edit_user, Ok(()));
        let req = _create_req(id, &edit_user);

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
    async fn check_phone_validation() {
        // Arrange
        let id = 1;
        let edit_user = EditUser::new(
            None,
            Some(EditPassword::new(
                "new_password".to_string(),
                "original_password".to_string(),
            )),
            Some("010-11-3".to_string()),
            None,
        );

        let app = _create_app(id, &edit_user, Ok(()));
        let req = _create_req(id, &edit_user);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }

    #[tokio::test]
    async fn check_update_user_not_found() {
        // Arrange
        let id = -32;
        let edit_user = EditUser::new(
            None,
            Some(EditPassword::new(
                "new_password".to_string(),
                "original_password".to_string(),
            )),
            Some("010-2345-6789".to_string()),
            None,
        );

        let app = _create_app(
            id,
            &edit_user,
            Err(Arc::new(CustomError::NotFound("User".to_string()))),
        );
        let req = _create_req(id, &edit_user);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_update_user_unauthorized() {
        // Arrange
        let id = 1;
        let edit_user = EditUser::new(
            None,
            Some(EditPassword::new(
                "new_password".to_string(),
                "incorrect_password".to_string(),
            )),
            Some("010-2345-6789".to_string()),
            None,
        );

        let app = _create_app(
            id,
            &edit_user,
            Err(Arc::new(CustomError::ValidationError("User".to_string()))),
        );
        let req = _create_req(id, &edit_user);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
