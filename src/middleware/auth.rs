use std::sync::Arc;

use axum::{
    body::Body, extract::State, http::Request, middleware::Next, response::IntoResponse, Json,
};
use axum_extra::extract::CookieJar;
use hyper::{header, StatusCode};
use serde::Serialize;

use crate::{config::jwt::AuthConfig, domain::auth::utils::jwt::decode_jwt};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

pub async fn verify(
    cookie_jar: CookieJar,
    State(config): State<Arc<AuthConfig>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let token = cookie_jar
        .get("Authorization")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let token = token.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                message: "토큰 검증 실패".to_string(),
            }),
        )
    })?;

    let claims = decode_jwt(&token, &config.get_access()).map_err(|e| {
        let err_msg = format!("Error(Verify): {:?}", &e);
        tracing::error!("{}", err_msg);

        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                message: "토큰 검증 실패".to_string(),
            }),
        )
    })?;

    let user_id = claims.sub;

    req.extensions_mut().insert(user_id);
    Ok(next.run(req).await)
}
