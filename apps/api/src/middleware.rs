use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, header};
use axum::middleware::Next;
use axum::response::Response;
use qryvanta_core::{AppError, UserIdentity};
use tower_sessions::Session;

use crate::auth::SESSION_USER_KEY;
use crate::error::ApiResult;
use crate::state::AppState;

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

    request.extensions_mut().insert(identity);
    Ok(next.run(request).await)
}

pub async fn require_same_origin_for_mutations(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> ApiResult<Response> {
    if is_state_changing_method(request.method()) {
        let headers = request.headers();

        if let Some(fetch_site) = headers.get("sec-fetch-site") {
            if fetch_site == HeaderValue::from_static("cross-site") {
                return Err(AppError::Unauthorized("cross-site request blocked".to_owned()).into());
            }
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

fn is_state_changing_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}
