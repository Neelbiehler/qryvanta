use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use qryvanta_core::AppError;
use serde::Serialize;
use ts_rs::TS;

/// API error payload.
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = "../../../packages/api-types/src/generated/error-response.ts"
)]
pub struct ErrorResponse {
    message: String,
}

/// HTTP API error wrapper around core application errors.
#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let payload = Json(ErrorResponse {
            message: self.0.to_string(),
        });

        (status, payload).into_response()
    }
}

/// Standard API result type.
pub type ApiResult<T> = Result<T, ApiError>;
