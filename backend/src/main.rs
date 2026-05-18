use backend::{AppDatabase, app};
use backend::{AppResult, AppState};
use tokio::signal;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Load env vars
    dotenvy::dotenv().ok();
    // Initialize tracing
    init_tracing();

    // Database setup
    let db = AppDatabase::try_new().await?;
    // Create app state
    let state = AppState { db: Some(db) };
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());

    info!("Binding to 0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!("Server listening on 0.0.0.0:{}, health at /health", port);

    // Graceful shutdown: listen for SIGTERM (Koyeb) or SIGINT (Ctrl+C locally)
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// Initialize tracing
fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_target(true)
                .with_level(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_writer(std::io::stdout),
        )
        .init();

    info!("Tracing configured");
}

/// Returns a future that completes when a shutdown signal is received.
/// Handles SIGTERM (sent by Koyeb and other container orchestrators)
/// and SIGINT (Ctrl+C in local development).
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            info!("Received SIGINT, shutting down...");
        },
        () = terminate => {
            info!("Received SIGTERM, shutting down...");
        },
    }
}
