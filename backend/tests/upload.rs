//! Integration tests for the `/upload` endpoint.
//!
//! Exercises multipart file upload, JSON validation, and the success /
//! error response paths.

mod common;

use axum::http::StatusCode;
use axum_test::multipart::MultipartForm;
use serde_json::Value;

/// A minimal but valid OpenAPI JSON payload used as test fixture.
const VALID_JSON: &str =
    r#"{"openapi":"3.0.3","info":{"title":"Test","version":"1.0"},"paths":{}}"#;

// ── Happy-path ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn upload_valid_json_returns_201() {
    let server = common::spawn_app().await;

    let form = MultipartForm::new().add_text("file", VALID_JSON);

    let response = server.post("/upload").multipart(form).await;

    assert_eq!(response.status_code(), StatusCode::CREATED);

    let body: Value = response.json();
    assert_eq!(body["status"], "ok");
    assert!(body["path"].as_str().unwrap().starts_with("tmp/uploads/"));
    assert!(body["filename"].as_str().unwrap().ends_with(".json"));
}

#[tokio::test]
async fn upload_valid_json_returns_expected_response_shape() {
    let server = common::spawn_app().await;

    let form = MultipartForm::new().add_text("file", VALID_JSON);

    let response = server.post("/upload").multipart(form).await;
    let body: Value = response.json();

    // Every field in UploadResponse must be present.
    assert!(body["status"].is_string());
    assert!(body["path"].is_string());
    assert!(body["filename"].is_string());

    // The filename must contain the original name we sent.
    // When using add_text, the server sees the content as raw bytes without a
    // file name, so it falls back to "upload.json".
    assert!(body["filename"].as_str().unwrap().contains("upload.json"));
}

// ── Error paths ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn upload_invalid_json_returns_400() {
    let server = common::spawn_app().await;

    let form = MultipartForm::new().add_text("file", "not valid json {{{");

    let response = server.post("/upload").multipart(form).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert!(response.text().contains("Invalid JSON"));
}

#[tokio::test]
async fn upload_missing_file_field_returns_400() {
    let server = common::spawn_app().await;

    // Send a multipart form with a field named "wrong_name" instead of "file".
    let form = MultipartForm::new().add_text("wrong_name", VALID_JSON);

    let response = server.post("/upload").multipart(form).await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert!(response.text().contains("No file field named 'file'"));
}
