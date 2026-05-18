use crate::{AppState, health, upload};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/upload",
            post(upload).layer(DefaultBodyLimit::max(50 * 1024 * 1024)),
        )
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state))
}
