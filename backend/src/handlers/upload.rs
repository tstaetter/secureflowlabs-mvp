use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{AppError, AppResult, AppState, UploadError};
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use serde::Serialize;
use tracing::info;

/// Returned to the client after a successful upload.
#[derive(Serialize)]
pub struct UploadResponse {
    status: String,
    path: String,
    filename: String,
}

/// Upload a JSON file via multipart form data.
///
/// Expects a single file in a field named `"file"`.  The file contents are
/// validated as JSON, then persisted under `tmp/uploads/` with a
/// timestamp-prefixed filename to avoid collisions.
pub async fn upload(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, axum::Json<UploadResponse>)> {
    let upload_dir = PathBuf::from("tmp/uploads");
    tokio::fs::create_dir_all(&upload_dir).await?;

    let mut saved_path: Option<PathBuf> = None;
    let mut saved_filename: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(AppError::Multipart)? {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }

        let original_name = field.file_name().unwrap_or("upload.json").to_string();

        let data = field.bytes().await?;

        // Validate that the uploaded payload is well-formed JSON.
        serde_json::from_slice::<serde_json::Value>(&data)
            .map_err(|e| AppError::Upload(UploadError::InvalidJson(e)))?;

        // Generate a unique filename: <timestamp_ms>-<original_name>
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let filename = format!("{ts}-{original_name}");
        let dest = upload_dir.join(&filename);

        tokio::fs::write(&dest, &data).await?;

        saved_path = Some(dest);
        saved_filename = Some(filename);
        break; // Only process the first matching file field.
    }

    match (saved_path, saved_filename) {
        (Some(path), Some(filename)) => {
            info!("Saved uploaded spec to {}", path.display());
            Ok((
                StatusCode::CREATED,
                axum::Json(UploadResponse {
                    status: "ok".into(),
                    path: path.display().to_string(),
                    filename,
                }),
            ))
        }
        _ => Err(AppError::Upload(UploadError::MissingField("file".into()))),
    }
}
