use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

/// API error types
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Feature requires license: {0}")]
    LicenseRequired(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Diagnostics error: {0}")]
    Diagnostics(String),
}

/// Standardized error response body
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status: u16,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ApiError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            ApiError::LicenseRequired(_) => (StatusCode::PAYMENT_REQUIRED, "license_required"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            ApiError::WebSocket(_) => (StatusCode::INTERNAL_SERVER_ERROR, "websocket_error"),
            ApiError::Diagnostics(_) => (StatusCode::INTERNAL_SERVER_ERROR, "diagnostics_error"),
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            status: status.as_u16(),
        };

        (status, axum::Json(body)).into_response()
    }
}

pub type ApiResult<T> = std::result::Result<T, ApiError>;
