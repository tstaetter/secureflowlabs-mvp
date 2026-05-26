use crate::db::{HttpMethod, NormalizedEndpoint};

/// Infer a capability name from a normalized endpoint.
///
/// Uses the HTTP method to determine a verb prefix (`create`, `get`, `update`,
/// `delete`) and the local Ollama server to extract the resource noun from
/// the path.  Falls back to `"{prefix}_unknown"` when Ollama is unavailable.
pub async fn infer_capability(endpoint: &NormalizedEndpoint) -> String {
    let prefix = match endpoint.method {
        HttpMethod::Post => "create",
        HttpMethod::Delete => "delete",
        HttpMethod::Patch | HttpMethod::Put => "update",
        HttpMethod::Get => "get",
    };

    let mut resource = endpoint.path.split('/').next_back().unwrap_or("unknown");
    if resource.contains('{') {
        resource = endpoint.path.split('/').rev().nth(1).unwrap_or("unknown");
    }

    format!("{prefix}_{resource}")
}
