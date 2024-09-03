// Rust Base Library
use std::time::Duration;

// Axum
use axum::{
    extract::Extension, 
    http::StatusCode, 
    routing::get, 
    Router
};

// Env
use dotenv::dotenv;

// Logging
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Database
use sqlx::{postgres::PgPoolOptions, PgPool};

// User Defined Modules
mod domain;

use crate::domain::book::route::get_router as book_router;

#[tokio::main]
async fn main() {
    // 환경변수 로드
    dotenv().ok();
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("set DATABASE_URL env variable");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .expect("unable to make connections");

    let book_router = book_router();

    let app = Router::new()
        .route("/", get(root))
        .nest("/api/v1/book", book_router)
        .layer(Extension(pool));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn root(Extension(pool): Extension<PgPool>) -> &'static str {
    let category_checks = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM tb_base_category WHERE id = $1",
    )
    .bind("2")
    .fetch_one(&pool)
    .await
    .map_err(|err| {
        let err_message = format!("Error check {}", err);
        tracing::error!("{}", &err_message);
        err_message
    }).unwrap();
    // 아.. ?하면 return Err() 감싸줌 (StatusCode, String -> msg)

    tracing::debug!("category check: {}", category_checks);

    "Hello, World!"
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
// map_err
// 예외처리들
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}