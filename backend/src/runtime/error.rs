#[derive(Debug)]
pub enum ExecutionError {
    Authentication,
    Authorization,
    RateLimited,
    InvalidInput,
    NetworkFailure,
    ProviderError,
}

pub fn classify_error(status: u16) -> ExecutionError {
    match status {
        401 => ExecutionError::Authentication,
        403 => ExecutionError::Authorization,
        429 => ExecutionError::RateLimited,
        422 => ExecutionError::InvalidInput,
        _ => ExecutionError::ProviderError,
    }
}
