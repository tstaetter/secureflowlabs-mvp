use crate::{
    db::{Model, NormalizedEndpoint}, AppDatabase,
    AppResult,
};
use mongodb::bson::doc;
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
    pub endpoint_id: ObjectId,
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

impl Capability {
    pub async fn endpoint(&self, db: AppDatabase) -> AppResult<NormalizedEndpoint> {
        let filter = doc! { "_id": self.endpoint_id };
        db.find_one(filter).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::try_connect;
    use crate::db::{AuthType, HttpMethod};

    #[tokio::test]
    async fn test_capability_endpoint_retrieval() {
        let db = match try_connect().await {
            Some(db) => db,
            None => return,
        };

        // 1. Create a NormalizedEndpoint
        let endpoint = NormalizedEndpoint {
            id: None,
            internal_id: "test_ep_for_cap".into(),
            provider: "test_provider".into(),
            method: HttpMethod::Get,
            path: "/test".into(),
            summary: Some("Test endpoint".into()),
            auth: AuthType::None,
            inputs: vec![],
            outputs: vec![],
        };
        let created_ep = db
            .insert_one(&endpoint)
            .await
            .expect("Failed to insert endpoint");
        let ep_id = created_ep.id.expect("Endpoint should have an ID");

        // 2. Create a Capability referencing it
        let capability = Capability {
            id: None,
            semantic_name: "test_capability".into(),
            description: "Test description".into(),
            endpoint_id: ep_id,
            tags: vec!["test".into()],
        };

        // 3. Call endpoint() and verify
        let fetched_ep = capability
            .endpoint(db.clone())
            .await
            .expect("Failed to fetch endpoint");
        assert_eq!(fetched_ep.id, Some(ep_id));
        assert_eq!(fetched_ep.internal_id, "test_ep_for_cap");

        // Cleanup
        let ep_coll = db
            .database
            .collection::<NormalizedEndpoint>(NormalizedEndpoint::COLLECTION);
        let _ = ep_coll.delete_one(doc! { "_id": ep_id }).await;
    }

    #[tokio::test]
    async fn test_capability_endpoint_not_found() {
        let db = match try_connect().await {
            Some(db) => db,
            None => return,
        };

        let capability = Capability {
            id: None,
            semantic_name: "missing_ep".into(),
            description: "Test".into(),
            endpoint_id: ObjectId::new(),
            tags: vec![],
        };

        let result = capability.endpoint(db).await;
        assert!(result.is_err());
    }
}
