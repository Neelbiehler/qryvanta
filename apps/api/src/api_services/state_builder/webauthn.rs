use std::sync::Arc;

use qryvanta_core::AppError;
use url::Url;
use webauthn_rs::{Webauthn, WebauthnBuilder};

use crate::api_config::ApiConfig;

pub(super) fn build_webauthn(config: &ApiConfig) -> Result<Arc<Webauthn>, AppError> {
    let webauthn_origin = Url::parse(&config.webauthn_rp_origin)
        .map_err(|error| AppError::Validation(format!("invalid WEBAUTHN_RP_ORIGIN: {error}")))?;

    Ok(Arc::new(
        WebauthnBuilder::new(&config.webauthn_rp_id, &webauthn_origin)
            .map_err(|error| {
                AppError::Validation(format!("invalid WebAuthn relying party config: {error}"))
            })?
            .rp_name("Qryvanta")
            .build()
            .map_err(|error| {
                AppError::Internal(format!("failed to initialize WebAuthn runtime: {error}"))
            })?,
    ))
}
