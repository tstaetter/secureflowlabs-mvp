use crate::db::{
    AuthType, FieldLocation, FieldType, HttpMethod, InputField, NormalizedEndpoint, OutputField,
};
use crate::openapi::{ApiNormalizer, NormalizeResult};
use mongodb::bson::oid::ObjectId;
use openapiv3::{OpenAPI, Operation, Parameter, ReferenceOr, Schema, SchemaKind, StatusCode, Type};

pub struct OpenApiNormalizer {
    pub provider: String,
    pub spec: OpenAPI,
}

impl ApiNormalizer for OpenApiNormalizer {
    fn normalize(&self) -> NormalizeResult<Vec<NormalizedEndpoint>> {
        Ok(self.normalize_api(self.provider.clone(), self.spec.clone()))
    }
}

impl OpenApiNormalizer {
    fn normalize_api(&self, provider: String, spec: OpenAPI) -> Vec<NormalizedEndpoint> {
        let mut endpoints = Vec::new();

        for (path, item) in spec.paths.paths {
            let item = match item {
                openapiv3::ReferenceOr::Item(i) => i,
                _ => continue,
            };

            if let Some(op) = item.get {
                endpoints.push(self.normalize_operation(
                    provider.clone(),
                    &path,
                    HttpMethod::Get,
                    op,
                ));
            }

            if let Some(op) = item.post {
                endpoints.push(self.normalize_operation(
                    provider.clone(),
                    &path,
                    HttpMethod::Post,
                    op,
                ));
            }

            if let Some(op) = item.put {
                endpoints.push(self.normalize_operation(
                    provider.clone(),
                    &path,
                    HttpMethod::Put,
                    op,
                ));
            }

            if let Some(op) = item.patch {
                endpoints.push(self.normalize_operation(
                    provider.clone(),
                    &path,
                    HttpMethod::Patch,
                    op,
                ));
            }

            if let Some(op) = item.delete {
                endpoints.push(self.normalize_operation(
                    provider.clone(),
                    &path,
                    HttpMethod::Delete,
                    op,
                ));
            }
        }

        endpoints
    }

