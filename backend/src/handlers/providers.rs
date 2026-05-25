use crate::{db::RawSchema, AppResult, AppState, DbError};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub schemas: Vec<RawSchema>,
}

impl IntoResponse for ProviderResponse {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}

pub async fn providers(State(state): State<Arc<AppState>>) -> AppResult<ProviderResponse> {
    let db = state.db.as_ref().ok_or(DbError::Configuration)?;

    let schemas = db.find_many::<RawSchema>(doc! {}).await?;

    Ok(ProviderResponse { schemas })
}
