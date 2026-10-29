//! Request extractors shared by every handler. Reuse these instead of reading
//! headers or bodies directly, so every endpoint rejects bad input the same way.

use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Request},
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
};
use serde::de::DeserializeOwned;

use crate::{core::error::AppError, state::AppState};

const IDENTITY_HEADER: &str = "x-user-id";

/// The authenticated caller: the subject of a valid product JWT, or the
/// `x-user-id` header when `TRUST_IDENTITY_HEADER=true`. Anything else is 401.
#[derive(Debug, Clone)]
pub struct OwnerId(pub String);

impl FromRequestParts<AppState> for OwnerId {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(token) = bearer_token(parts) {
            let validator = state.jwt.as_ref().ok_or(AppError::Unauthorized)?;
            let subject = validator
                .validate(&token)
                .await
                .map_err(|_| AppError::Unauthorized)?;
            return Ok(OwnerId(subject));
        }
        // TEMPLATE-TODO(auth): replace header identity with real authentication
        if !state.config.trust_identity_header {
            return Err(AppError::Unauthorized);
        }
        parts
            .headers
            .get(IDENTITY_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|v| OwnerId(v.to_string()))
            .ok_or(AppError::Unauthorized)
    }
}

/// Reads the bearer token from the `Authorization` header, if one is present.
fn bearer_token(parts: &Parts) -> Option<String> {
    let value = parts.headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

/// JSON body extractor whose rejections use the standard error envelope.
#[derive(Debug)]
pub struct ApiJson<T>(pub T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(ApiJson(value)),
            Err(rejection) if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE => {
                Err(AppError::PayloadTooLarge)
            }
            Err(_) => Err(AppError::BadRequest("Invalid JSON body".to_string())),
        }
    }
}
