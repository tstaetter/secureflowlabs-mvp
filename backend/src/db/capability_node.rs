use crate::db::Model;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub semantic_name: String,
    pub description: String,
    pub endpoint_id: String,
    pub tags: Vec<String>,
}

impl Model for Capability {
    #[cfg(test)]
    const COLLECTION: &'static str = "test_capabilities";
    #[cfg(not(test))]
    const COLLECTION: &'static str = "capabilities";

    fn get_id(&self) -> ObjectId {
        self.id.unwrap_or_default()
    }
}
