use crate::{AppState, health, upload};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/upload", post(upload))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state))
}
