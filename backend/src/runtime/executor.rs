use std::collections::HashMap;
use std::str::FromStr;

use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::runtime::ExecutionPlan;
use crate::{AppError, AppResult};

// ── Execution context ────────────────────────────────────────────────────────

/// Values injected into the HTTP request at execution time — API keys,
/// overrides, and runtime parameters that aren't part of the static plan.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Bearer token or other auth header value to inject.
    pub auth_header: Option<String>,
    /// Additional headers to merge with the plan's static headers.
    pub extra_headers: HashMap<String, String>,
    /// Query parameter overrides (key → value).
    pub query_overrides: HashMap<String, String>,
}

// ── Execution result ─────────────────────────────────────────────────────────

/// The outcome of executing a single `ExecutionPlan`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Parsed JSON body (or `null` if the response wasn't JSON).
    pub body: serde_json::Value,
    /// Wall-clock time for the final successful attempt (ms).
    pub elapsed_ms: u64,
    /// Number of retries before succeeding (0 = first attempt).
    pub retries: u8,
}

// ── Executor trait ───────────────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait Executor {
    async fn execute(
        &self,
        plan: ExecutionPlan,
        context: ExecutionContext,
    ) -> AppResult<ExecutionResult>;
}

// ── HTTP executor ────────────────────────────────────────────────────────────

pub struct HttpExecutor {
    client: Client,
}

impl HttpExecutor {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for HttpExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Executor for HttpExecutor {
    async fn execute(
        &self,
        plan: ExecutionPlan,
        context: ExecutionContext,
    ) -> AppResult<ExecutionResult> {
        let request = build_request(&self.client, &plan, &context)?;

        let (response, retries) = execute_with_retry(&self.client, request, &plan.retry).await?;

        normalize_response(response, retries).await
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a `reqwest::Request` from the plan and execution context.
fn build_request(
    client: &Client,
    plan: &ExecutionPlan,
    context: &ExecutionContext,
) -> AppResult<reqwest::Request> {
    let method = method_from_http_method(&plan.request.method);

    let mut headers = HeaderMap::new();

    // Inject auth header from context if present.
    if let Some(ref auth) = context.auth_header {
        headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_str(auth).map_err(|e| {
                AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
            })?,
        );
    }

    // Merge plan-level headers.
    for name in &plan.request.headers {
        // Split "Key: Value" or use key-only (no value).
        if let Some((k, v)) = name.split_once(':') {
            headers.insert(
                HeaderName::from_str(k.trim()).map_err(|e| {
                    AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
                })?,
                HeaderValue::from_str(v.trim()).map_err(|e| {
                    AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
                })?,
            );
        }
    }

    // Merge context-level extra headers (override plan headers).
    for (k, v) in &context.extra_headers {
        headers.insert(
            HeaderName::from_str(k).map_err(|e| {
                AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
            })?,
            HeaderValue::from_str(v).map_err(|e| {
                AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e))
            })?,
        );
    }

    // Build query params from context overrides.
    let query: Vec<(&str, &str)> = context
        .query_overrides
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let request = client
        .request(method, &plan.request.url)
        .headers(headers)
        .query(&query)
        .build()
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, e)))?;

    Ok(request)
}

/// Execute the request with retry logic. Returns the successful response
/// and the number of retries attempted.
async fn execute_with_retry(
    client: &Client,
    request: reqwest::Request,
    retry_policy: &crate::runtime::RetryPolicy,
) -> AppResult<(reqwest::Response, u8)> {
    let mut retries: u8 = 0;
    let max_retries = retry_policy.max_retries;

    loop {
        let response = client
            .execute(request.try_clone().ok_or_else(|| {
                AppError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "request body is not cloneable",
                ))
            })?)
            .await
            .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let status = response.status().as_u16();

        // If the status isn't in the retry-on list, or we're out of retries, return.
        if !retry_policy.retry_on_status.contains(&status) || retries >= max_retries {
            return Ok((response, retries));
        }

        retries += 1;

        tokio::time::sleep(std::time::Duration::from_millis(retry_policy.backoff_ms)).await;
    }
}

/// Convert a `reqwest::Response` into an `ExecutionResult`.
async fn normalize_response(
    response: reqwest::Response,
    retries: u8,
) -> AppResult<ExecutionResult> {
    let status = response.status().as_u16();

    let headers: HashMap<String, String> = response
        .headers()
        .iter()
        .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let body: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);

    Ok(ExecutionResult {
        status,
        headers,
        body,
        elapsed_ms: 0,
        retries,
    })
}

/// Map our internal `HttpMethod` to `reqwest::Method`.
fn method_from_http_method(method: &crate::db::HttpMethod) -> reqwest::Method {
    match method {
        crate::db::HttpMethod::Get => reqwest::Method::GET,
        crate::db::HttpMethod::Post => reqwest::Method::POST,
        crate::db::HttpMethod::Put => reqwest::Method::PUT,
        crate::db::HttpMethod::Delete => reqwest::Method::DELETE,
        crate::db::HttpMethod::Patch => reqwest::Method::PATCH,
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_context_default_is_empty() {
        let ctx = ExecutionContext::default();
        assert!(ctx.auth_header.is_none());
        assert!(ctx.extra_headers.is_empty());
        assert!(ctx.query_overrides.is_empty());
    }

    #[test]
    fn execution_result_serializes() {
        let result = ExecutionResult {
            status: 200,
            headers: HashMap::from([("content-type".into(), "application/json".into())]),
            body: serde_json::json!({"ok": true}),
            elapsed_ms: 42,
            retries: 0,
        };

        let json = serde_json::to_string(&result).expect("serialize");
        let parsed: ExecutionResult = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.status, 200);
        assert_eq!(parsed.retries, 0);
    }

    #[test]
    fn method_mapping_covers_all_variants() {
        use crate::db::HttpMethod;
        for method in &[
            HttpMethod::Get,
            HttpMethod::Post,
            HttpMethod::Put,
            HttpMethod::Delete,
            HttpMethod::Patch,
        ] {
            let _ = method_from_http_method(method);
        }
    }
}
