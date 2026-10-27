//! Request extractors shared by every handler. Reuse these instead of reading
//! headers or bodies directly, so every endpoint rejects bad input the same way.

use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Request},
    http::{StatusCode, request::Parts},
};
use serde::de::DeserializeOwned;

use crate::{core::error::AppError, state::AppState};

const IDENTITY_HEADER: &str = "x-user-id";

/// The authenticated caller. Rejects with 401 unless `TRUST_IDENTITY_HEADER=true`
/// and the `x-user-id` header carries a non-blank value.
#[derive(Debug, Clone)]
pub struct OwnerId(pub String);

// TEMPLATE-TODO(auth): replace header identity with real authentication
impl FromRequestParts<AppState> for OwnerId {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
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
