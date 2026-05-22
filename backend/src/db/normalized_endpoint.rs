use crate::{
    db::{AuthType, HttpMethod, InputField, Model, OutputField, RawSchema}, AppDatabase,
    AppResult,
};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedEndpoint {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub schema: ObjectId,
    pub internal_id: String,
    pub method: HttpMethod,
    pub path: String,
    pub summary: Option<String>,
    pub auth: AuthType,
    pub inputs: Vec<InputField>,
    pub outputs: Vec<OutputField>,
}

impl Model for NormalizedEndpoint {
    #[cfg(test)]
    const COLLECTION: &'static str = "test_endpoints";
    #[cfg(not(test))]
    const COLLECTION: &'static str = "endpoints";

    fn get_id(&self) -> ObjectId {
        self.id.unwrap_or_default()
    }
}

impl NormalizedEndpoint {
    pub async fn provider(&self, db: AppDatabase) -> AppResult<RawSchema> {
        let filter = doc! { "_id": self.schema };
        db.find_one(filter).await
    }
}
