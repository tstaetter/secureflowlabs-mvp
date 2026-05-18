use crate::db::{RawSchema, SchemaSource};
use crate::{SpecParsingError, SpecParsingResult};
use openapiv3::OpenAPI;

/// Asynchronously reads and parses an OpenAPI specification file into an `OpenAPI` struct.
///
/// # Parameters
/// - `path`: A string slice representing the path to the OpenAPI specification file.
///
/// # Returns
/// Returns a `SpecParsingResult<OpenAPI>`:
/// - On success, it contains an `OpenAPI` instance parsed from the file.
/// - On failure, it returns an error indicating what went wrong during file reading or parsing.
///
/// # Errors
/// This function may return the following errors wrapped in `SpecParsingResult`:
/// - `std::io::Error`: If the file cannot be read (e.g., file not found, permission denied).
/// - `serde_json::Error`: If the content of the file cannot be deserialized into the `OpenAPI` struct.
///
/// # Example
/// ```rust
/// use your_crate::{get_raw_spec, SpecParsingResult, OpenAPI};
///
/// #[tokio::main]
/// async fn main() -> SpecParsingResult<()> {
///     let path = "path/to/openapi.json";
///     let openapi: OpenAPI = get_raw_spec(path).await?;
///     // Use the parsed OpenAPI object...
///     Ok(())
/// }
/// ```
///
/// # Note
/// This function expects the file to be in JSON format containing a valid OpenAPI specification.
async fn get_raw_spec(path: &str) -> SpecParsingResult<OpenAPI> {
    let content = std::fs::read_to_string(path)?;
    let openapi: OpenAPI = serde_json::from_str(&content)?;

    Ok(openapi)
}

impl TryFrom<OpenAPI> for RawSchema {
    type Error = SpecParsingError;

    fn try_from(value: OpenAPI) -> Result<Self, Self::Error> {
        Ok(Self {
            id: None,
            provider: value.info.title.clone(),
            source: SchemaSource::OpenApiJson,
            version: value.info.version.clone(),
            spec: serde_json::to_value(value)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_raw_spec() -> anyhow::Result<()> {
        let spec_path = "tmp/stripe_spec3.json";
        let spec = get_raw_spec(spec_path).await?;

        assert!(!spec.info.title.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_raw_spec_fails() -> anyhow::Result<()> {
        let spec_path = "nonexistent_spec.json";
        let result = get_raw_spec(spec_path).await;

        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_try_from() -> anyhow::Result<()> {
        let spec_path = "tmp/stripe_spec3.json";
        let spec = get_raw_spec(spec_path).await?;
        let schema = RawSchema::try_from(spec)?;
        let value = schema.spec;

        assert_eq!(schema.provider, "Stripe API");
        assert!(value.is_object());

        Ok(())
    }
}
