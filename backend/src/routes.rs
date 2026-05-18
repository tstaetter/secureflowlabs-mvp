use crate::{AppState, health, upload};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/upload", post(upload))
        .with_state(Arc::new(state))
}
