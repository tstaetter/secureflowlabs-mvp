use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub type AppResult<T> = Result<T, AppError>;
pub type SpecParsingResult<T> = Result<T, SpecParsingError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Error with MongoDB: {0}")]
    MongoDb(#[from] mongodb::error::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("DB error: {0}")]
    Db(#[from] DbError),
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Failed to connect to database: {0}")]
    Connection(#[from] mongodb::error::Error),
    #[error("Failed to insert new doc")]
    Insertion(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::MongoDb(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
        .into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SpecParsingError {
    #[error("Error deserializing input: {0}")]
    Serde(#[from] serde_json::error::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
