//! Data pipeline: Raw OpenAPI spec → RawSchema → NormalizedEndpoint → Capability.
//!
//! Orchestrates the full ingestion flow for an uploaded OpenAPI specification.

use crate::db::{AppDatabase, Capability, Model, NormalizedEndpoint, RawSchema};
use crate::openapi::{ApiNormalizer, OpenApiNormalizer, get_raw_spec, infer_capability};
use crate::{AppError, AppResult, PipelineError};
use serde::Serialize;
use tracing::info;

/// Summary returned after a successful pipeline run.
#[derive(Debug, Serialize)]
pub struct PipelineResult {
    pub provider: String,
    pub version: String,
    pub endpoints_created: usize,
    pub capabilities_created: usize,
}

/// Run the full ingestion pipeline on raw OpenAPI JSON bytes.
///
/// # Steps
/// 1. Parse file → `openapiv3::OpenAPI`
/// 2. Convert → `RawSchema` and persist in MongoDB
/// 3. Normalize → `Vec<NormalizedEndpoint>` and persist each
/// 4. Infer a `Capability` for each endpoint and persist
///
/// Returns a `PipelineResult` with counts of what was created.
pub async fn run_pipeline(db: &AppDatabase, path: &str) -> AppResult<PipelineResult> {
    // ── Step 1: Parse the raw JSON into an OpenAPI struct ────────────────
    let spec = get_raw_spec(path).await?;
    let provider = spec.info.title.clone();
    let version = spec.info.version.clone();

    // ── Step 2: Persist the raw schema ───────────────────────────────────
    let raw_schema = RawSchema::try_from(spec.clone())
        .map_err(|e| AppError::Pipeline(PipelineError::Convert(e.to_string())))?;

    let _raw = db
        .insert_one(&raw_schema)
        .await
        .map_err(|e| AppError::Pipeline(PipelineError::Db(e.to_string())))?;

    info!(
        "Ingested raw schema for provider '{}' v{}",
        provider, version
    );

    let normalizer = OpenApiNormalizer {
        provider: provider.clone(),
        spec: spec,
    };

    let endpoints = normalizer.normalize()?;

    let mut persisted_endpoints: Vec<NormalizedEndpoint> = Vec::with_capacity(endpoints.len());
    for ep in &endpoints {
        let created = db
            .insert_one(ep)
            .await
            .map_err(|e| AppError::Pipeline(PipelineError::Db(e.to_string())))?;
        persisted_endpoints.push(created);
    }

    info!(
        "Normalized {} endpoints for '{}'",
        endpoints.len(),
        provider
    );

    // ── Step 4: Infer capabilities and persist each ──────────────────────
    let mut capabilities_created = 0usize;
    for ep in &persisted_endpoints {
        let semantic_name = infer_capability(ep);
        let description = ep
            .summary
            .clone()
            .unwrap_or_else(|| format!("Handle {semantic_name}"));

        let capability = Capability {
            id: None,
            semantic_name,
            description,
            endpoint_id: ep.get_id(),
            tags: vec![provider.clone()],
        };

        db.insert_one(&capability)
            .await
            .map_err(|e| AppError::Pipeline(PipelineError::Db(e.to_string())))?;

        capabilities_created += 1;
    }

    info!(
        "Inferred {} capabilities for '{}'",
        capabilities_created, provider
    );

    Ok(PipelineResult {
        provider,
        version,
        endpoints_created: endpoints.len(),
        capabilities_created,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use openapiv3::OpenAPI;

    const MINIMAL_SPEC: &str = r#"{
        "openapi": "3.0.3",
        "info": { "title": "Pipeline Test API", "version": "1.0" },
        "paths": {
            "/v1/customers": {
                "post": { "summary": "Create a customer", "responses": { "200": { "description": "ok" } } }
            },
            "/v1/customers/{id}": {
                "delete": { "summary": "Delete a customer", "responses": { "200": { "description": "ok" } } }
            }
        }
    }"#;

    #[test]
    fn parse_valid_spec_succeeds() {
        let spec: OpenAPI =
            serde_json::from_str(MINIMAL_SPEC).expect("valid OpenAPI JSON must parse");
        assert_eq!(spec.info.title, "Pipeline Test API");
        assert_eq!(spec.paths.paths.len(), 2);
    }

    #[test]
    fn parse_invalid_json_fails() {
        let result = serde_json::from_slice::<OpenAPI>(b"not json");
        assert!(result.is_err());
    }

    #[test]
    fn raw_schema_try_from_round_trips() {
        let spec: OpenAPI = serde_json::from_str(MINIMAL_SPEC).expect("valid OpenAPI JSON");
        let raw = RawSchema::try_from(spec).expect("try_from must succeed");
        assert_eq!(raw.provider, "Pipeline Test API");
        assert_eq!(raw.version, "1.0");
        assert!(raw.spec.is_object());
    }

    #[test]
    fn normalization_produces_endpoints() {
        let spec: OpenAPI = serde_json::from_str(MINIMAL_SPEC).expect("valid OpenAPI JSON");
        let provider = spec.info.title.clone();

        let normalizer = OpenApiNormalizer { provider, spec };
        let endpoints = normalizer.normalize().expect("normalize");

        // POST /v1/customers + DELETE /v1/customers/{id} = 2 endpoints
        assert_eq!(endpoints.len(), 2);

        let post = endpoints
            .iter()
            .find(|e| e.path == "/v1/customers")
            .expect("POST /v1/customers");
        assert_eq!(post.summary.as_deref(), Some("Create a customer"));

        let delete = endpoints
            .iter()
            .find(|e| e.path == "/v1/customers/{id}")
            .expect("DELETE /v1/customers/{id}");
        assert_eq!(delete.summary.as_deref(), Some("Delete a customer"));
    }

    #[test]
    fn capabilities_inferred_correctly() {
        let spec: OpenAPI = serde_json::from_str(MINIMAL_SPEC).expect("valid OpenAPI JSON");
        let provider = spec.info.title.clone();

        let normalizer = OpenApiNormalizer { provider, spec };
        let endpoints = normalizer.normalize().expect("normalize");

        let names: Vec<String> = endpoints.iter().map(|ep| infer_capability(ep)).collect();

        // POST /v1/customers → "create_customer", DELETE → "delete_resource"
        assert!(names.contains(&"create_customer".to_string()));
        assert!(names.contains(&"delete_customer".to_string()));
    }
}
