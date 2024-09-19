use anyhow::Error as AnyhowError;
use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use sqlx::Error as SqlxError;

#[derive(Debug)]
pub enum CustomError {
    NotFound(String),
    DatabaseError(SqlxError),
    ValidationError(String),
    Unauthorized(String),
    Unexpected(AnyhowError),
    Duplicated(String),
    NoFieldUpdate(String),
}

impl From<SqlxError> for CustomError {
    fn from(err: SqlxError) -> Self {
        CustomError::DatabaseError(err)
    }
}

impl From<AnyhowError> for CustomError {
    fn from(err: AnyhowError) -> Self {
        CustomError::Unexpected(err)
    }
}

impl IntoResponse for &CustomError {
    fn into_response(self) -> Response {
        match self {
            CustomError::NotFound(t) => {
                (StatusCode::NOT_FOUND, format!("{} not found", t)).into_response()
            }
            CustomError::DatabaseError(_) => {
                (StatusCode::BAD_REQUEST, "Database error").into_response()
            }
            CustomError::ValidationError(_) => {
                (StatusCode::BAD_REQUEST, "Validation failed").into_response()
            }
            CustomError::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, "Authorization failed").into_response()
            }
            CustomError::Unexpected(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error").into_response()
            }
            CustomError::Duplicated(t) => {
                (StatusCode::BAD_REQUEST, format!("Duplicated {}", t)).into_response()
            }
            CustomError::NoFieldUpdate(_) => {
                (StatusCode::BAD_REQUEST, "No field to update").into_response()
            }
        }
    }
}
