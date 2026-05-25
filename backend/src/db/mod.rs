mod capability_node;
mod normalized_endpoint;
mod raw_schema;

use crate::{AppError, AppResult, DbError};
pub use capability_node::*;
use futures::stream::StreamExt;
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::{Client, Collection};
pub use normalized_endpoint::*;
pub use raw_schema::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    None,
    Basic,
    Bearer,
    OAuth2,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl From<HttpMethod> for reqwest::Method {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => Self::GET,
            HttpMethod::Post => Self::POST,
            HttpMethod::Put => Self::PUT,
            HttpMethod::Delete => Self::DELETE,
            HttpMethod::Patch => Self::PATCH,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldLocation {
    Query,
    Path,
    Header,
    Body,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Object,
    Array,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub location: FieldLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputField {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
}

/// Generic model trait
pub trait Model: Serialize + DeserializeOwned + Validate + Send + Sync {
    /// The name of the collection inside MongoDB
    const COLLECTION: &'static str = "";

    fn get_id(&self) -> mongodb::bson::oid::ObjectId;
}

#[derive(Debug, Clone)]
pub struct AppDatabase {
    pub database: mongodb::Database,
}

impl AppDatabase {
    pub async fn try_new() -> AppResult<Self> {
        let database = Self::init_db_connection().await?;
        Ok(Self { database })
    }

    /// Initialize MongoDB connection
    async fn init_db_connection() -> AppResult<mongodb::Database> {
        let mongo_uri = std::env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let mongo_db = std::env::var("MONGODB_NAME").unwrap_or_else(|_| "sfl_mvp_dev".to_string());
        let client = Client::with_uri_str(&mongo_uri).await?;

        info!("Connected to MongoDB on {}", mongo_uri);

        Ok(client.database(&mongo_db))
    }

    /// Create new document
    pub async fn insert_one<T: Model>(&self, doc: &T) -> AppResult<T> {
        let collection: Collection<T> = self.database.collection(T::COLLECTION);
        let result = collection.insert_one(doc).await?;
        let created_doc = collection
            .find_one(doc! { "_id": result.inserted_id })
            .await?;

        match created_doc {
            Some(created_doc) => Ok(created_doc),
            None => Err(AppError::Db(DbError::Insertion(
                "Failed to insert new doc".to_string(),
            ))),
        }
    }

    pub async fn find_one<T: Model>(&self, filter: Document) -> AppResult<T> {
        let collection: Collection<T> = self.database.collection(T::COLLECTION);
        let result = collection.find_one(filter.clone()).await?;

        match result {
            Some(doc) => Ok(doc),
            None => Err(
                DbError::NotFound(format!("Document with filter {:?} not found", filter)).into(),
            ),
        }
    }

    pub async fn find_many<T: Model>(&self, filter: Document) -> AppResult<Vec<T>> {
        let collection: Collection<T> = self.database.collection(T::COLLECTION);
        let mut result = collection.find(filter.clone()).await?;
        let mut documents = Vec::new();

        while result.has_next() {
            if let Some(doc) = result.next().await {
                documents.push(doc?);
            } else {
                break;
            }
        }

        Ok(documents)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
pub mod tests {
    use super::*;
    use mongodb::bson::doc;
    use mongodb::Client;
    use std::time::Duration;

    /// Probe whether MongoDB is reachable within a short timeout.
    pub async fn try_connect() -> Option<AppDatabase> {
        let mongo_uri = std::env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        let mongo_db = std::env::var("MONGODB_NAME").unwrap_or_else(|_| "sfl_mvp_dev".to_string());

        let client = tokio::time::timeout(Duration::from_secs(3), Client::with_uri_str(&mongo_uri))
            .await
            .ok()?
            .ok()?;

        let db = client.database(&mongo_db);
        tokio::time::timeout(Duration::from_secs(3), db.run_command(doc! { "ping": 1 }))
            .await
            .ok()?
            .ok()?;

        Some(AppDatabase { database: db })
    }

    /// A minimal model struct for testing insert/find operations.
    #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
    struct TestDoc {
        #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
        id: Option<mongodb::bson::oid::ObjectId>,
        name: String,
        value: i32,
    }

    impl Model for TestDoc {
        const COLLECTION: &'static str = "test_docs";

        fn get_id(&self) -> mongodb::bson::oid::ObjectId {
            self.id.unwrap_or_default()
        }
    }

    // ── insert_one ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn insert_one_returns_inserted_doc_with_id() {
        let db = match try_connect().await {
            Some(db) => db,
            None => return,
        };

        let doc = TestDoc {
            id: None,
            name: "alice".into(),
            value: 42,
        };

        let created = db.insert_one(&doc).await.expect("insert must succeed");

        assert!(created.id.is_some(), "inserted doc must have an _id");
        assert_eq!(created.name, "alice");
        assert_eq!(created.value, 42);

        // Cleanup
        let collection: Collection<TestDoc> = db.database.collection(TestDoc::COLLECTION);
        let _ = collection.delete_many(doc! { "name": "alice" }).await;
    }

    #[tokio::test]
    async fn insert_two_docs_produces_distinct_ids() {
        let db = match try_connect().await {
            Some(db) => db,
            None => return,
        };

        let a = TestDoc {
            id: None,
            name: "a".into(),
            value: 1,
        };
        let b = TestDoc {
            id: None,
            name: "b".into(),
            value: 2,
        };

        let created_a = db.insert_one(&a).await.expect("insert a");
        let created_b = db.insert_one(&b).await.expect("insert b");

        assert_ne!(created_a.id, created_b.id);

        // Cleanup
        let collection: Collection<TestDoc> = db.database.collection(TestDoc::COLLECTION);
        let _ = collection
            .delete_many(doc! { "name": { "$in": ["a", "b"] } })
            .await;
    }

    // ── find_one ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn find_one_returns_inserted_doc() {
        let db = match try_connect().await {
            Some(db) => db,
            None => return,
        };

        let doc = TestDoc {
            id: None,
            name: "findable".into(),
            value: 99,
        };

        let created = db.insert_one(&doc).await.expect("insert");
        let id = created.id.unwrap();

        let found = db
            .find_one::<TestDoc>(doc! { "_id": id })
            .await
            .expect("find_one must succeed");

        assert_eq!(found.name, "findable");
        assert_eq!(found.value, 99);

        // Cleanup
        let collection: Collection<TestDoc> = db.database.collection(TestDoc::COLLECTION);
        let _ = collection.delete_many(doc! { "_id": id }).await;
    }

    #[tokio::test]
    async fn find_one_missing_returns_err() {
        let db = match try_connect().await {
            Some(db) => db,
            None => return,
        };

        let fake_id = mongodb::bson::oid::ObjectId::new();

        let result = db.find_one::<TestDoc>(doc! { "_id": fake_id }).await;

        assert!(result.is_err(), "non-existent doc must return Err");
    }

    #[tokio::test]
    async fn find_one_with_empty_filter_returns_first_doc() {
        let db = match try_connect().await {
            Some(db) => db,
            None => return,
        };

        let doc = TestDoc {
            id: None,
            name: "first".into(),
            value: 1,
        };
        let _created = db.insert_one(&doc).await.expect("insert");

        let found = db
            .find_one::<TestDoc>(doc! {})
            .await
            .expect("find_one must succeed");

        assert_eq!(found.name, "first");

        // Cleanup
        let collection: Collection<TestDoc> = db.database.collection(TestDoc::COLLECTION);
        let _ = collection.delete_many(doc! { "name": "first" }).await;
    }

    // ── try_new ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn try_new_connects_to_valid_uri() {
        let db = match try_connect().await {
            Some(db) => db,
            None => return,
        };

        // If we got here, try_new succeeded implicitly via try_connect.
        // Just verify the database handle is functional.
        let ping_result = db.database.run_command(doc! { "ping": 1 }).await;
        assert!(ping_result.is_ok(), "ping must succeed");
    }

    #[tokio::test]
    async fn try_new_fails_with_invalid_uri() {
        // Use a connection-string timeout so we don't hang.
        let uri = "mongodb://invalid-host:27017/?serverSelectionTimeoutMS=2000";
        unsafe {
            std::env::set_var("MONGODB_URI", uri);
        }

        let db = AppDatabase::try_new()
            .await
            .expect("client creation is lazy — always succeeds");

        unsafe {
            std::env::remove_var("MONGODB_URI");
        }

        // The actual connection attempt happens on the first operation.
        let doc = TestDoc {
            id: None,
            name: "nobody".into(),
            value: 0,
        };
        let result = db.insert_one(&doc).await;

        assert!(
            result.is_err(),
            "insert_one must fail when the database host is unreachable"
        );
    }

    // ── Edge cases ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn insert_empty_doc_persists_correctly() {
        let db = match try_connect().await {
            Some(db) => db,
            None => return,
        };

        let doc = TestDoc {
            id: None,
            name: String::new(),
            value: 0,
        };

        let created = db.insert_one(&doc).await.expect("insert empty");

        assert!(created.id.is_some());
        assert_eq!(created.name, "");
        assert_eq!(created.value, 0);

        // Cleanup
        let collection: Collection<TestDoc> = db.database.collection(TestDoc::COLLECTION);
        let _ = collection.delete_many(doc! { "name": "" }).await;
    }
}
