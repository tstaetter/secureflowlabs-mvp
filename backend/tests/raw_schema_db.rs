//! Integration tests for persisting `RawSchema` documents in MongoDB.
//!
//! These tests use the real MongoDB connection (defaulting to
//! `mongodb://localhost:27017/filez_zone_dev`) and write to the
//! `test_schemas` collection so production data is never touched.
//!
//! When MongoDB is not reachable the tests that require it are skipped
//! gracefully instead of failing.

use backend::db::{AppDatabase, Capability, Model, NormalizedEndpoint, RawSchema, SchemaSource};
use backend::pipeline::run_pipeline;
use mongodb::Client;
use mongodb::Collection;
use mongodb::bson::doc;
use openapiv3::OpenAPI;
use std::time::Duration;

/// The same sample OpenAPI spec used throughout the test suite.
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
    }
  }
}
"#;

/// Build a `RawSchema` from the inline sample spec.
fn build_sample_raw_schema() -> RawSchema {
    let spec: OpenAPI =
        serde_json::from_str(SAMPLE_SPEC).expect("sample spec must be valid OpenAPI 3.0 JSON");
    RawSchema::try_from(spec).expect("OpenAPI must convert to RawSchema")
}

/// Delete every document in the `test_schemas` collection whose provider
/// matches the given string.  Used for test cleanup so repeated runs are
/// idempotent.
async fn purge_by_provider(db: &AppDatabase, provider: &str) {
    let collection: Collection<RawSchema> = db.database.collection(RawSchema::COLLECTION);
    let _ = collection.delete_many(doc! { "provider": provider }).await;
}

