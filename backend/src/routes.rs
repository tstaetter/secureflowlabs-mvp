use crate::{health, providers, upload, AppState};
use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
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
        .route("/providers", get(providers))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state))
}
