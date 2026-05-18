use crate::db::NormalizedEndpoint;
use crate::AppError;

mod capability;
mod normalizer;
mod parser;

pub type NormalizeResult<T> = Result<T, AppError>;

pub trait ApiNormalizer {
    fn normalize(&self) -> NormalizeResult<Vec<NormalizedEndpoint>>;
}