    fn normalize_operation(
        &self,
        provider: String,
        path: &str,
        method: HttpMethod,
        op: Operation,
    ) -> NormalizedEndpoint {
        let mut inputs: Vec<InputField> = Vec::new();

        // ── Extract parameter-based inputs ───────────────────────────────
        for param in &op.parameters {
            let param = match param {
                ReferenceOr::Item(p) => p,
                _ => continue,
            };

            let data = param.parameter_data_ref();
            let (location, schema) = match param {
                Parameter::Query {
                    parameter_data,
                    allow_reserved: _,
                    style: _,
                    allow_empty_value: _,
                } => (
                    FieldLocation::Query,
                    schema_from_param_format(&parameter_data.format),
                ),
                Parameter::Header {
                    parameter_data,
                    style: _,
                } => (
                    FieldLocation::Header,
                    schema_from_param_format(&parameter_data.format),
                ),
                Parameter::Path {
                    parameter_data,
                    style: _,
                } => (
                    FieldLocation::Path,
                    schema_from_param_format(&parameter_data.format),
                ),
                Parameter::Cookie { .. } => continue, // Skip cookie params for now.
            };

            inputs.push(InputField {
                name: data.name.clone(),
                field_type: schema_type(&schema),
                required: data.required,
                location,
            });
        }

        // ── Extract request-body-based inputs ────────────────────────────
        if let Some(request_body) = &op.request_body {
            if let ReferenceOr::Item(body) = request_body {
                // Look for the first JSON-based media type.
                for (content_type, media_type) in &body.content {
                    if content_type != "application/json" {
                        continue;
                    }

                    if let Some(schema_ref) = &media_type.schema {
                        if let ReferenceOr::Item(schema) = schema_ref {
                            extract_body_fields(schema, &mut inputs);
                        }
                    }
                    break; // Only process the first JSON media type.
                }
            }
        }

        // ── Extract response-based outputs ────────────────────────────
        let mut outputs: Vec<OutputField> = Vec::new();
        for (status, response_ref) in &op.responses.responses {
            if !is_success(status) {
                continue;
            }
            if let ReferenceOr::Item(response) = response_ref {
                for (content_type, media_type) in &response.content {
                    if content_type != "application/json" {
                        continue;
                    }
                    if let Some(schema_ref) = &media_type.schema {
                        if let ReferenceOr::Item(schema) = schema_ref {
                            extract_response_fields(schema, &mut outputs);
                        }
                    }
                    break; // First JSON media type only.
                }
            }
        }

        NormalizedEndpoint {
            id: None,
            schema: ObjectId::default(),
            internal_id: format!("{}:{}:{}", provider.clone(), format!("{:?}", method), path),
            method,
            path: path.to_string(),
            summary: op.summary,
            auth: AuthType::Bearer,
            inputs,
            outputs,
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Extract a `Schema` reference from a `ParameterSchemaOrContent`.
fn schema_from_param_format(format: &openapiv3::ParameterSchemaOrContent) -> Option<&Schema> {
    match format {
        openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => match schema_ref {
            ReferenceOr::Item(s) => Some(s),
            _ => None,
        },
        openapiv3::ParameterSchemaOrContent::Content(_) => None,
    }
}

/// Map an optional `Schema` to a `FieldType`.  Returns `String` when no
/// schema is available (safe default).
fn schema_type(schema: &Option<&Schema>) -> FieldType {
    match schema {
        Some(s) => type_to_field_type(&s.schema_kind),
        None => FieldType::String,
    }
}

/// Map a `SchemaKind` to our internal `FieldType`.
fn type_to_field_type(kind: &SchemaKind) -> FieldType {
    match kind {
        SchemaKind::Type(t) => match t {
            Type::String(_) => FieldType::String,
            Type::Integer(_) => FieldType::Integer,
            Type::Number(_) => FieldType::Float,
            Type::Boolean(_) => FieldType::Boolean,
            Type::Array(_) => FieldType::Array,
            Type::Object(_) => FieldType::Object,
        },
        SchemaKind::OneOf { .. } | SchemaKind::AnyOf { .. } | SchemaKind::AllOf { .. } => {
            FieldType::Object
        }
        SchemaKind::Any(_) => FieldType::String,
        SchemaKind::Not { .. } => FieldType::String,
    }
}

/// Recursively walk an object schema's properties and push them as
/// body-located `InputField`s.
fn extract_body_fields(schema: &Schema, inputs: &mut Vec<InputField>) {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) => {
            for (name, prop_ref) in &obj.properties {
                let prop = match prop_ref {
                    ReferenceOr::Item(s) => s.as_ref(),
                    _ => continue,
                };

                let required = obj.required.contains(name);

                inputs.push(InputField {
                    name: name.clone(),
                    field_type: type_to_field_type(&prop.schema_kind),
                    required,
                    location: FieldLocation::Body,
                });
            }
        }
        SchemaKind::Any(any) => {
            for (name, prop_ref) in &any.properties {
                let prop = match prop_ref {
                    ReferenceOr::Item(s) => s.as_ref(),
                    _ => continue,
                };

                let required = any.required.contains(name);

                inputs.push(InputField {
                    name: name.clone(),
                    field_type: type_to_field_type(&prop.schema_kind),
                    required,
                    location: FieldLocation::Body,
                });
            }
        }
        _ => {
            // Non-object schemas (e.g., a raw array or string body) — skip
            // property-level extraction.
        }
    }
}

/// Returns `true` for success-range status codes (2XX).
fn is_success(status: &StatusCode) -> bool {
    matches!(status, StatusCode::Range(2) | StatusCode::Code(200..=299))
}

/// Walk a response object schema's properties and push them as
/// `OutputField`s.
fn extract_response_fields(schema: &Schema, outputs: &mut Vec<OutputField>) {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) => {
            for (name, prop_ref) in &obj.properties {
                let prop = match prop_ref {
                    ReferenceOr::Item(s) => s.as_ref(),
                    _ => continue,
                };

                let required = obj.required.contains(name);

                outputs.push(OutputField {
                    name: name.clone(),
                    field_type: type_to_field_type(&prop.schema_kind),
                    required,
                });
            }
        }
        SchemaKind::Any(any) => {
            for (name, prop_ref) in &any.properties {
                let prop = match prop_ref {
                    ReferenceOr::Item(s) => s.as_ref(),
                    _ => continue,
                };

                let required = any.required.contains(name);

                outputs.push(OutputField {
                    name: name.clone(),
                    field_type: type_to_field_type(&prop.schema_kind),
                    required,
                });
            }
        }
        _ => {
            // Non-object schemas — skip.
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// A small OpenAPI spec with a variety of parameter locations and a
    /// JSON request body.
    const SPEC_WITH_INPUTS: &str = r#"{
        "openapi": "3.0.3",
        "info": { "title": "Input Test API", "version": "1.0" },
        "paths": {
            "/v1/orders/{id}": {
                "post": {
                    "summary": "Create an order",
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "string" }
                        },
                        {
                            "name": "idempotency-key",
                            "in": "header",
                            "required": false,
                            "schema": { "type": "string" }
                        },
                        {
                            "name": "expand",
                            "in": "query",
                            "required": false,
                            "schema": { "type": "array", "items": { "type": "string" } }
                        }
                    ],
                    "requestBody": {
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "amount":  { "type": "integer" },
                                        "currency": { "type": "string" },
                                        "metadata": { "type": "object" }
                                    },
                                    "required": ["amount", "currency"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Order created",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "id":      { "type": "string" },
                                            "status":  { "type": "string" },
                                            "created": { "type": "integer" }
                                        },
                                        "required": ["id", "status"]
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }"#;

