#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u8,
    pub retry_on_status: Vec<u16>,
    pub backoff_ms: u64,
}
