use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use qryvanta_core::AppError;

mod codes;
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
        let code = codes::error_code_for(&self.0);
        let is_rate_limited = matches!(self.0, AppError::RateLimited(_));

        let status = match &self.0 {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let payload = Json(ErrorResponse::new(code.to_owned(), self.0.to_string()));

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

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::ApiError;
    use qryvanta_core::AppError;

    #[tokio::test]
    async fn validation_response_contains_stable_publish_code() {
        let response = ApiError(AppError::Validation(
            "publish checks failed for entity 'contact':\n- entity 'contact' requires at least one field before publishing"
                .to_owned(),
        ))
        .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap_or_else(|_| unreachable!());
        let payload: serde_json::Value =
            serde_json::from_slice(body.as_ref()).unwrap_or_else(|_| unreachable!());

        assert_eq!(
            payload.get("code").and_then(serde_json::Value::as_str),
            Some("validation.publish.checks_failed")
        );
    }

    #[tokio::test]
    async fn rate_limited_response_sets_retry_after_header() {
        let response = ApiError(AppError::RateLimited(
            "runtime query backpressure active".to_owned(),
        ))
        .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            response
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok()),
            Some("60")
        );
    }
}
