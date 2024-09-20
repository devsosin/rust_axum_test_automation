// Rust Base Library
use std::sync::Arc;

// Axum
use axum::Router;
use hyper::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Method, StatusCode,
};
use tower_http::cors::{AllowOrigin, CorsLayer};

// Env
use dotenv::dotenv;

// Logging
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// User Defined Modules
pub mod config {
    pub mod aws;
    pub mod database;
    pub mod jwt;
}

pub mod global {
    pub mod constants;
    pub mod errors;
    pub mod utils;
}

pub mod domain {
    pub mod auth;
    pub mod book;
    pub mod category;
    pub mod connect;
    pub mod image;
    pub mod record;
    pub mod user;
}

pub mod middleware {
    pub mod auth;
}

use crate::domain::{
    auth::route::get_router as auth_router, book::route::get_router as book_router,
    category::route::get_router as category_router, image::route::get_router as image_router,
    record::route::get_router as record_router, user::route::get_router as user_router,
};
use config::{aws::get_bucket, jwt::get_config};
use middleware::auth::verify;

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
    let aws_bucket = Arc::new(get_bucket());

    // public router
    let auth_router = auth_router(&pool, &auth_config);
    let public_router = Router::new().nest("/api/v1/auth", auth_router);

    // private router
    let book_router = book_router(&pool);
    let record_router = record_router(&pool);
    let user_router = user_router(&pool);
    let image_router = image_router(&pool, &aws_bucket);
    let category_router = category_router(&pool);

    let private_router = Router::new()
        .nest("/api/v1/book", book_router)
        .nest("/api/v1/record", record_router)
        .nest("/api/v1/user", user_router)
        .nest("/api/v1/image", image_router)
        .nest("/api/v1/category", category_router)
        .layer(axum::middleware::from_fn_with_state(auth_config, verify));

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact("http://localhost:5500".parse().unwrap())) // Replace with your frontend origin
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .allow_credentials(true); // Allow cookies and credentials

    let app = Router::new()
        .route("/", axum::routing::get(|| async { "{\"status\": \"OK\"}" }))
        .merge(public_router)
        .merge(private_router)
        .layer(cors); // Apply CORS layer

    let app =
        app.fallback(|| async { (StatusCode::NOT_FOUND, "존재하지 않는 API입니다.") });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
