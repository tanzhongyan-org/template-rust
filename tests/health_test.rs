#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use template_rust::api::app_router;
use template_rust::api::health::HealthResponse;
use template_rust::core::config::AppConfig;
use template_rust::state::AppState;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check_endpoint() {
    let config = AppConfig::default();
    let pool = sqlx::PgPool::connect_lazy(&config.database_url).unwrap();
    let state = AppState::new(config, pool, None);
    let app = app_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), 1024 * 16).await.unwrap();
    let health: HealthResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(health.status, "ok");
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(health.environment, "development");
}
