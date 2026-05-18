mod capability_node;
mod normalized_endpoint;
mod raw_schema;

pub use capability_node::*;
pub use normalized_endpoint::*;
pub use raw_schema::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    None,
    Basic,
    Bearer,
    OAuth2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldLocation {
    Query,
    Path,
    Header,
    Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
impl PartialEq for HttpMethod {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

pub trait Model: Send + Sync {}
