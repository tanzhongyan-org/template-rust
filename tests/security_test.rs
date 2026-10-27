#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use template_rust::api::app_router;
use template_rust::core::config::AppConfig;
use template_rust::state::AppState;
use tower::ServiceExt;

/// Builds the router with a lazy pool: every request below is rejected before any query runs.
fn app(config: AppConfig) -> axum::Router {
    let pool = sqlx::PgPool::connect_lazy(&config.database_url).unwrap();
    app_router(AppState::new(config, pool))
}

fn trusting() -> AppConfig {
    AppConfig {
        trust_identity_header: true,
        ..AppConfig::default()
    }
}

async fn status_and_message(res: axum::response::Response) -> (StatusCode, String) {
    let status = res.status();
    let body = to_bytes(res.into_body(), 1024 * 16).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"]["status"], status.as_u16());
    (
        status,
        json["error"]["message"].as_str().unwrap().to_string(),
    )
}

fn get_notes(user: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().uri("/notes");
    if let Some(user) = user {
        builder = builder.header("x-user-id", user);
    }
    builder.body(Body::empty()).unwrap()
}

#[tokio::test]
async fn missing_identity_header_is_unauthorized() {
    let res = app(trusting()).oneshot(get_notes(None)).await.unwrap();
    let (status, message) = status_and_message(res).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(message, "Authentication required");
}

#[tokio::test]
async fn blank_identity_header_is_unauthorized() {
    let res = app(trusting())
        .oneshot(get_notes(Some("   ")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn identity_header_is_ignored_unless_trusted() {
    let res = app(AppConfig::default())
        .oneshot(get_notes(Some("owner-a")))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn oversized_body_is_rejected_with_error_envelope() {
    let big = format!(r#"{{"title":"t","body":"{}"}}"#, "x".repeat(70 * 1024));
    let req = Request::builder()
        .method("POST")
        .uri("/notes")
        .header("content-type", "application/json")
        .header("x-user-id", "owner-a")
        .body(Body::from(big))
        .unwrap();
    let res = app(trusting()).oneshot(req).await.unwrap();
    let (status, _) = status_and_message(res).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn malformed_json_uses_error_envelope() {
    let req = Request::builder()
        .method("POST")
        .uri("/notes")
        .header("content-type", "application/json")
        .header("x-user-id", "owner-a")
        .body(Body::from("{not json"))
        .unwrap();
    let res = app(trusting()).oneshot(req).await.unwrap();
    let (status, message) = status_and_message(res).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(message, "Invalid JSON body");
}

#[tokio::test]
async fn unknown_route_uses_error_envelope() {
    let req = Request::builder().uri("/nope").body(Body::empty()).unwrap();
    let res = app(trusting()).oneshot(req).await.unwrap();
    let (status, _) = status_and_message(res).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

fn health_from(
    origin: &str,
    config: AppConfig,
) -> impl std::future::Future<Output = axum::response::Response> {
    let req = Request::builder()
        .uri("/health")
        .header("origin", origin)
        .body(Body::empty())
        .unwrap();
    async move { app(config).oneshot(req).await.unwrap() }
}

#[tokio::test]
async fn cors_allows_only_listed_origins() {
    let config = AppConfig {
        cors_allowed_origins: vec!["https://app.example.com".to_string()],
        ..AppConfig::default()
    };
    let allowed = health_from("https://app.example.com", config.clone()).await;
    assert_eq!(
        allowed
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "https://app.example.com"
    );
    let denied = health_from("https://evil.example.com", config).await;
    assert!(
        denied
            .headers()
            .get("access-control-allow-origin")
            .is_none()
    );
}

#[tokio::test]
async fn cors_is_off_when_no_origins_are_configured() {
    let res = health_from("https://app.example.com", AppConfig::default()).await;
    assert!(res.headers().get("access-control-allow-origin").is_none());
}
