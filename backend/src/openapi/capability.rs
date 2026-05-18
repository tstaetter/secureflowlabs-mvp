use crate::db::{HttpMethod, NormalizedEndpoint};

pub fn infer_capability(endpoint: &NormalizedEndpoint) -> String {
    let path = endpoint.path.to_lowercase();

    if endpoint.method == HttpMethod::Post && path.contains("customer") {
        return "create_customer".into();
    }

    if endpoint.method == HttpMethod::Delete {
        return "delete_resource".into();
    }

    "unknown".into()
}
