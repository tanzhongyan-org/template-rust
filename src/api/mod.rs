pub mod routes;

use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn app_router() -> Router {
    Router::new()
        .route("/health", get(routes::health::health_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

pub use app_router as create_router;
