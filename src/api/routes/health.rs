use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub use health_handler as health_check;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_handler_response() {
        let response = health_handler().await;
        assert_eq!(response.status, "ok");
        assert_eq!(response.version, env!("CARGO_PKG_VERSION"));
    }
}
