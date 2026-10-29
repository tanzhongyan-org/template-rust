use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// The body of every error response: `{"error": {"message": "...", "status": 404}}`.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
}

/// The inner error object, shared by every endpoint.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    pub message: String,
    pub status: u16,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Request body too large")]
    PayloadTooLarge,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Config(err) => {
                tracing::error!(error = %err, "Configuration error occurred");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::PayloadTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, self.to_string()),
            AppError::Internal(err) => {
                tracing::error!(error = %err, "Internal server error occurred");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(ErrorEnvelope {
            error: ErrorBody {
                message,
                status: status.as_u16(),
            },
        });

        (status, body).into_response()
    }
}
