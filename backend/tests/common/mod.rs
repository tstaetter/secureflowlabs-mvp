use axum_test::TestServer;
use backend::app;
use backend::AppState;

/// Build and return an `axum_test::TestServer` from the full app router.
/// This avoids binding to real ports — `axum_test` handles transport internally.
pub async fn spawn_app() -> TestServer {
    let state = AppState { db: None };
    TestServer::new(app(state).into_make_service())
}
