use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    AppDatabase,
    db::RawSchema,
    runtime::{RequestDefinition, RetryPolicy},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// TODO: The endpoint provider???
    pub provider_id: ObjectId,
    /// The HTTP request to be executed
    pub request: RequestDefinition,
    // The HTTP auth definition
    // pub auth: AuthDefinition,
    // pub validation: ValidationDefinition,
    /// The retry policy if the request fails
    pub retry: RetryPolicy,
    // The safety policy defines the security scope
    // pub safety: SafetyPolicy,
}

impl ExecutionPlan {
    async fn schema(&self, db: &AppDatabase) -> RawSchema {
        let filter = doc! {"_id": self.provider_id};
        db.find_one::<RawSchema>(filter)
            .await
            .expect("failed to query schema")
    }
}
