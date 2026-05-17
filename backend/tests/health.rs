mod common;

use axum::http::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn test_health_endpoint_returns_200() {
    let server = common::spawn_app().await;

    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
async fn test_health_endpoint_returns_status_ok() {
    let server = common::spawn_app().await;

    let response = server.get("/health").await;

    let body: Value = response.json();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_health_endpoint_has_json_content_type() {
    let server = common::spawn_app().await;

    let response = server.get("/health").await;

    let content_type = response
        .headers()
        .get("content-type")
        .expect("Content-Type header should be present")
        .to_str()
        .unwrap();
    assert!(content_type.starts_with("application/json"));
}

#[tokio::test]
async fn test_health_endpoint_rejects_post() {
    let server = common::spawn_app().await;

    let response = server.post("/health").await;

    // A POST to a GET-only route should return 405 Method Not Allowed
    assert_eq!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);
}