    #[test]
    fn extracts_parameters_and_body_fields() {
        let spec: OpenAPI = serde_json::from_str(SPEC_WITH_INPUTS).expect("valid spec");
        let provider = spec.info.title.clone();

        let normalizer = OpenApiNormalizer { provider, spec };
        let endpoints = normalizer.normalize().expect("normalize");

        let ep = &endpoints[0];
        assert_eq!(ep.path, "/v1/orders/{id}");

        // ── Parameter inputs ──────────────────────────────────────────
        let path_param = ep.inputs.iter().find(|i| i.name == "id").unwrap();
        assert_eq!(path_param.location, FieldLocation::Path);
        assert_eq!(path_param.field_type, FieldType::String);
        assert!(path_param.required);

        let header_param = ep
            .inputs
            .iter()
            .find(|i| i.name == "idempotency-key")
            .unwrap();
        assert_eq!(header_param.location, FieldLocation::Header);
        assert_eq!(header_param.field_type, FieldType::String);
        assert!(!header_param.required);

        let query_param = ep.inputs.iter().find(|i| i.name == "expand").unwrap();
        assert_eq!(query_param.location, FieldLocation::Query);
        assert_eq!(query_param.field_type, FieldType::Array);
        assert!(!query_param.required);

        // ── Body inputs ───────────────────────────────────────────────
        let amount = ep.inputs.iter().find(|i| i.name == "amount").unwrap();
        assert_eq!(amount.location, FieldLocation::Body);
        assert_eq!(amount.field_type, FieldType::Integer);
        assert!(amount.required);

        let currency = ep.inputs.iter().find(|i| i.name == "currency").unwrap();
        assert_eq!(currency.location, FieldLocation::Body);
        assert_eq!(currency.field_type, FieldType::String);
        assert!(currency.required);

        let metadata = ep.inputs.iter().find(|i| i.name == "metadata").unwrap();
        assert_eq!(metadata.location, FieldLocation::Body);
        assert_eq!(metadata.field_type, FieldType::Object);
        assert!(!metadata.required);

        // Total: 3 params + 3 body properties = 6 inputs.
        assert_eq!(ep.inputs.len(), 6);

        // ── Output fields ────────────────────────────────────────────
        let id_out = ep.outputs.iter().find(|o| o.name == "id").unwrap();
        assert_eq!(id_out.field_type, FieldType::String);
        assert!(id_out.required);

        let status_out = ep.outputs.iter().find(|o| o.name == "status").unwrap();
        assert_eq!(status_out.field_type, FieldType::String);
        assert!(status_out.required);

        let created_out = ep.outputs.iter().find(|o| o.name == "created").unwrap();
        assert_eq!(created_out.field_type, FieldType::Integer);
        assert!(!created_out.required);

        // 3 response properties.
        assert_eq!(ep.outputs.len(), 3);
    }

    #[test]
    fn operation_without_body_produces_only_params() {
        let spec_json = r#"{
            "openapi": "3.0.3",
            "info": { "title": "No Body API", "version": "1.0" },
            "paths": {
                "/v1/items": {
                    "get": {
                        "summary": "List items",
                        "parameters": [
                            { "name": "limit", "in": "query", "schema": { "type": "integer" } }
                        ],
                        "responses": { "200": { "description": "ok" } }
                    }
                }
            }
        }"#;

        let spec: OpenAPI = serde_json::from_str(spec_json).expect("valid spec");
        let provider = spec.info.title.clone();

        let normalizer = OpenApiNormalizer { provider, spec };
        let endpoints = normalizer.normalize().expect("normalize");

        let ep = &endpoints[0];
        assert_eq!(ep.inputs.len(), 1);
        assert_eq!(ep.inputs[0].name, "limit");
        assert_eq!(ep.inputs[0].location, FieldLocation::Query);
        assert_eq!(ep.inputs[0].field_type, FieldType::Integer);
    }
}
