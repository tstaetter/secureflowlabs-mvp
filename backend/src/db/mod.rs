mod capability_node;
mod normalized_schema_entry;
mod raw_schema;

pub use capability_node::*;
pub use normalized_schema_entry::*;
pub use raw_schema::*;

pub trait Model: Send + Sync {}
