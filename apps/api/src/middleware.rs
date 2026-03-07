use std::time::Instant;

use std::net::SocketAddr;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderValue, Method, header};
use axum::middleware::Next;
use axum::response::Response;
use ipnet::IpNet;
use qryvanta_application::{RateLimitRule, UserRecord};
use qryvanta_core::{AppError, UserIdentity};
use tower_sessions::Session;
use tracing::warn;
use uuid::Uuid;

use crate::auth::session_helpers::constant_time_eq;
use crate::auth::{SESSION_CREATED_AT_KEY, SESSION_USER_KEY};
use crate::error::ApiResult;
use crate::state::AppState;

/// Maximum absolute session lifetime (8 hours).
/// OWASP Session Management Cheat Sheet: enforce absolute timeout regardless
/// of activity to limit the window for session hijacking.
const ABSOLUTE_SESSION_TIMEOUT_SECONDS: i64 = 8 * 60 * 60;
const TRACE_ID_HEADER: &str = "x-trace-id";

#[derive(Debug, Clone)]
pub struct RequestTraceContext {
    trace_id: String,
}

impl RequestTraceContext {
    #[must_use]
    pub fn trace_id(&self) -> &str {
        self.trace_id.as_str()
    }
}

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

pub async fn trace_and_observe(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let trace_id = request
        .headers()
        .get(TRACE_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(generate_trace_id);

    request.extensions_mut().insert(RequestTraceContext {
        trace_id: trace_id.clone(),
    });
    let trace_id = request
        .extensions()
        .get::<RequestTraceContext>()
        .map(|context| context.trace_id().to_owned())
        .unwrap_or(trace_id);

    let method = request.method().clone();
    let path = request.uri().path().to_owned();

    state.observability_metrics.on_request_start();
    let started = Instant::now();
    let mut response = next.run(request).await;
    let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);

    state.observability_metrics.on_request_end(
        response.status().as_u16(),
        elapsed_ms,
        state.slow_request_threshold_ms,
    );

    if elapsed_ms >= state.slow_request_threshold_ms {
        warn!(
            trace_id = %trace_id,
            method = %method,
            path = %path,
            status = response.status().as_u16(),
            elapsed_ms,
            threshold_ms = state.slow_request_threshold_ms,
            "slow http request detected"
        );
    }

    if let Ok(header_value) = HeaderValue::from_str(trace_id.as_str()) {
        response.headers_mut().insert(TRACE_ID_HEADER, header_value);
    }

    response
}

pub async fn apply_security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    write_security_headers(response.headers_mut());

    response
}

fn write_security_headers(headers: &mut axum::http::HeaderMap) {
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        header::HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        header::HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'; base-uri 'none'"),
    );
    headers.insert(
        header::HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()",
        ),
    );
}

pub async fn require_auth(
    State(state): State<AppState>,
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
    let created_at = session
        .get::<i64>(SESSION_CREATED_AT_KEY)
        .await
        .map_err(|error| {
            AppError::Internal(format!("failed to read session creation time: {error}"))
        })?;

    let created_at = match created_at {
        Some(created_at) => created_at,
        None => return delete_session_and_reject(&session, "session expired").await,
    };

    let elapsed = chrono::Utc::now().timestamp() - created_at;
    if elapsed > ABSOLUTE_SESSION_TIMEOUT_SECONDS {
        return delete_session_and_reject(&session, "session expired").await;
    }

    let user = state
        .user_service
        .find_by_subject(identity.subject())
        .await?
        .ok_or_else(|| AppError::Unauthorized("authentication required".to_owned()))?;

    if session_is_revoked(
        session_created_at(created_at),
        session_revocation_cutoff(&user),
    ) {
        return delete_session_and_reject(&session, "session revoked").await;
    }

    request.extensions_mut().insert(identity);
    Ok(next.run(request).await)
}

