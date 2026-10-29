#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Product JWT checks through the `OwnerId` extractor. A local axum server
//! stands in for the product JWKS endpoint, serving `tests/fixtures/jwks.json`,
//! and tokens are minted with the matching test-only private keys.

use axum::extract::FromRequestParts;
use axum::http::Request;
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use template_rust::api::extract::OwnerId;
use template_rust::core::config::AppConfig;
use template_rust::core::error::AppError;
use template_rust::core::jwt::JwtValidator;
use template_rust::state::AppState;

const ISSUER: &str = "https://app.example.com";
const AUDIENCE: &str = "template-rust";
const RSA_PEM: &[u8] = include_bytes!("fixtures/rsa-private.pem");
const EC_PEM: &[u8] = include_bytes!("fixtures/ec-private.pem");

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    exp: u64,
    iss: String,
    aud: String,
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn claims(sub: &str, exp_offset_secs: i64) -> TestClaims {
    TestClaims {
        sub: sub.to_string(),
        exp: unix_now().saturating_add_signed(exp_offset_secs),
        iss: ISSUER.to_string(),
        aud: AUDIENCE.to_string(),
    }
}

fn mint(pem: &[u8], alg: Algorithm, kid: &str, claims: &impl Serialize) -> String {
    let mut header = Header::new(alg);
    header.kid = Some(kid.to_string());
    let key = match alg {
        Algorithm::RS256 => EncodingKey::from_rsa_pem(pem).unwrap(),
        Algorithm::ES256 => EncodingKey::from_ec_pem(pem).unwrap(),
        _ => unreachable!("tests mint only RS256 and ES256"),
    };
    encode(&header, claims, &key).unwrap()
}

/// Serves the fixture JWKS on an ephemeral port and returns its URL.
async fn jwks_url() -> String {
    let jwks: serde_json::Value = serde_json::from_str(include_str!("fixtures/jwks.json")).unwrap();
    let app = axum::Router::new().route(
        "/jwks.json",
        axum::routing::get(move || {
            let jwks = jwks.clone();
            async move { axum::Json(jwks) }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/jwks.json", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}

fn state_with_jwt(jwks_url: &str) -> AppState {
    state_with_jwt_and_trust(jwks_url, false)
}

fn state_with_jwt_and_trust(jwks_url: &str, trust_identity_header: bool) -> AppState {
    let config = AppConfig {
        product_jwks_url: Some(jwks_url.to_string()),
        product_jwt_issuer: Some(ISSUER.to_string()),
        product_jwt_audience: Some(AUDIENCE.to_string()),
        trust_identity_header,
        ..AppConfig::default()
    };
    let jwt = JwtValidator::from_config(&config).unwrap();
    assert!(jwt.is_some());
    let pool = sqlx::PgPool::connect_lazy(&config.database_url).unwrap();
    AppState::new(config, pool, jwt)
}

async fn extract(
    state: &AppState,
    builder: axum::http::request::Builder,
) -> Result<OwnerId, AppError> {
    let (mut parts, _) = builder.body(()).unwrap().into_parts();
    OwnerId::from_request_parts(&mut parts, state).await
}

fn bearer(token: &str) -> axum::http::request::Builder {
    Request::builder().header("authorization", format!("Bearer {token}"))
}

#[tokio::test]
async fn valid_rsa_bearer_identifies_the_subject() {
    let state = state_with_jwt(&jwks_url().await);
    let token = mint(
        RSA_PEM,
        Algorithm::RS256,
        "test-rsa",
        &claims("user-1", 3600),
    );
    let OwnerId(subject) = extract(&state, bearer(&token)).await.unwrap();
    assert_eq!(subject, "user-1");
}

#[tokio::test]
async fn valid_ec_bearer_identifies_the_subject() {
    let state = state_with_jwt(&jwks_url().await);
    let token = mint(EC_PEM, Algorithm::ES256, "test-ec", &claims("user-2", 3600));
    let OwnerId(subject) = extract(&state, bearer(&token)).await.unwrap();
    assert_eq!(subject, "user-2");
}

#[tokio::test]
async fn tampered_bearer_is_unauthorized() {
    let state = state_with_jwt(&jwks_url().await);
    let mut token = mint(
        RSA_PEM,
        Algorithm::RS256,
        "test-rsa",
        &claims("user-1", 3600),
    );
    let last = token.pop().unwrap();
    token.push(if last == 'A' { 'B' } else { 'A' });
    assert!(matches!(
        extract(&state, bearer(&token)).await,
        Err(AppError::Unauthorized)
    ));
}

#[tokio::test]
async fn expired_wrong_scope_and_unknown_key_bearers_are_unauthorized() {
    let state = state_with_jwt(&jwks_url().await);
    let expired = mint(
        RSA_PEM,
        Algorithm::RS256,
        "test-rsa",
        &claims("user-1", -3600),
    );
    let mut wrong_aud = claims("user-1", 3600);
    wrong_aud.aud = "someone-else".to_string();
    let wrong_aud = mint(RSA_PEM, Algorithm::RS256, "test-rsa", &wrong_aud);
    let mut wrong_iss = claims("user-1", 3600);
    wrong_iss.iss = "https://evil.example.com".to_string();
    let wrong_iss = mint(RSA_PEM, Algorithm::RS256, "test-rsa", &wrong_iss);
    let unknown_kid = mint(
        RSA_PEM,
        Algorithm::RS256,
        "retired-key",
        &claims("user-1", 3600),
    );
    let missing_sub = mint(
        RSA_PEM,
        Algorithm::RS256,
        "test-rsa",
        &serde_json::json!({"exp": unix_now() + 3600, "iss": ISSUER, "aud": AUDIENCE}),
    );
    for token in [expired, wrong_aud, wrong_iss, unknown_kid, missing_sub] {
        assert!(
            matches!(
                extract(&state, bearer(&token)).await,
                Err(AppError::Unauthorized)
            ),
            "token must be rejected: {token}"
        );
    }
}

#[tokio::test]
async fn bearer_is_refused_without_a_configured_jwks() {
    let config = AppConfig {
        trust_identity_header: true,
        ..AppConfig::default()
    };
    let pool = sqlx::PgPool::connect_lazy(&config.database_url).unwrap();
    let state = AppState::new(config, pool, None);
    let token = mint(
        RSA_PEM,
        Algorithm::RS256,
        "test-rsa",
        &claims("user-1", 3600),
    );
    assert!(matches!(
        extract(&state, bearer(&token)).await,
        Err(AppError::Unauthorized)
    ));
}

#[tokio::test]
async fn invalid_bearer_does_not_fall_through_to_the_header() {
    let state = state_with_jwt_and_trust(&jwks_url().await, true);
    let req = Request::builder()
        .header("authorization", "Bearer not-a-token")
        .header("x-user-id", "owner-a");
    assert!(matches!(
        extract(&state, req).await,
        Err(AppError::Unauthorized)
    ));
}

#[tokio::test]
async fn unreachable_jwks_is_unauthorized() {
    let closed = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = closed.local_addr().unwrap().port();
    drop(closed);
    let state = state_with_jwt(&format!("http://127.0.0.1:{port}/jwks.json"));
    let token = mint(
        RSA_PEM,
        Algorithm::RS256,
        "test-rsa",
        &claims("user-1", 3600),
    );
    assert!(matches!(
        extract(&state, bearer(&token)).await,
        Err(AppError::Unauthorized)
    ));
}
