mod db;
mod error;
mod handlers;
mod openapi;
mod payloads;
mod routes;

pub use error::*;
pub use handlers::*;
pub use routes::*;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub db: Option<mongodb::Database>,
}
