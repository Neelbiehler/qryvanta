//! AES-256-GCM encryptor for TOTP secrets at rest.

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use async_trait::async_trait;
use qryvanta_application::SecretEncryptor;
use qryvanta_core::{AppError, AppResult};

/// AES-256-GCM encryptor for protecting TOTP secrets in the database.
#[derive(Clone)]
pub struct AesSecretEncryptor {
    cipher: Aes256Gcm,
}

impl AesSecretEncryptor {
    /// Creates a new encryptor from a 32-byte key.
    pub fn new(key_bytes: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(key_bytes.into());
        Self { cipher }
    }

    /// Creates a new encryptor from a hex-encoded 32-byte key.
    pub fn from_hex(hex_key: &str) -> AppResult<Self> {
        let decoded = hex::decode(hex_key).map_err(|error| {
            AppError::Validation(format!("invalid TOTP_ENCRYPTION_KEY hex: {error}"))
        })?;

        if decoded.len() != 32 {
            return Err(AppError::Validation(
                "TOTP_ENCRYPTION_KEY must be exactly 32 bytes (64 hex chars)".to_owned(),
            ));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&decoded);
        Ok(Self::new(&key))
    }
}

#[async_trait]
impl SecretEncryptor for AesSecretEncryptor {
    fn encrypt(&self, plaintext: &[u8]) -> AppResult<Vec<u8>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|error| AppError::Internal(format!("failed to encrypt secret: {error}")))?;

        // Prepend the 12-byte nonce to the ciphertext for storage.
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> AppResult<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(AppError::Internal(
                "ciphertext too short: missing nonce".to_owned(),
            ));
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(12);
        let nonce_array: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| AppError::Internal("nonce must be exactly 12 bytes".to_owned()))?;
        let nonce = Nonce::from(nonce_array);

        self.cipher
            .decrypt(&nonce, encrypted)
            .map_err(|error| AppError::Internal(format!("failed to decrypt secret: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qryvanta_application::SecretEncryptor;

    #[test]
    fn encrypt_decrypt_roundtrip() -> AppResult<()> {
        let key = [42u8; 32];
        let encryptor = AesSecretEncryptor::new(&key);

        let plaintext = b"my-totp-secret";
        let encrypted = encryptor.encrypt(plaintext)?;
        let decrypted = encryptor.decrypt(&encrypted)?;

        assert_eq!(decrypted, plaintext);
        Ok(())
    }

    #[test]
    fn decrypt_with_wrong_key_fails() -> AppResult<()> {
        let key1 = [42u8; 32];
        let key2 = [99u8; 32];
        let encryptor1 = AesSecretEncryptor::new(&key1);
        let encryptor2 = AesSecretEncryptor::new(&key2);

        let encrypted = encryptor1.encrypt(b"secret")?;
        assert!(encryptor2.decrypt(&encrypted).is_err());
        Ok(())
    }
}
