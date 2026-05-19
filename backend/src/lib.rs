pub mod db;
mod error;
mod handlers;
pub mod openapi;
mod payloads;
pub mod pipeline;
mod routes;
pub mod runtime;

pub use crate::db::AppDatabase;
pub use error::*;
pub use handlers::*;
pub use pipeline::*;
pub use routes::*;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: Option<AppDatabase>,
}
