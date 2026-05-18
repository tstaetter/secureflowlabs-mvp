use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaSource {
    #[serde(rename = "openapi_json")]
    OpenApiJson,
    #[serde(rename = "openapi_yaml")]
    OpenApiYaml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSchema {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub provider: String,
    pub source: SchemaSource,
    pub version: String,
    pub spec: Value,
}
