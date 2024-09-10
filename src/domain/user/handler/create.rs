use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

use crate::{
    domain::user::{
        dto::request::NewUser, usecase::create::CreateUserUsecase,
        util::validation_password_strength,
    },
    global::errors::CustomError,
};

pub(crate) async fn create_user<T>(
    Extension(usecase): Extension<Arc<T>>,
    Json(new_user): Json<NewUser>,
) -> impl IntoResponse
where
    T: CreateUserUsecase,
{
    if !new_user.is_password_matching() {
        return CustomError::ValidationError("Password maching".to_string()).into_response();
    }

    if let Err(_) = validation_password_strength(new_user.password()) {
        return CustomError::ValidationError("Password validation".to_string()).into_response();
    }

    match usecase.create_user(new_user).await {
        Ok(id) => (StatusCode::CREATED, Json(json!({"user_id": id}))).into_response(),
        Err(e) => e.as_ref().into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{async_trait, extract::Request, routing::post, Extension, Router};
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::{to_string, Value};
    use tower::ServiceExt;

    use crate::{
        domain::user::{
            dto::request::{LoginType, NewUser},
            handler::create::create_user,
            usecase::create::CreateUserUsecase,
        },
        global::errors::CustomError,
    };

    mock! {
        CreateUserUsecaseImpl {}

        #[async_trait]
        impl CreateUserUsecase for CreateUserUsecaseImpl {
            async fn create_user(&self, new_user: NewUser) -> Result<i32, Arc<CustomError>>;
        }
    }

    #[tokio::test]
    async fn check_create_user_status() {
        // Arrange
        let new_user = NewUser::new(
            "test1234@test.test".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "nickname".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let mut mock_usecase = MockCreateUserUsecaseImpl::new();
        mock_usecase
            .expect_create_user()
            .with(predicate::eq(new_user.clone()))
            .returning(|_| Ok(1));

        let app = Router::new()
            .route(
                "/api/v1/user",
                post(create_user::<MockCreateUserUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)));

        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/user")
            .header("content-type", "application/json")
            .body(to_string(&new_user).unwrap())
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 201)
    }

    fn _create_app(new_user: &NewUser, ret: Result<i32, Arc<CustomError>>) -> Router {
        let mut mock_usecase = MockCreateUserUsecaseImpl::new();
        mock_usecase
            .expect_create_user()
            .with(predicate::eq(new_user.clone()))
            .returning(move |_| ret.clone());

        Router::new()
            .route(
                "/api/v1/user",
                post(create_user::<MockCreateUserUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
    }

    fn _create_req(new_user: &NewUser) -> Request<String> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/user")
            .header("content-type", "application/json")
            .body(to_string(&new_user).unwrap())
            .unwrap()
    }

    #[tokio::test]
    async fn check_create_user_body() {
        // Arrange
        let new_user = NewUser::new(
            "test1234@test.test".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "nickname".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let app = _create_app(&new_user, Ok(1));
        let req = _create_req(&new_user);

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
        assert_eq!(body_json["user_id"], 1)
    }

    #[tokio::test]
    async fn check_password_strength() {
        // Arrange
        let new_user = NewUser::new(
            "test1234@test.test".to_string(),
            "nostrong".to_string(),
            "nostrong".to_string(),
            "nickname".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let app = _create_app(&new_user, Ok(1));
        let req = _create_req(&new_user);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }

    #[tokio::test]
    async fn check_email_validation() {
        // Arrange
        let new_user = NewUser::new(
            "email@notvalid".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "nickname".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let app = _create_app(&new_user, Ok(1));
        let req = _create_req(&new_user);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }

    #[tokio::test]
    async fn check_phone_check() {
        // Arrange
        let new_user = NewUser::new(
            "email@notvalid".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "pwnotmatch".to_string(),
            "nickname".to_string(),
            LoginType::Email,
            Some("010-11-49483".to_string()),
            None,
            None,
        );

        // Act

        // Assert
        assert_eq!(1, 400)
    }

    #[tokio::test]
    async fn check_password_match() {
        // Arrange
        let new_user = NewUser::new(
            "email@notvalid".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "pwnotmatch".to_string(),
            "nickname".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let app = _create_app(&new_user, Ok(1));
        let req = _create_req(&new_user);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }

    #[tokio::test]
    async fn check_duplicated_user() {
        // Arrange
        let new_user = NewUser::new(
            "test@test.test".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "Str0nGPassW0rd!@".to_string(),
            "nickname".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let app = _create_app(
            &new_user,
            Err(Arc::new(CustomError::Duplicated("User".to_string()))),
        );
        let req = _create_req(&new_user);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
