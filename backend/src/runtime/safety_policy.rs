#[derive(Debug, Clone)]
pub struct SafetyPolicy {
    pub allow_delete: bool,
    pub require_confirmation: bool,
    pub sandbox_only: bool,
    pub max_requests_per_minute: u32,
}
