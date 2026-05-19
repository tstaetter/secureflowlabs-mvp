#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDefinition {
    pub method: HttpMethod,

    pub url: String,

    pub headers: Vec<Header>,

    pub query: Vec<QueryParam>,

    pub body: Option<RequestBody>,
}
