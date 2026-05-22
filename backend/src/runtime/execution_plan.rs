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
    pub provider: ObjectId,
    /// The HTTP request to be executed
    pub request: RequestDefinition,
    // The HTTP auth definition
    // pub auth: AuthDefinition,
    // pub validation: ValidationDefinition,
    // The retry policy if the request fails
    // pub retry: RetryPolicy,
    // The safety policy defines the security scope
    // pub safety: SafetyPolicy,
}

impl ExecutionPlan {
    /// WO KOMMEN DIE VALUES FÜR DIE PARAMETER HER? IST DAFÜR DER HTTTP-EXECUTOR
    /// MIT KONTEXT DA?
    pub async fn execute_http_json(&self, db: AppDatabase) {
        let client = reqwest::Client::new();
        let request = client
            .request(
                self.request.method.into(),
                format!("{}/{}", self.schema(db).await.url, self.request.url),
            )
            .query(query)
            .body(body)
            .headers(headers)
            .send()
            .await
            .unwrap();
    }

    async fn schema(&self, db: AppDatabase) -> RawSchema {
        let filter = doc! {"_id": self.provider};
        db.find_one(filter).await.unwrap()
    }
}
