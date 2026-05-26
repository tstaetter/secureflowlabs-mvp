//! Integration test for the full pipeline:
//! Raw Schema → Normalized Endpoint → Capability Node
//!
//! This tests the end-to-end flow of ingesting an OpenAPI specification,
//! normalizing its endpoints, and inferring capability nodes from each one.

use backend::db::{Capability, HttpMethod, NormalizedEndpoint, RawSchema};
use backend::openapi::{ApiNormalizer, OpenApiNormalizer, infer_capability};
use mongodb::bson::oid::ObjectId;
use openapiv3::OpenAPI;

/// A minimal but realistic sample OpenAPI 3.0 specification, modeled after a
/// typical SaaS REST API (e.g., a subset of Stripe's customer + charge resources).
const SAMPLE_SPEC: &str = r#"
{
  "openapi": "3.0.3",
  "info": {
    "title": "Sample SaaS API",
    "version": "2025-01"
  },
  "paths": {
    "/v1/customers": {
      "post": {
        "summary": "Create a new customer",
        "operationId": "createCustomer",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "properties": {
                  "name": { "type": "string" },
                  "email": { "type": "string" }
                },
                "required": ["name", "email"]
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Customer created successfully"
          }
        }
      },
      "get": {
        "summary": "List all customers",
        "operationId": "listCustomers",
        "parameters": [
          {
            "name": "limit",
            "in": "query",
            "schema": { "type": "integer" }
          }
        ],
        "responses": {
          "200": {
            "description": "A list of customers"
          }
        }
      }
    },
    "/v1/customers/{id}": {
      "delete": {
        "summary": "Delete a customer",
        "operationId": "deleteCustomer",
        "parameters": [
          {
            "name": "id",
            "in": "path",
            "required": true,
            "schema": { "type": "string" }
          }
        ],
        "responses": {
          "200": {
            "description": "Customer deleted"
          }
        }
      }
    },
    "/v1/charges": {
      "post": {
        "summary": "Create a charge",
        "operationId": "createCharge",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": {
                "type": "object",
                "properties": {
                  "amount": { "type": "integer" },
                  "currency": { "type": "string" },
                  "customer": { "type": "string" }
                },
                "required": ["amount", "currency"]
              }
            }
          }
        },
        "responses": {
          "200": {
            "description": "Charge created"
          }
        }
      }
    }
  }
}
"#;

/// Returns a parsed `OpenAPI` struct from the inline sample spec.
fn parse_sample_spec() -> OpenAPI {
    serde_json::from_str::<OpenAPI>(SAMPLE_SPEC)
        .expect("sample spec must be valid OpenAPI 3.0 JSON")
}

/// Step 1: Parse the raw JSON string into an `openapiv3::OpenAPI` struct.
#[test]
fn step1_parse_raw_openapi_spec() {
    let spec = parse_sample_spec();

    assert_eq!(spec.info.title, "Sample SaaS API");
    assert_eq!(spec.info.version, "2025-01");
    // We expect 3 paths: /v1/customers, /v1/customers/{id}, /v1/charges
    assert_eq!(spec.paths.paths.len(), 3);
    assert!(spec.paths.paths.contains_key("/v1/customers"));
    assert!(spec.paths.paths.contains_key("/v1/customers/{id}"));
    assert!(spec.paths.paths.contains_key("/v1/charges"));
}

/// Step 2: Convert the parsed `OpenAPI` into a `RawSchema` via `TryFrom`.
#[test]
fn step2_convert_to_raw_schema() {
    let spec = parse_sample_spec();
    let raw_schema = RawSchema::try_from(spec).expect("OpenAPI must convert to RawSchema");

    assert_eq!(raw_schema.provider, "Sample SaaS API");
    assert_eq!(raw_schema.version, "2025-01");
    // The spec field is the original JSON re-serialized as a serde_json::Value.
    assert!(raw_schema.spec.is_object());
    assert_eq!(raw_schema.spec["info"]["title"], "Sample SaaS API");
}

