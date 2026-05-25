use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use mongodb::bson::doc;
use serde::Serialize;

use crate::db::RawSchema;
use crate::{AppResult, AppState, DbError};

#[derive(Serialize)]
pub struct ProviderSummary {
    id: String,
    provider: String,
    version: String,
    source: String,
    url: String,
}

/// List all ingested providers (RawSchemas).
pub async fn providers(
    State(state): State<Arc<AppState>>,
) -> AppResult<(StatusCode, Json<Vec<ProviderSummary>>)> {
    let db = state.db.as_ref().ok_or(DbError::Configuration)?;

    let schemas: Vec<RawSchema> = db.find_many(doc! {}).await?;

    let summaries: Vec<ProviderSummary> = schemas
        .into_iter()
        .map(|s| ProviderSummary {
            id: s.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            provider: s.provider,
            version: s.version,
            source: format!("{:?}", s.source),
            url: s.url,
        })
        .collect();

    Ok((StatusCode::OK, Json(summaries)))
}
