use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use qryvanta_core::AppError;

mod types;

pub use types::ErrorResponse;

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
        let is_rate_limited = matches!(self.0, AppError::RateLimited(_));

        let status = match self.0 {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let payload = Json(ErrorResponse::new(self.0.to_string()));

        if is_rate_limited {
            // OWASP: include Retry-After header on 429 responses.
            (status, [("retry-after", "60")], payload).into_response()
        } else {
            (status, payload).into_response()
        }
    }
}

/// Standard API result type.
pub type ApiResult<T> = Result<T, ApiError>;
