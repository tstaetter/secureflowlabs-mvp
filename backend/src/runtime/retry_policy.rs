use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// The max number of retries
    pub max_retries: u8,
    ///Define the HTTP status a retry should be made for
    pub retry_on_status: Vec<u16>,
    /// Time in ms to wait until the next retry
    pub backoff_ms: u64,
}

impl RetryPolicy {
    pub fn retry(&mut self) -> bool {
        self.max_retries -= 1;

        self.max_retries > 0
    }
}
