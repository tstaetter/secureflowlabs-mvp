use crate::{health, AppState};
use axum::{routing::get, Router};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .with_state(state)
}
