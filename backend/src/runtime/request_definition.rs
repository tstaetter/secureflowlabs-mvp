use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::db::{FieldLocation, HttpMethod, NormalizedEndpoint};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RequestDefinition {
    /// HTTP method to use
    pub method: HttpMethod,
    /// URL taken from the normalized endpoint definition
    pub url: String,
    /// Path params
    pub path: Vec<String>,
    /// Headers taken from input fields of the endpoint
    pub headers: Vec<String>,
    /// Query fields
    pub query: Vec<String>,
    /// Body fields
    pub body: Vec<String>,
}

impl From<NormalizedEndpoint> for RequestDefinition {
    fn from(value: NormalizedEndpoint) -> Self {
        Self {
            method: value.method.clone(),
            url: value.path.clone(),
            path: path_from_input(value.clone()),
            headers: headers_from_input(value.clone()),
            query: query_from_input(value.clone()),
            body: body_from_input(value),
        }
    }
}

fn headers_from_input(endpoint: NormalizedEndpoint) -> Vec<String> {
    let mut headers = Vec::new();

    for param in endpoint.inputs {
        if param.location == FieldLocation::Header {
            headers.push(param.name);
        }
    }

    headers
}

fn query_from_input(endpoint: NormalizedEndpoint) -> Vec<String> {
    let mut query_params = Vec::new();

    for param in endpoint.inputs {
        if param.location == FieldLocation::Query {
            query_params.push(param.name);
        }
    }

    query_params
}

fn path_from_input(endpoint: NormalizedEndpoint) -> Vec<String> {
    let mut path_params = Vec::new();

    for param in endpoint.inputs {
        if param.location == FieldLocation::Path {
            path_params.push(param.name);
        }
    }

    path_params
}

fn body_from_input(endpoint: NormalizedEndpoint) -> Vec<String> {
    let mut body_fields = Vec::new();

    for param in endpoint.inputs {
        if param.location == FieldLocation::Body {
            body_fields.push(param.name);
        }
    }

    body_fields
}
