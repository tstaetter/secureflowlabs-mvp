#[derive(Debug, Clone)]
pub struct Capability {
    pub id: String,

    pub semantic_name: String,

    pub description: String,

    pub endpoint_id: String,

    pub tags: Vec<String>,
}
