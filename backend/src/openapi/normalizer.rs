use crate::db::{AuthType, HttpMethod, NormalizedEndpoint};
use crate::openapi::{ApiNormalizer, NormalizeResult};
use openapiv3::{OpenAPI, Operation};

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
        NormalizedEndpoint {
            id: format!("{}:{}:{}", provider, format!("{:?}", method), path),
            provider: provider.to_string(),
            method,
            path: path.to_string(),
            summary: op.summary,
            auth: AuthType::Bearer,
            inputs: vec![],
            outputs: vec![],
        }
    }
}
