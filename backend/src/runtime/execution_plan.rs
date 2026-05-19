use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub action_id: String,
    pub provider: String,
    pub request: RequestDefinition,
    pub auth: AuthDefinition,
    pub validation: ValidationDefinition,
    pub retry: RetryPolicy,
    pub safety: SafetyPolicy,
}
