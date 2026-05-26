use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use mongodb::bson::doc;
use serde::Serialize;

use crate::db::Capability;
use crate::{AppResult, AppState, DbError};

#[derive(Serialize)]
pub struct CapabilitySummary {
    id: String,
    semantic_name: String,
    description: String,
    endpoint_id: String,
    tags: Vec<String>,
}

/// List all capabilities across all providers.
pub async fn list_capabilities(
    State(state): State<Arc<AppState>>,
) -> AppResult<(StatusCode, Json<Vec<CapabilitySummary>>)> {
    let db = state.db.as_ref().ok_or(DbError::Configuration)?;

    let capabilities: Vec<Capability> = db.find_many(doc! {}).await?;

    let summaries: Vec<CapabilitySummary> = capabilities
        .into_iter()
        .map(|c| CapabilitySummary {
            id: c.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            semantic_name: c.semantic_name,
            description: c.description,
            endpoint_id: c.endpoint_id.to_hex(),
            tags: c.tags,
        })
        .collect();

    Ok((StatusCode::OK, Json(summaries)))
}