/// Step 3: Normalize the `RawSchema` into `Vec<NormalizedEndpoint>` via `OpenApiNormalizer`.
///
/// This tests that every HTTP method on every path produces one `NormalizedEndpoint`.
#[test]
fn step3_normalize_to_endpoints() {
    let spec = parse_sample_spec();
    let provider = spec.info.title.clone();

    let normalizer = OpenApiNormalizer {
        provider: provider.clone(),
        spec,
    };

    let endpoints = normalizer.normalize().expect("normalization must succeed");

    // The sample spec has:
    //   POST /v1/customers
    //   GET  /v1/customers
    //   DELETE /v1/customers/{id}
    //   POST /v1/charges
    // = 4 total endpoints
    assert_eq!(endpoints.len(), 4);

    // Verify individual endpoints exist with the correct shape.
    let find = |method: HttpMethod, path: &str| -> Option<&NormalizedEndpoint> {
        endpoints
            .iter()
            .find(|e| e.method == method && e.path == path)
    };

    let post_customers =
        find(HttpMethod::Post, "/v1/customers").expect("POST /v1/customers must exist");
    assert!(post_customers.internal_id.starts_with(&provider));
    assert_eq!(
        post_customers.summary.as_deref(),
        Some("Create a new customer")
    );
    assert_eq!(
        post_customers.internal_id,
        "Sample SaaS API:Post:/v1/customers"
    );

    let get_customers =
        find(HttpMethod::Get, "/v1/customers").expect("GET /v1/customers must exist");
    assert_eq!(get_customers.summary.as_deref(), Some("List all customers"));
    assert_eq!(
        get_customers.internal_id,
        "Sample SaaS API:Get:/v1/customers"
    );

    let delete_customer = find(HttpMethod::Delete, "/v1/customers/{id}")
        .expect("DELETE /v1/customers/{id} must exist");
    assert_eq!(
        delete_customer.summary.as_deref(),
        Some("Delete a customer")
    );

    let post_charges = find(HttpMethod::Post, "/v1/charges").expect("POST /v1/charges must exist");
    assert_eq!(post_charges.summary.as_deref(), Some("Create a charge"));
}

/// Step 4: Infer capability nodes from each normalized endpoint.
///
/// This validates the business logic that maps endpoints onto named capabilities.
/// When Ollama is unavailable the capability names fall back to `"{verb}_unknown"`.
#[tokio::test]
async fn step4_infer_capabilities() {
    let spec = parse_sample_spec();
    let provider = spec.info.title.clone();

    let normalizer = OpenApiNormalizer {
        provider: provider.clone(),
        spec,
    };
    let endpoints = normalizer.normalize().expect("normalization must succeed");

    // Build capabilities from every endpoint.  Because we're not backed by a
    // real database, each capability gets a fresh ObjectId for its endpoint
    // foreign key — in production this would be the NormalizedEndpoint._id.
    let provider_name = provider.clone();
    let mut capabilities = Vec::new();
    for ep in &endpoints {
        let semantic_name = infer_capability(ep).await;
        let description = ep
            .summary
            .clone()
            .unwrap_or_else(|| format!("Handle {semantic_name}"));

        capabilities.push(Capability {
            id: None,
            semantic_name,
            description,
            endpoint_id: ObjectId::new(),
            tags: vec![provider_name.clone()],
        });
    }

    assert_eq!(capabilities.len(), 4, "one capability per endpoint");

    // Every capability must follow the "{verb}_{noun}" pattern.
    for cap in &capabilities {
        let parts: Vec<&str> = cap.semantic_name.splitn(2, '_').collect();
        assert_eq!(
            parts.len(),
            2,
            "name must be verb_noun: {}",
            cap.semantic_name
        );
        assert!(
            ["create", "get", "update", "delete"].contains(&parts[0]),
            "prefix must be a known verb: {}",
            parts[0]
        );
    }
}

/// Step 5: Full end-to-end pipeline — from raw JSON string to capability nodes in one shot.
#[tokio::test]
async fn step5_full_pipeline() {
    let spec: OpenAPI = serde_json::from_str(SAMPLE_SPEC).expect("valid OpenAPI JSON");

    // 1. RawSchema
    let raw = RawSchema::try_from(spec).expect("try_from OpenAPI");
    assert_eq!(raw.provider, "Sample SaaS API");

    // For the next steps, re-parse since `spec` was consumed by TryFrom.
    let spec2: OpenAPI = serde_json::from_str(SAMPLE_SPEC).expect("valid OpenAPI JSON");
    let provider = spec2.info.title.clone();

    // 2. Normalize
    let normalizer = OpenApiNormalizer {
        provider: provider.clone(),
        spec: spec2,
    };
    let endpoints = normalizer.normalize().expect("normalize");
    assert!(!endpoints.is_empty());

    // 3. Infer capabilities
    let provider_name = provider.clone();
    let mut capabilities = Vec::new();
    for ep in &endpoints {
        let name = infer_capability(ep).await;
        capabilities.push(Capability {
            id: None,
            semantic_name: name,
            description: ep.summary.clone().unwrap_or_default(),
            endpoint_id: ObjectId::new(),
            tags: vec![provider_name.clone()],
        });
    }

    // All endpoints have a corresponding capability.
    assert_eq!(capabilities.len(), endpoints.len());

    // Every capability should have a non-empty semantic name and a valid
    // ObjectId for its endpoint foreign key.
    for cap in &capabilities {
        assert!(!cap.semantic_name.is_empty());
        assert!(cap.endpoint_id != ObjectId::default());
    }

    // Every capability follows the "{verb}_{noun}" pattern.
    for cap in &capabilities {
        let parts: Vec<&str> = cap.semantic_name.splitn(2, '_').collect();
        assert_eq!(
            parts.len(),
            2,
            "name must be verb_noun: {}",
            cap.semantic_name
        );
    }
}
