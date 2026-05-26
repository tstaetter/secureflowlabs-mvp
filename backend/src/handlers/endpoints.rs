use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use mongodb::bson::doc;
use serde::Serialize;

use crate::db::NormalizedEndpoint;
use crate::{AppResult, AppState, DbError};

#[derive(Serialize)]
pub struct EndpointSummary {
    id: String,
    internal_id: String,
    method: String,
    path: String,
    summary: Option<String>,
    inputs: usize,
    outputs: usize,
}

/// List all normalized endpoints across all providers.
pub async fn list_endpoints(
    State(state): State<Arc<AppState>>,
) -> AppResult<(StatusCode, Json<Vec<EndpointSummary>>)> {
    let db = state.db.as_ref().ok_or(DbError::Configuration)?;

    let endpoints: Vec<NormalizedEndpoint> = db.find_many(doc! {}).await?;

    let summaries: Vec<EndpointSummary> = endpoints
        .into_iter()
        .map(|ep| EndpointSummary {
            id: ep.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            internal_id: ep.internal_id,
            method: format!("{:?}", ep.method),
            path: ep.path,
            summary: ep.summary,
            inputs: ep.inputs.len(),
            outputs: ep.outputs.len(),
        })
        .collect();

    Ok((StatusCode::OK, Json(summaries)))
}
