use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_into_response() {
        let not_found = AppError::NotFound("test item".to_string());
        let res = not_found.into_response();
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        let bad_request = AppError::BadRequest("invalid input".to_string());
        let res = bad_request.into_response();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let config_err = AppError::Config("missing setting".to_string());
        let res = config_err.into_response();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let internal_err = AppError::Internal(anyhow::anyhow!("unexpected failure"));
        let res = internal_err.into_response();
        assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
