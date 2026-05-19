use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthDefinition {
    None,
    BearerToken {
        token_ref: String,
    },
    ApiKey {
        key_ref: String,
        location: ApiKeyLocation,
    },
    OAuth2 {
        connection_id: String,
    },
}
