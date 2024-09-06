// Rust Base Library
use std::sync::Arc;

// Axum
use axum::Router;
use hyper::StatusCode;

// Env
use dotenv::dotenv;

// Logging
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// User Defined Modules
pub mod config {
    pub mod database;
}

pub mod global {
    pub mod utils {}
}

pub mod domain {
    pub mod book;
}

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

    let pool = config::database::create_connection_pool().await;
    let pool = Arc::new(pool);

    let book_router = book_router(pool.clone());

    let app = Router::new()
        .route("/", axum::routing::get(|| async { "{\"status\": \"OK\"}" }))
        .nest("/api/v1/book", book_router);

    let app =
        app.fallback(|| async { (StatusCode::NOT_FOUND, "존재하지 않는 API입니다.") });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
