use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::Model;
use crate::{
    AppDatabase, AppResult,
    db::RawSchema,
    runtime::{RequestDefinition, RetryPolicy},
};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ExecutionPlan {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub provider_id: ObjectId,
    /// The HTTP request to be executed
    pub request: RequestDefinition,
    /// The retry policy if the request fails
    pub retry: RetryPolicy,
}

impl Model for ExecutionPlan {
    #[cfg(test)]
    const COLLECTION: &'static str = "test_execution_plans";
    #[cfg(not(test))]
    const COLLECTION: &'static str = "execution_plans";

    fn get_id(&self) -> ObjectId {
        self.id.unwrap_or_default()
    }
}

impl ExecutionPlan {
    /// Resolve the `RawSchema` by its foreign key so callers can obtain the
    /// base URL for the provider.
    pub async fn resolve_schema(&self, db: &AppDatabase) -> AppResult<RawSchema> {
        let filter = doc! {"_id": self.provider_id};
        db.find_one::<RawSchema>(filter).await
    }
}
