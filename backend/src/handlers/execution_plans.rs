use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use mongodb::bson::doc;
use serde::Serialize;

use crate::runtime::ExecutionPlan;
use crate::{AppResult, AppState, DbError};

#[derive(Serialize)]
pub struct ExecutionPlanSummary {
    id: String,
    provider_id: String,
    method: String,
    url: String,
    headers: usize,
    query_params: usize,
    body_fields: usize,
    max_retries: u8,
}

/// List all execution plans.
pub async fn list_execution_plans(
    State(state): State<Arc<AppState>>,
) -> AppResult<(StatusCode, Json<Vec<ExecutionPlanSummary>>)> {
    let db = state.db.as_ref().ok_or(DbError::Configuration)?;

    let plans: Vec<ExecutionPlan> = db.find_many(doc! {}).await?;

    let summaries: Vec<ExecutionPlanSummary> = plans
        .into_iter()
        .map(|p| ExecutionPlanSummary {
            id: p.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            provider_id: p.provider_id.to_hex(),
            method: format!("{:?}", p.request.method),
            url: p.request.url,
            headers: p.request.headers.len(),
            query_params: p.request.query.len(),
            body_fields: p.request.body.len(),
            max_retries: p.retry.max_retries,
        })
        .collect();

    Ok((StatusCode::OK, Json(summaries)))
}
