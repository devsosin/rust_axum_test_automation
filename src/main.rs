// Rust Base Library
use std::sync::Arc;

// Axum
use axum::Router;
use config::jwt::get_config;
use hyper::StatusCode;

// Env
use dotenv::dotenv;

use middleware::auth::verify;
// Logging
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// User Defined Modules
pub mod config {
    pub mod database;
    pub mod jwt;
}

pub mod global {
    pub mod utils {}
    pub mod constants;
    pub mod errors;
}

pub mod domain {
    pub mod book;
    pub mod record;
    pub mod user;
}

pub mod middleware {
    pub mod auth;
}

use crate::domain::{
    book::route::get_router as book_router,
    record::route::get_router as record_router,
    user::route::{
        get_private_router as user_private_router, get_public_router as user_public_router,
    },
};

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
    let auth_config = Arc::new(get_config());

    // public router
    let user_public_router = user_public_router(&pool, &auth_config);
    let public_router = Router::new().nest("/api/v1/user", user_public_router);

    // private router
    let book_router = book_router(&pool);
    let record_router = record_router(&pool);
    let user_private_router = user_private_router(&pool);

    let private_router = Router::new()
        .nest("/api/v1/book", book_router)
        .nest("/api/v1/record", record_router)
        .nest("/api/v1/user", user_private_router)
        .layer(axum::middleware::from_fn_with_state(auth_config, verify));

    let app = Router::new()
        .route("/", axum::routing::get(|| async { "{\"status\": \"OK\"}" }))
        .merge(public_router)
        .merge(private_router);

    let app =
        app.fallback(|| async { (StatusCode::NOT_FOUND, "존재하지 않는 API입니다.") });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
