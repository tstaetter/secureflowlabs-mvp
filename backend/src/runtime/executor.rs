#[async_trait::async_trait]
pub trait Executor {
    async fn execute(
        &self,
        plan: ExecutionPlan,
        context: ExecutionContext,
    ) -> anyhow::Result<ExecutionResult>;
}

#[derive(Debug, Serialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: serde_json::Value,
    pub metadata: ResponseMetadata,
}
