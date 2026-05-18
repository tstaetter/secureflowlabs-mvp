mod capability_node;
mod normalized_endpoint;
mod raw_schema;

use crate::{AppError, AppResult, DbError};
pub use capability_node::*;
use mongodb::bson::doc;
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
}
