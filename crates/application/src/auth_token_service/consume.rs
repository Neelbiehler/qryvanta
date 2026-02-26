use qryvanta_core::AppError;
use qryvanta_domain::AuthTokenType;

use super::token_crypto::hash_token;
use super::*;

impl AuthTokenService {
    /// Atomically validates and consumes a token.
    pub async fn consume_valid_token(
        &self,
        raw_token: &str,
        token_type: AuthTokenType,
    ) -> AppResult<AuthTokenRecord> {
        let token_hash = hash_token(raw_token);

        let record = self
            .token_repository
            .consume_valid_token(&token_hash, token_type)
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid or expired token".to_owned()))?;

        Ok(record)
    }
}
