use crate::payloads::HealthResponse;
use axum::{http::StatusCode, response::Json};

/// Health check endpoint.
/// Returns 200 OK with `{"status":"ok"}`.
pub async fn health() -> (StatusCode, Json<HealthResponse>) {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header;

    #[tokio::test]
    async fn test_health_returns_200_ok() {
        let (status, _) = health().await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_returns_status_ok_in_body() {
        let (_, Json(body)) = health().await;
        assert_eq!(body.status, "ok");
    }

    #[tokio::test]
    async fn test_health_response_has_json_content_type() {
        // The Json extractor sets content-type automatically in responses
        let response = axum::response::IntoResponse::into_response(health().await);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("Content-Type header should be present")
            .to_str()
            .unwrap();
        assert!(content_type.starts_with("application/json"));
    }
}
