use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, header};
use axum::middleware::Next;
use axum::response::Response;
use qryvanta_application::RateLimitRule;
use qryvanta_core::{AppError, UserIdentity};
use tower_sessions::Session;

use crate::auth::SESSION_USER_KEY;
use crate::error::ApiResult;
use crate::state::AppState;

/// Maximum absolute session lifetime (8 hours).
/// OWASP Session Management Cheat Sheet: enforce absolute timeout regardless
/// of activity to limit the window for session hijacking.
const ABSOLUTE_SESSION_TIMEOUT_SECONDS: i64 = 8 * 60 * 60;

#[derive(Debug, Clone)]
pub struct WorkerIdentity {
    worker_id: String,
}

impl WorkerIdentity {
    #[must_use]
    pub fn worker_id(&self) -> &str {
        self.worker_id.as_str()
    }
}

pub async fn require_auth(
    session: Session,
    mut request: Request,
    next: Next,
) -> ApiResult<Response> {
    let identity = session
        .get::<UserIdentity>(SESSION_USER_KEY)
        .await
        .map_err(|error| AppError::Internal(format!("failed to read session identity: {error}")))?
        .ok_or_else(|| AppError::Unauthorized("authentication required".to_owned()))?;

    // OWASP Session Management: enforce absolute session timeout.
    if let Some(created_at) = session
        .get::<i64>(crate::auth::SESSION_CREATED_AT_KEY)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to read session creation time: {error}"))
        })?
    {
        let elapsed = chrono::Utc::now().timestamp() - created_at;
        if elapsed > ABSOLUTE_SESSION_TIMEOUT_SECONDS {
            session.delete().await.map_err(|error| {
                AppError::Internal(format!("failed to delete expired session: {error}"))
            })?;
            return Err(AppError::Unauthorized("session expired".to_owned()).into());
        }
    }

    request.extensions_mut().insert(identity);
    Ok(next.run(request).await)
}

pub async fn require_same_origin_for_mutations(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> ApiResult<Response> {
    if request.uri().path().starts_with("/api/internal/worker/") {
        return Ok(next.run(request).await);
    }

    if is_state_changing_method(request.method()) {
        let headers = request.headers();

        if let Some(fetch_site) = headers.get("sec-fetch-site")
            && fetch_site == HeaderValue::from_static("cross-site")
        {
            return Err(AppError::Unauthorized("cross-site request blocked".to_owned()).into());
        }

        let origin = headers
            .get(header::ORIGIN)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        let referer = headers
            .get(header::REFERER)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();

        let allowed_origin = state.frontend_url;
        let origin_is_allowed = origin == allowed_origin;
        let referer_is_allowed = referer.starts_with(&allowed_origin);

        if !origin_is_allowed && !referer_is_allowed {
            return Err(AppError::Unauthorized("origin validation failed".to_owned()).into());
        }
    }

    Ok(next.run(request).await)
}

pub async fn require_worker_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> ApiResult<Response> {
    let configured_secret = state
        .worker_shared_secret
        .as_deref()
        .ok_or_else(|| AppError::Unauthorized("worker auth is not configured".to_owned()))?;

    let authorization_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("worker authorization header missing".to_owned()))?;

    let provided_secret = authorization_header
        .strip_prefix("Bearer ")
        .map(str::trim)
        .ok_or_else(|| AppError::Unauthorized("worker auth scheme must be Bearer".to_owned()))?;

    if provided_secret != configured_secret {
        return Err(AppError::Unauthorized("worker auth token is invalid".to_owned()).into());
    }

    let worker_id = request
        .headers()
        .get("x-qryvanta-worker-id")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Unauthorized("x-qryvanta-worker-id header is required".to_owned())
        })?
        .to_owned();

    request
        .extensions_mut()
        .insert(WorkerIdentity { worker_id });

    Ok(next.run(request).await)
}

fn is_state_changing_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

/// Rate limiting middleware for auth endpoints.
///
/// Extracts the client IP from `X-Forwarded-For` or falls back to an
/// opaque key, then checks the rate limit using the provided rule
/// (injected via `Extension<RateLimitRule>`).
///
/// OWASP Credential Stuffing Prevention: limits login, registration,
/// and password reset attempts per IP.
pub async fn rate_limit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> ApiResult<Response> {
    let rule = request
        .extensions()
        .get::<RateLimitRule>()
        .cloned()
        .ok_or_else(|| {
            AppError::Internal(
                "rate limit middleware misconfigured: missing RateLimitRule extension".to_owned(),
            )
        })?;

    let ip = extract_client_ip(&request);
    state
        .rate_limit_service
        .check_rate_limit(&rule, &ip)
        .await?;

    Ok(next.run(request).await)
}

/// Extracts the client IP address from request headers.
///
/// Prefers `X-Forwarded-For` (first entry) for reverse-proxy setups,
/// falls back to `X-Real-Ip`, then to `"unknown"`.
fn extract_client_ip(request: &Request) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|forwarded| forwarded.split(',').next())
        .map(|ip| ip.trim().to_owned())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
                .map(|ip| ip.trim().to_owned())
        })
        .unwrap_or_else(|| "unknown".to_owned())
}
