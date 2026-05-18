use crate::AppError;
use crate::db::NormalizedEndpoint;

mod capability;
mod normalizer;
mod parser;

pub use capability::infer_capability;
pub use normalizer::OpenApiNormalizer;
pub use parser::*;

pub type NormalizeResult<T> = Result<T, AppError>;

pub trait ApiNormalizer {
    fn normalize(&self) -> NormalizeResult<Vec<NormalizedEndpoint>>;
}
