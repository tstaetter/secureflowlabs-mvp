mod db;
mod error;
mod handlers;
mod payloads;
pub mod routes;

pub use error::*;
pub use handlers::*;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: Option<mongodb::Database>,
}
