use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: OffsetDateTime,
    pub version: String,
    pub environment: String,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "The service is running", body = HealthResponse),
    )
)]
pub async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: OffsetDateTime::now_utc(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: state.config.app_env.clone(),
    })
}
