use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, Method};
use qryvanta_core::AppError;
use tower_http::cors::CorsLayer;

pub(super) fn build_cors_layer(frontend_url: &str) -> Result<CorsLayer, AppError> {
    Ok(CorsLayer::new()
        .allow_origin(
            HeaderValue::from_str(frontend_url)
                .map_err(|error| AppError::Internal(format!("invalid FRONTEND_URL: {error}")))?,
        )
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([CONTENT_TYPE]))
}
