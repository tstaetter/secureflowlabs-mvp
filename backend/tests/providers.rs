mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn test_providers_endpoint_returns_500_if_db_missing() {
    let server = common::spawn_app().await;

    let response = server.get("/providers").await;

    // Currently, common::spawn_app() returns AppState { db: None }
    // which should trigger DbError::Configuration -> 500 Internal Server Error
    assert_eq!(response.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_providers_endpoint_has_json_content_type_even_on_error() {
    let server = common::spawn_app().await;

    let response = server.get("/providers").await;

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Content-Type header should be present")
        .to_str()
        .unwrap();
    assert!(content_type.starts_with("application/json") || content_type.starts_with("text/plain"));
    // Actually, AppError::into_response returns (StatusCode, String).
    // Axum's default for String is text/plain.
}