pub async fn require_same_origin_for_mutations(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> ApiResult<Response> {
    if request.uri().path().starts_with("/api/internal/worker/")
        || request
            .uri()
            .path()
            .starts_with("/api/public/workflows/webhooks/")
        || request
            .uri()
            .path()
            .starts_with("/api/public/workflows/forms/")
        || request
            .uri()
            .path()
            .starts_with("/api/public/workflows/email/")
        || request
            .uri()
            .path()
            .starts_with("/api/public/workflows/approvals/")
    {
        return Ok(next.run(request).await);
    }

    let requires_same_origin = is_state_changing_method(request.method())
        || request.uri().path() == "/auth/webauthn/login/start";

    if requires_same_origin {
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

    if !constant_time_eq(provided_secret, configured_secret) {
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

    let ip = extract_client_ip(
        &request,
        state.trust_proxy_headers,
        &state.trusted_proxy_cidrs,
    );
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
fn extract_client_ip<T>(
    request: &axum::http::Request<T>,
    trust_proxy_headers: bool,
    trusted_proxy_cidrs: &[IpNet],
) -> String {
    let socket_addr = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0);

    extract_client_ip_from_parts(
        request.headers(),
        socket_addr,
        trust_proxy_headers,
        trusted_proxy_cidrs,
    )
}

pub(crate) fn extract_client_ip_from_parts(
    headers: &axum::http::HeaderMap,
    socket_addr: Option<SocketAddr>,
    trust_proxy_headers: bool,
    trusted_proxy_cidrs: &[IpNet],
) -> String {
    if proxy_headers_are_trusted(socket_addr, trust_proxy_headers, trusted_proxy_cidrs) {
        extract_forwarded_ip(headers).unwrap_or_else(|| "proxy-unknown".to_owned())
    } else {
        socket_addr
            .map(|socket_addr| socket_addr.ip().to_string())
            .unwrap_or_else(|| "direct-unknown".to_owned())
    }
}

fn proxy_headers_are_trusted(
    socket_addr: Option<SocketAddr>,
    trust_proxy_headers: bool,
    trusted_proxy_cidrs: &[IpNet],
) -> bool {
    trust_proxy_headers
        && socket_addr
            .map(|socket_addr| {
                trusted_proxy_cidrs
                    .iter()
                    .any(|cidr| cidr.contains(&socket_addr.ip()))
            })
            .unwrap_or(false)
}

fn extract_forwarded_ip(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|forwarded| forwarded.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
}

fn generate_trace_id() -> String {
    format!("api-{}", Uuid::new_v4())
}

fn session_created_at(timestamp: i64) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp, 0)
}

fn session_revocation_cutoff(user: &UserRecord) -> Option<chrono::DateTime<chrono::Utc>> {
    user.password_changed_at
        .iter()
        .chain(user.auth_sessions_revoked_after.iter())
        .max()
        .cloned()
}

fn session_is_revoked(
    session_created_at: Option<chrono::DateTime<chrono::Utc>>,
    revocation_cutoff: Option<chrono::DateTime<chrono::Utc>>,
) -> bool {
    match revocation_cutoff {
        Some(revocation_cutoff) => match session_created_at {
            Some(session_created_at) => session_created_at < revocation_cutoff,
            None => true,
        },
        None => false,
    }
}

async fn delete_session_and_reject(session: &Session, message: &str) -> ApiResult<Response> {
    session.delete().await.map_err(|error| {
        AppError::Internal(format!("failed to delete expired session: {error}"))
    })?;

    Err(AppError::Unauthorized(message.to_owned()).into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::ConnectInfo;
    use axum::http::HeaderMap;
    use qryvanta_domain::UserId;

    #[test]
    fn extract_client_ip_prefers_socket_addr_when_proxy_headers_are_disabled() {
        let mut request = Request::new(axum::body::Body::empty());
        request
            .headers_mut()
            .insert("x-forwarded-for", HeaderValue::from_static("203.0.113.10"));
        request.extensions_mut().insert(ConnectInfo(
            "198.51.100.4:443"
                .parse::<SocketAddr>()
                .unwrap_or_else(|_| unreachable!()),
        ));

        let client_ip = extract_client_ip(&request, false, &[]);
        assert_eq!(client_ip, "198.51.100.4");
    }

    #[test]
    fn extract_client_ip_uses_forwarded_headers_when_proxy_is_trusted() {
        let mut request = Request::new(axum::body::Body::empty());
        request.headers_mut().insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.10, 198.51.100.2"),
        );

        request.extensions_mut().insert(ConnectInfo(
            "10.0.0.5:443"
                .parse::<SocketAddr>()
                .unwrap_or_else(|_| unreachable!()),
        ));

        let trusted_proxy_cidrs = vec![
            "10.0.0.0/24"
                .parse::<IpNet>()
                .unwrap_or_else(|_| unreachable!()),
        ];
        let client_ip = extract_client_ip(&request, true, &trusted_proxy_cidrs);
        assert_eq!(client_ip, "203.0.113.10");
    }

    #[test]
    fn extract_client_ip_ignores_forwarded_headers_from_untrusted_peer() {
        let mut request = Request::new(axum::body::Body::empty());
        request.headers_mut().insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.10, 198.51.100.2"),
        );
        request.extensions_mut().insert(ConnectInfo(
            "198.51.100.4:443"
                .parse::<SocketAddr>()
                .unwrap_or_else(|_| unreachable!()),
        ));

        let trusted_proxy_cidrs = vec![
            "10.0.0.0/24"
                .parse::<IpNet>()
                .unwrap_or_else(|_| unreachable!()),
        ];
        let client_ip = extract_client_ip(&request, true, &trusted_proxy_cidrs);
        assert_eq!(client_ip, "198.51.100.4");
    }

    #[test]
    fn write_security_headers_sets_hardening_defaults() {
        let mut headers = HeaderMap::new();
        write_security_headers(&mut headers);

        assert_eq!(
            headers.get("x-content-type-options"),
            Some(&HeaderValue::from_static("nosniff"))
        );
        assert_eq!(
            headers.get("x-frame-options"),
            Some(&HeaderValue::from_static("DENY"))
        );
        assert_eq!(
            headers.get("referrer-policy"),
            Some(&HeaderValue::from_static("strict-origin-when-cross-origin"))
        );
        assert_eq!(
            headers.get("content-security-policy"),
            Some(&HeaderValue::from_static(
                "default-src 'none'; frame-ancestors 'none'; base-uri 'none'"
            ))
        );
        assert!(headers.contains_key("permissions-policy"));
    }

    #[test]
    fn session_revocation_cutoff_prefers_latest_security_event() {
        let password_changed_at = chrono::Utc::now();
        let auth_sessions_revoked_after = password_changed_at + chrono::Duration::minutes(5);
        let user = sample_user(Some(password_changed_at), Some(auth_sessions_revoked_after));

        assert_eq!(
            session_revocation_cutoff(&user),
            Some(auth_sessions_revoked_after)
        );
    }

    #[test]
    fn session_is_revoked_when_created_before_cutoff() {
        let created_at = chrono::Utc::now();
        let cutoff = created_at + chrono::Duration::seconds(1);

        assert!(session_is_revoked(Some(created_at), Some(cutoff)));
        assert!(!session_is_revoked(Some(cutoff), Some(cutoff)));
    }

    #[test]
    fn session_without_creation_timestamp_is_rejected_when_cutoff_exists() {
        let cutoff = chrono::Utc::now();
        assert!(session_is_revoked(None, Some(cutoff)));
    }

    fn sample_user(
        password_changed_at: Option<chrono::DateTime<chrono::Utc>>,
        auth_sessions_revoked_after: Option<chrono::DateTime<chrono::Utc>>,
    ) -> UserRecord {
        UserRecord {
            id: UserId::new(),
            email: "user@example.com".to_owned(),
            email_verified: true,
            password_hash: Some("hash".to_owned()),
            totp_enabled: false,
            totp_secret_enc: None,
            recovery_codes_hash: None,
            totp_pending_secret_enc: None,
            recovery_codes_pending_hash: None,
            failed_login_count: 0,
            locked_until: None,
            password_changed_at,
            auth_sessions_revoked_after,
            default_tenant_id: None,
        }
    }
}
