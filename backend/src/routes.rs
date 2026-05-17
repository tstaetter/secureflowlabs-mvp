use axum::Router;

pub fn app(state: AppState) -> Router {
    let routes = Router::new()
        .route("/create-upload", post(create_upload))
        .layer(CorsLayer::permissive())
        .with_state(state);

    Router::new()
        .route("/health", get(health))
        .nest("/v1", routes)
}