/// Probe whether a MongoDB instance is reachable within a short timeout.
///
/// Returns `Some(AppDatabase)` on success so callers can reuse the connection.
async fn try_connect() -> Option<AppDatabase> {
    let mongo_uri =
        std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
    let mongo_db = std::env::var("MONGODB_NAME").unwrap_or_else(|_| "filez_zone_dev".to_string());

    let client = tokio::time::timeout(Duration::from_secs(3), Client::with_uri_str(&mongo_uri))
        .await
        .ok()?
        .ok()?;

    // Run a cheap command to confirm the server is alive.
    let db = client.database(&mongo_db);
    tokio::time::timeout(Duration::from_secs(3), db.run_command(doc! { "ping": 1 }))
        .await
        .ok()?
        .ok()?;

    Some(AppDatabase { database: db })
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// Insert a single `RawSchema` and verify the database assigns an `_id` and
/// round-trips every field correctly.
#[tokio::test]
async fn insert_single_raw_schema() {
    let db = match try_connect().await {
        Some(db) => db,
        None => {
            eprintln!("SKIP: MongoDB not reachable");
            return;
        }
    };

    let raw = build_sample_raw_schema();
    let provider = raw.provider.clone();

    // Clean up any leftover from a previous run.
    purge_by_provider(&db, &provider).await;

    // ── Act ──────────────────────────────────────────────────────────────
    let created = db.insert_one(&raw).await.expect("insert_one must succeed");

    // ── Assert ───────────────────────────────────────────────────────────
    // The database must have assigned a real ObjectId.
    assert!(created.id.is_some(), "inserted document must have an _id");

    // Every field must round-trip correctly.
    assert_eq!(created.provider, "Sample SaaS API");
    assert_eq!(created.version, "2025-01");
    assert!(matches!(created.source, SchemaSource::OpenApiJson));
    assert!(created.spec.is_object());
    assert_eq!(created.spec["info"]["title"], "Sample SaaS API");
    assert_eq!(created.spec["paths"].as_object().unwrap().len(), 2);

    // ── Cleanup ──────────────────────────────────────────────────────────
    purge_by_provider(&db, &provider).await;
}

/// Insert multiple `RawSchema` documents (different providers) and verify
/// they each receive distinct `_id` values.
#[tokio::test]
async fn insert_multiple_raw_schemas() {
    let db = match try_connect().await {
        Some(db) => db,
        None => {
            eprintln!("SKIP: MongoDB not reachable");
            return;
        }
    };

    // Build two schemas with distinct providers so they don't collide.
    let mut raw_a = build_sample_raw_schema();
    raw_a.provider = "integration-test-provider-a".into();

    let mut raw_b = build_sample_raw_schema();
    raw_b.provider = "integration-test-provider-b".into();
    raw_b.version = "2025-02".into();

    // Clean up.
    purge_by_provider(&db, &raw_a.provider).await;
    purge_by_provider(&db, &raw_b.provider).await;

    // Insert both.
    let created_a = db.insert_one(&raw_a).await.expect("insert a");
    let created_b = db.insert_one(&raw_b).await.expect("insert b");

    // Each document gets its own ObjectId.
    assert_ne!(created_a.id, created_b.id);

    // Provider-level fields are independent.
    assert_eq!(created_a.provider, "integration-test-provider-a");
    assert_eq!(created_a.version, "2025-01");
    assert_eq!(created_b.provider, "integration-test-provider-b");
    assert_eq!(created_b.version, "2025-02");

    // Both carry the same underlying spec paths.
    for created in &[&created_a, &created_b] {
        assert_eq!(created.spec["paths"].as_object().unwrap().len(), 2);
    }

    // Clean up.
    purge_by_provider(&db, &raw_a.provider).await;
    purge_by_provider(&db, &raw_b.provider).await;
}

/// Verify that inserting a `RawSchema` with an explicit YAML source type is
/// persisted correctly.
#[tokio::test]
async fn insert_raw_schema_with_yaml_source() {
    let db = match try_connect().await {
        Some(db) => db,
        None => {
            eprintln!("SKIP: MongoDB not reachable");
            return;
        }
    };

    let mut raw = build_sample_raw_schema();
    raw.provider = "integration-test-yaml-source".into();
    raw.source = SchemaSource::OpenApiYaml;

    purge_by_provider(&db, &raw.provider).await;

    let created = db
        .insert_one(&raw)
        .await
        .expect("insert yaml-source schema");

    assert!(matches!(created.source, SchemaSource::OpenApiYaml));

    purge_by_provider(&db, &raw.provider).await;
}

/// Verify that inserting into an unreachable database returns an error
/// within a reasonable timeout (controlled via `serverSelectionTimeoutMS`).
#[tokio::test]
async fn insert_fails_when_db_unreachable() {
    // Use a connection-string-level timeout so the driver doesn't hang.
    let uri = "mongodb://invalid-host:27017/?serverSelectionTimeoutMS=2000";

    // SAFETY: single-threaded test; we restore immediately after.
    unsafe {
        std::env::set_var("MONGODB_URI", uri);
    }

    let db = AppDatabase::try_new()
        .await
        .expect("client creation is lazy — always succeeds");

    unsafe {
        std::env::remove_var("MONGODB_URI");
    }

    let raw = build_sample_raw_schema();
    let result = db.insert_one(&raw).await;

    assert!(
        result.is_err(),
        "insert_one must fail when the database host is unreachable"
    );
}

// ── Full pipeline integration ───────────────────────────────────────────────

/// Run the full ingestion pipeline against a real database and verify every
/// stage writes to the correct collections.
#[tokio::test]
async fn run_pipeline_end_to_end() {
    let db = match try_connect().await {
        Some(db) => db,
        None => {
            eprintln!("SKIP: MongoDB not reachable");
            return;
        }
    };

    let spec_json = r#"{
        "openapi": "3.0.3",
        "info": { "title": "Pipeline Integration Test", "version": "0.1" },
        "paths": {
            "/v1/items": {
                "post": { "summary": "Create item", "responses": { "200": { "description": "ok" } } },
                "get":  { "summary": "List items",  "responses": { "200": { "description": "ok" } } }
            }
        }
    }"#;

    // Write spec to a temp file so run_pipeline can read it from disk.
    let tmp_dir = std::path::PathBuf::from("tmp/uploads");
    tokio::fs::create_dir_all(&tmp_dir)
        .await
        .expect("create tmp dir");
    let file_path = tmp_dir.join("pipeline-integration-test.json");
    tokio::fs::write(&file_path, spec_json)
        .await
        .expect("write temp spec");

    let provider = "Pipeline Integration Test";

    // Clean up any remnants from a previous run.
    purge_by_provider(&db, provider).await;

    // ── Act ────────────────────────────────────────────────────────────
    let result = run_pipeline(&db, file_path.to_str().unwrap())
        .await
        .expect("pipeline must succeed");

    // ── Assert ─────────────────────────────────────────────────────────
    assert_eq!(result.provider, provider);
    assert_eq!(result.version, "0.1");
    assert_eq!(result.endpoints_created, 2); // POST + GET on /v1/items
    assert_eq!(result.capabilities_created, 2);

    // Verify RawSchema was persisted.
    {
        let collection: Collection<RawSchema> = db.database.collection(RawSchema::COLLECTION);
        let count = collection
            .count_documents(doc! { "provider": provider })
            .await
            .expect("count raw schemas");
        assert_eq!(count, 1, "exactly one RawSchema must be persisted");
    }

    // Verify NormalizedEndpoints were persisted.
    {
        let collection: Collection<NormalizedEndpoint> =
            db.database.collection(NormalizedEndpoint::COLLECTION);
        let count = collection
            .count_documents(doc! { "provider": provider })
            .await
            .expect("count endpoints");
        assert_eq!(count, 2, "exactly two endpoints must be persisted");
    }

    // Verify Capabilities were persisted.
    {
        let collection: Collection<Capability> = db.database.collection(Capability::COLLECTION);
        let count = collection
            .count_documents(doc! { "tags": provider })
            .await
            .expect("count capabilities");
        assert_eq!(count, 2, "exactly two capabilities must be persisted");
    }

    // ── Cleanup ────────────────────────────────────────────────────────
    purge_by_provider(&db, provider).await;
    let _ = tokio::fs::remove_file(&file_path).await;
}
