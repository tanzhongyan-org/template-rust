pub mod health;

use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use crate::state::AppState;

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
