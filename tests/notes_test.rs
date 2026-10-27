#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use template_rust::api::app_router;
use template_rust::api::notes::Note;
use template_rust::core::config::AppConfig;
use template_rust::state::AppState;
use tower::ServiceExt;

async fn setup_app() -> axum::Router {
    let config = AppConfig {
        trust_identity_header: true,
        ..AppConfig::default()
    };
    let pool = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let state = AppState::new(config, pool);
    app_router(state)
}

#[tokio::test]
async fn test_notes_roundtrip_and_tenant_isolation() {
    let app = setup_app().await;

    // 1. Create a note for owner-a
    let create_payload = serde_json::json!({
        "title": "Owner A Note",
        "body": "Secret content for owner A"
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "application/json")
                .header("x-user-id", "owner-a")
                .body(axum::body::Body::from(create_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = to_bytes(res.into_body(), 1024 * 16).await.unwrap();
    let note_a: Note = serde_json::from_slice(&body).unwrap();
    assert_eq!(note_a.title, "Owner A Note");
    assert_eq!(note_a.owner_id, "owner-a");

    // 2. Owner A can fetch their note
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/notes/{}", note_a.id))
                .header("x-user-id", "owner-a")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Tenant Isolation: Owner B CANNOT read Owner A's note (404)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/notes/{}", note_a.id))
                .header("x-user-id", "owner-b")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // 4. Tenant Isolation: Owner B CANNOT delete Owner A's note (404)
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/notes/{}", note_a.id))
                .header("x-user-id", "owner-b")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // 5. Validation failure: empty title returns 400 Bad Request
    let bad_payload = serde_json::json!({
        "title": "   ",
        "body": "valid body"
    });
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/notes")
                .header("content-type", "application/json")
                .header("x-user-id", "owner-a")
                .body(axum::body::Body::from(bad_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // 6. Owner A deletes their note -> 204 No Content
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/notes/{}", note_a.id))
                .header("x-user-id", "owner-a")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}
