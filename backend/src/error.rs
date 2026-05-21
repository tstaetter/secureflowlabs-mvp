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
    #[error("Upload error: {0}")]
    Upload(#[from] UploadError),
    #[error("Multipart error: {0}")]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error("Pipeline error: {0}")]
    Pipeline(#[from] PipelineError),
    #[error("Error parsing spec: {0}")]
    SpecParsing(#[from] SpecParsingError),
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Failed to connect to database: {0}")]
    Connection(#[from] mongodb::error::Error),
    #[error("Failed to insert new doc")]
    Insertion(String),
    #[error("Requested document not found: {0}")]
    NotFound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("No file field named '{0}' found in upload")]
    MissingField(String),
    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("Invalid YAML: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Failed to parse OpenAPI spec: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Failed to convert to schema: {0}")]
    Convert(String),
    #[error("DB operation failed: {0}")]
    Db(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::MongoDb(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Upload(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Multipart(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Pipeline(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::SpecParsing(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        }
        .into_response()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SpecParsingError {
    #[error("Error deserializing input: {0}")]
    SerdeJson(#[from] serde_json::error::Error),
    #[error("Error deserializing input: {0}")]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
