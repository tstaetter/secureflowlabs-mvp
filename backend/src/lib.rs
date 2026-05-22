pub mod db;
mod error;
mod handlers;
pub mod openapi;
mod payloads;
mod routes;
pub mod runtime;

pub use crate::db::AppDatabase;
pub use error::*;
pub use handlers::*;
pub use openapi::pipeline::*;
pub use routes::*;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: Option<AppDatabase>,
}
