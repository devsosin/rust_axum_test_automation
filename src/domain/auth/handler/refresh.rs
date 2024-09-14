use std::sync::Arc;

use axum::{
    response::{IntoResponse, Response},
    Extension,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use hyper::{header, StatusCode};
use serde_json::json;

use crate::{
    config::jwt::AuthConfig,
    domain::auth::{
        usecase::refresh::RefreshTokenUsecase,
        utils::jwt::{create_jwt, decode_jwt},
    },
    global::errors::CustomError,
};

pub async fn refresh_token<T>(
    Extension(usecase): Extension<Arc<T>>,
    Extension(auth_config): Extension<Arc<AuthConfig>>,
    cookie_jar: CookieJar,
) -> impl IntoResponse
where
    T: RefreshTokenUsecase,
{
    let token = cookie_jar
        .get("refresh")
        .map(|cookie| cookie.value().to_string());

    let token = if let Some(token) = token {
        token
    } else {
        return CustomError::Unauthorized("Refresh".to_string()).into_response();
    };

    let claims = match decode_jwt(&token, &auth_config.get_refresh()) {
        Ok(r) => r,
        Err(_) => return CustomError::Unauthorized("Refresh".to_string()).into_response(),
    };

    let id = claims.sub;

    let user_info = match usecase.refresh(id).await {
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

    use axum::{
        async_trait, body::Body, extract::Request, http::HeaderValue, routing::post, Extension,
        Router,
    };
    use http_body_util::BodyExt;
    use mockall::{mock, predicate};
    use serde_json::Value;
    use tower::ServiceExt;

    use crate::{
        config::jwt::get_config,
        domain::{
            auth::{usecase::refresh::RefreshTokenUsecase, utils::jwt::create_jwt},
            user::dto::{request::LoginType, response::UserInfo},
        },
        global::errors::CustomError,
    };

    use super::refresh_token;

    mock! {
        RefreshTokenUsecaseImpl {}

        #[async_trait]
        impl RefreshTokenUsecase for RefreshTokenUsecaseImpl {
            async fn refresh(&self, id: i32) -> Result<UserInfo, Arc<CustomError>>;
        }
    }

    fn _create_app(id: i32) -> Router {
        let mut mock_usecase = MockRefreshTokenUsecaseImpl::new();
        mock_usecase
            .expect_refresh()
            .with(predicate::eq(id))
            .returning(|i| {
                Ok(UserInfo::new(
                    i,
                    "test_username".to_string(),
                    "test_email".to_string(),
                    "test_nickname".to_string(),
                    LoginType::Email,
                    None,
                    None,
                ))
            });

        Router::new()
            .route(
                "/api/v1/auth/refresh",
                post(refresh_token::<MockRefreshTokenUsecaseImpl>),
            )
            .layer(Extension(Arc::new(mock_usecase)))
            .layer(Extension(Arc::new(get_config())))
    }

    fn _create_req(refresh: &str) -> Request {
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/refresh")
            .header(
                "Cookie",
                HeaderValue::from_str(&("refresh=".to_string() + refresh)).unwrap(),
            )
            .body(Body::empty())
            .unwrap()
    }

    #[tokio::test]
    async fn check_refresh_token_status() {
        // Arrange
        let id = 1;
        let refresh = create_jwt(id, None, get_config().get_refresh(), 76400).unwrap();

        let app = _create_app(id);
        let req = _create_req(&refresh);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 200)
    }

    #[tokio::test]
    async fn check_refresh_token_body() {
        // Arrange
        let id = 1;
        let refresh = create_jwt(id, None, get_config().get_refresh(), 76400).unwrap();

        let app = _create_app(id);
        let req = _create_req(&refresh);

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
    async fn check_refresh_token_cookie() {
        // Arrange
        let id = 1;
        let refresh = create_jwt(id, None, get_config().get_refresh(), 76400).unwrap();

        let app = _create_app(id);
        let req = _create_req(&refresh);

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
    async fn check_refresh_token_not_valid() {
        // Arrange
        let id = 1;
        let refresh = create_jwt(id, None, "invalid-secret-code", 3600).unwrap();

        let app = _create_app(id);
        let req = _create_req(&refresh);

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }

    #[tokio::test]
    async fn check_cookie_not_found() {
        // Arrange
        let id = 1;

        let app = _create_app(id);
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/auth/refresh")
            .body(Body::empty())
            .unwrap();

        // Act
        let response = app.oneshot(req).await.unwrap();

        // Assert
        assert_eq!(response.status(), 401)
    }
}
