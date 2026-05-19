use crate::db::{HttpMethod, NormalizedEndpoint};

pub fn infer_capability(endpoint: &NormalizedEndpoint) -> String {
    let path = endpoint.path.to_lowercase();
    let prefix = match endpoint.method {
        HttpMethod::Post => "create",
        HttpMethod::Delete => "delete",
        HttpMethod::Patch | HttpMethod::Put => "update",
        HttpMethod::Get => "get",
    };

    let resource = if path.contains("customer") {
        "customer"
    } else if path.contains("account") {
        "account"
    } else if path.contains("apple") {
        "apple_pay"
    } else if path.contains("google") {
        "google_pay"
    } else if path.contains("fees") {
        "fees"
    } else if path.contains("orders") {
        "orders"
    } else if path.contains("payments") {
        "payments"
    } else {
        "unknown"
    };

    format!("{prefix}_{resource}")
}
