pub mod health;
pub mod notes;

use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub fn app_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health_handler))
        .route("/notes", get(notes::list_notes).post(notes::create_note))
        .route(
            "/notes/{id}",
            get(notes::get_note)
                .put(notes::update_note)
                .delete(notes::delete_note),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}