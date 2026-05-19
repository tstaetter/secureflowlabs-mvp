#[derive(Debug, Clone)]
pub struct ValidationDefinition {
    pub required_fields: Vec<String>,
    pub schema: Option<JsonSchema>,
}
