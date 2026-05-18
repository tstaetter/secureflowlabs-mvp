use crate::db::{AuthType, HttpMethod, InputField, OutputField};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedEndpoint {
    pub id: String,
    pub provider: String,
    pub method: HttpMethod,
    pub path: String,
    pub summary: Option<String>,
    pub auth: AuthType,
    pub inputs: Vec<InputField>,
    pub outputs: Vec<OutputField>,
}
