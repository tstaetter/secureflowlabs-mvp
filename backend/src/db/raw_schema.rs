use crate::db::Model;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaSource {
    #[serde(rename = "openapi_json")]
    OpenApiJson,
    #[serde(rename = "openapi_yaml")]
    OpenApiYaml,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RawSchema {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub provider: String,
    pub source: SchemaSource,
    pub version: String,
    pub spec: Value,
    pub url: String,
}

impl Model for RawSchema {
    #[cfg(test)]
    const COLLECTION: &'static str = "test_schemas";
    #[cfg(not(test))]
    const COLLECTION: &'static str = "schemas";

    fn get_id(&self) -> ObjectId {
        self.id.unwrap_or_default()
    }
}
