use qryvanta_core::AppError;

use super::*;

/// Generates a cryptographically random token and its SHA-256 hash.
///
/// Returns `(raw_token_hex, sha256_hash_hex)`.
pub(super) fn generate_token() -> AppResult<(String, String)> {
    use std::fmt::Write;

    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|error| AppError::Internal(format!("failed to generate auth token: {error}")))?;

    let raw_token = bytes
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            let _ = write!(acc, "{byte:02x}");
            acc
        });

    let hash = hash_token(&raw_token);
    Ok((raw_token, hash))
}

/// Computes the SHA-256 hash of a token string for storage.
pub(super) fn hash_token(raw_token: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;

    let mut hasher = Sha256::new();
    hasher.update(raw_token.as_bytes());
    let result = hasher.finalize();

    result
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}
