use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::db::RawSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    None,
    Basic,
    Bearer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaEntryField {
    #[serde(rename = "type")]
    pub _type: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedSchemaEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub raw_schema: ObjectId,
    pub endpoint_id: String,
    pub method: String,
    pub path: String,
    pub description: Option<String>,
    pub auth_type: AuthType,
    pub input_schema: BTreeMap<String, SchemaEntryField>,
    pub response_schema: BTreeMap<String, SchemaEntryField>,
}

impl NormalizedSchemaEntry {
    async fn schema() -> RawSchema {
        todo!()
    }
}
