use axum::response::IntoResponse;

#[derive(Debug, thiserror::Error)]
pub enum AppError {}

impl IntoResponse for AppError {}
