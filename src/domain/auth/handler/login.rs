use std::sync::Arc;

use axum::{
    http::{header, Response},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};

use hyper::StatusCode;
use serde_json::json;

use crate::{
    config::jwt::AuthConfig,
    domain::{
        auth::{usecase::login::LoginUserUsecase, utils::jwt::create_jwt},
        user::dto::request::LoginInfo,
    },
};

pub async fn login<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(auth_config): Extension<Arc<AuthConfig>>,
    Json(login_info): Json<LoginInfo>,
) -> impl IntoResponse
where
    T: LoginUserUsecase,
{
    // 입력값 검증
    let user_info = match usecase.login(login_info).await {
        Ok(info) => info,
        Err(e) => return e.as_ref().into_response(),
    };

    // 토큰 생성
    let access_token = create_jwt(
        user_info.get_id(),
        Some(user_info.get_username().to_string()),
        auth_config.get_access(),
        60, // 1시간
    )
    .unwrap();
    let refresh_token = create_jwt(
        user_info.get_id(),
        Some(user_info.get_username().to_string()),
        auth_config.get_refresh(),
        43200, // 30일
    )
    .unwrap();

    // 쿠키 추가
    let access_cookie = Cookie::build(("Authorization", format!("Bearer {}", access_token)))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict);
    let refresh_cookie = Cookie::build(("refresh", refresh_token))
        .path("/")
        .http_only(true);

    let mut response = Response::new(json!({"message": "성공"}).to_string());

    let headers = response.headers_mut();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );

    (StatusCode::OK, response).into_response()
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, sync::Arc};

    use axum::{async_trait, body::Body, extract::Request, routing::post, Extension, Router};
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::{to_string, Value};
    use tower::ServiceExt;

    use crate::{
        config::jwt::get_config,
        domain::{
            auth::usecase::login::LoginUserUsecase,
            user::dto::{
                request::{LoginInfo, LoginType},
                response::UserInfo,
            },
        },
        global::errors::CustomError,
    };

    use super::login;

    mock! {
        LoginUserUsecaseImpl {}

        #[async_trait]
        impl LoginUserUsecase for LoginUserUsecaseImpl {
            async fn login(&self, login_info: LoginInfo) -> Result<UserInfo, Arc<CustomError>>;
        }
    }

    fn _create_ok_mock(login_info: &LoginInfo, user_id: i32) -> MockLoginUserUsecaseImpl {
        let mut mock_usecase = MockLoginUserUsecaseImpl::new();
        mock_usecase
            .expect_login()
            .with(predicate::eq(login_info.clone()))
            .returning(move |info| {
                Ok(UserInfo::new(
                    user_id,
                    info.get_username().to_string(),
                    info.get_email().clone().unwrap_or("".to_string()),
                    info.get_nickname().clone().unwrap_or("".to_string()),
                    info.get_login_type().to_owned(),
                    None,
                    None,
                ))
            });
        mock_usecase
    }
    fn _create_err_mock(login_info: &LoginInfo, err: Arc<CustomError>) -> MockLoginUserUsecaseImpl {
        let mut mock_usecase = MockLoginUserUsecaseImpl::new();
        mock_usecase
            .expect_login()
            .with(predicate::eq(login_info.clone()))
            .returning(move |_| Err(err.clone()));
        mock_usecase
    }

    fn _create_app(mock_usecase: MockLoginUserUsecaseImpl) -> Router {
        Router::new()
            .route(
                "/api/v1/auth/login",
                post(login::<MockLoginUserUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(Arc::new(get_config())))
    }

    fn _create_req(login_info: &LoginInfo) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(to_string(&login_info).unwrap()))
            .unwrap()
    }

    #[tokio::test]
    async fn check_login_status() {
        // Arrange
        let username = "login_usecase@test.test";
        let user_id = 1;
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let mock_usecase = _create_ok_mock(&login_info, user_id);
        let app = _create_app(mock_usecase);
        let req = _create_req(&login_info);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_login_body() {
        // Arrange
        let username = "login_usecase@test.test";
        let user_id = 1;
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let mock_usecase = _create_ok_mock(&login_info, user_id);
        let app = _create_app(mock_usecase);
        let req = _create_req(&login_info);

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

        println!("ERROR : {}", &body_str);

        let body_json: Value = serde_json::from_str(&body_str).expect("failed to parse JSON");

        // Assert
        assert_eq!(body_json["message"], "성공");
        // assert!(body_json.get("accessToken").is_some());
        // assert!(body_json.get("refreshToken").is_some()) // 토큰 제공여부
    }

    #[tokio::test]
    async fn check_user_not_found() {
        // Arrange
        let username = "not_found_user@test.test";
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let mock_usecase = _create_err_mock(
            &login_info,
            Arc::new(CustomError::NotFound("User".to_string())),
        );
        let app = _create_app(mock_usecase);
        let req = _create_req(&login_info);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 404)
    }

    #[tokio::test]
    async fn check_login_cookie() {
        // Arrange
        let username = "login_usecase@test.test";
        let user_id = 1;
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let mock_usecase = _create_ok_mock(&login_info, user_id);
        let app = _create_app(mock_usecase);
        let req = _create_req(&login_info);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        let headers = response.headers();
        assert!(!headers.is_empty());
        assert!(headers.contains_key("set-cookie"));

        let set_cookies: HashSet<_> = headers
            .get_all("set-cookie")
            .into_iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();

        // let set_cookie = headers.get("set-cookie").unwrap().to_str().unwrap();
        assert_eq!(set_cookies.len(), 2);
        let cookie_str = set_cookies.iter().cloned().collect::<Vec<_>>().join("; ");
        assert!(cookie_str.contains("Bearer"));
        assert!(cookie_str.contains("Authorization"));
        assert!(cookie_str.contains("HttpOnly;"));
        assert!(cookie_str.contains("refresh"));
    }

    #[tokio::test]
    async fn check_password_incorrect() {
        // Arrange
        let username = "not_found_user@test.test";
        let login_info = LoginInfo::new(
            username.to_string(),
            "valid_pw".to_string(),
            LoginType::Email,
            None,
            None,
            None,
        );

        let mock_usecase = _create_err_mock(
            &login_info,
            Arc::new(CustomError::ValidationError("Password".to_string())),
        );
        let app = _create_app(mock_usecase);
        let req = _create_req(&login_info);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 400)
    }
}
