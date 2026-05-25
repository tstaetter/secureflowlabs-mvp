use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RetryPolicy {
    /// The max number of retries
    pub max_retries: u8,
    /// The HTTP status codes to retry on
    pub retry_on_status: Vec<u16>,
    /// Time in ms to wait until the next retry
    pub backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_on_status: vec![429, 500, 502, 503, 504],
            backoff_ms: 1000,
        }
    }
}

impl RetryPolicy {
    pub fn retry(&mut self) -> bool {
        if self.max_retries == 0 {
            return false;
        }
        self.max_retries -= 1;

        self.max_retries > 0
    }
}
