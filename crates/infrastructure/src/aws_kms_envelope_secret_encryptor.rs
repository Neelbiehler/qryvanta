//! Envelope encryptor for TOTP secrets using AWS KMS-wrapped data keys.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use aes_gcm::aead::{Aead, KeyInit, OsRng, rand_core::RngCore};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use async_trait::async_trait;
use base64::Engine;
use qryvanta_application::SecretEncryptor;
use qryvanta_core::{AppError, AppResult};
use uuid::Uuid;

use crate::aes_secret_encryptor::AesSecretEncryptor;

const ENVELOPE_MAGIC: &[u8; 4] = b"QEK1";
const WRAPPED_KEY_LENGTH_BYTES: usize = 4;
const NONCE_LENGTH_BYTES: usize = 12;
const DATA_KEY_LENGTH_BYTES: usize = 32;

trait KeyManagementService: Clone + Send + Sync {
    fn wrap_key(&self, plaintext_key: &[u8; DATA_KEY_LENGTH_BYTES]) -> AppResult<Vec<u8>>;

    fn unwrap_key(&self, wrapped_key: &[u8]) -> AppResult<[u8; DATA_KEY_LENGTH_BYTES]>;
}

#[derive(Clone)]
struct AwsCliKeyManagementService {
    key_id: String,
}

impl AwsCliKeyManagementService {
    fn new(key_id: impl Into<String>) -> AppResult<Self> {
        let key_id = key_id.into();
        if key_id.trim().is_empty() {
            return Err(AppError::Validation(
                "TOTP_KMS_KEY_ID must not be empty when TOTP_ENCRYPTION_MODE=aws_kms_envelope"
                    .to_owned(),
            ));
        }

        Ok(Self { key_id })
    }
}

impl KeyManagementService for AwsCliKeyManagementService {
    fn wrap_key(&self, plaintext_key: &[u8; DATA_KEY_LENGTH_BYTES]) -> AppResult<Vec<u8>> {
        let plaintext_path = TempFile::new("kms-plaintext", plaintext_key)?;
        let plaintext_arg = format!("fileb://{}", plaintext_path.path().display());
        let stdout = run_command(
            "aws",
            &[
                "kms".to_owned(),
                "encrypt".to_owned(),
                "--key-id".to_owned(),
                self.key_id.clone(),
                "--plaintext".to_owned(),
                plaintext_arg,
                "--query".to_owned(),
                "CiphertextBlob".to_owned(),
                "--output".to_owned(),
                "text".to_owned(),
            ],
            "wrap AWS KMS data key",
        )?;

        decode_base64(stdout.trim(), "AWS KMS CiphertextBlob")
    }

    fn unwrap_key(&self, wrapped_key: &[u8]) -> AppResult<[u8; DATA_KEY_LENGTH_BYTES]> {
        let wrapped_path = TempFile::new("kms-ciphertext", wrapped_key)?;
        let wrapped_arg = format!("fileb://{}", wrapped_path.path().display());
        let stdout = run_command(
            "aws",
            &[
                "kms".to_owned(),
                "decrypt".to_owned(),
                "--ciphertext-blob".to_owned(),
                wrapped_arg,
                "--query".to_owned(),
                "Plaintext".to_owned(),
                "--output".to_owned(),
                "text".to_owned(),
            ],
            "unwrap AWS KMS data key",
        )?;

        let decoded = decode_base64(stdout.trim(), "AWS KMS Plaintext")?;
        let decoded_length = decoded.len();
        let decoded: [u8; DATA_KEY_LENGTH_BYTES] = decoded.try_into().map_err(|_| {
            AppError::Internal(format!(
                "AWS KMS returned {} bytes for decrypted data key, expected {DATA_KEY_LENGTH_BYTES}",
                decoded_length
            ))
        })?;
        Ok(decoded)
    }
}

#[derive(Clone)]
struct EnvelopeSecretEncryptor<K> {
    key_management_service: K,
    legacy_encryptor: Option<AesSecretEncryptor>,
}

impl<K> EnvelopeSecretEncryptor<K>
where
    K: KeyManagementService,
{
    fn new(key_management_service: K, legacy_encryptor: Option<AesSecretEncryptor>) -> Self {
        Self {
            key_management_service,
            legacy_encryptor,
        }
    }

    fn encrypt_envelope(&self, plaintext: &[u8]) -> AppResult<Vec<u8>> {
        let mut data_key = [0u8; DATA_KEY_LENGTH_BYTES];
        OsRng.fill_bytes(&mut data_key);

        let cipher = Aes256Gcm::new((&data_key).into());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|error| AppError::Internal(format!("failed to encrypt secret: {error}")))?;
        let wrapped_key = self.key_management_service.wrap_key(&data_key)?;

        let wrapped_key_length = u32::try_from(wrapped_key.len()).map_err(|_| {
            AppError::Internal("wrapped AWS KMS data key exceeded 4 GiB envelope limit".to_owned())
        })?;

        let mut result = Vec::with_capacity(
            ENVELOPE_MAGIC.len()
                + WRAPPED_KEY_LENGTH_BYTES
                + wrapped_key.len()
                + nonce.len()
                + ciphertext.len(),
        );
        result.extend_from_slice(ENVELOPE_MAGIC);
        result.extend_from_slice(&wrapped_key_length.to_be_bytes());
        result.extend_from_slice(&wrapped_key);
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt_envelope(&self, ciphertext: &[u8]) -> AppResult<Vec<u8>> {
        if !ciphertext.starts_with(ENVELOPE_MAGIC) {
            let Some(legacy_encryptor) = &self.legacy_encryptor else {
                return Err(AppError::Internal(
                    "legacy TOTP ciphertext requires TOTP_ENCRYPTION_KEY fallback when TOTP_ENCRYPTION_MODE=aws_kms_envelope"
                        .to_owned(),
                ));
            };
            return legacy_encryptor.decrypt(ciphertext);
        }

        let minimum_length = ENVELOPE_MAGIC.len() + WRAPPED_KEY_LENGTH_BYTES + NONCE_LENGTH_BYTES;
        if ciphertext.len() < minimum_length {
            return Err(AppError::Internal(
                "envelope ciphertext too short: missing wrapped key metadata or nonce".to_owned(),
            ));
        }

        let wrapped_key_length_offset = ENVELOPE_MAGIC.len();
        let wrapped_key_end_offset = wrapped_key_length_offset + WRAPPED_KEY_LENGTH_BYTES;
        let wrapped_key_length: [u8; WRAPPED_KEY_LENGTH_BYTES] = ciphertext
            [wrapped_key_length_offset..wrapped_key_end_offset]
            .try_into()
            .map_err(|_| AppError::Internal("invalid envelope wrapped-key length".to_owned()))?;
        let wrapped_key_length = u32::from_be_bytes(wrapped_key_length) as usize;
        let wrapped_key_offset = wrapped_key_end_offset;
        let nonce_offset = wrapped_key_offset + wrapped_key_length;
        let payload_offset = nonce_offset + NONCE_LENGTH_BYTES;

        if ciphertext.len() < payload_offset {
            return Err(AppError::Internal(
                "envelope ciphertext truncated before wrapped key and nonce".to_owned(),
            ));
        }

        let wrapped_key = &ciphertext[wrapped_key_offset..nonce_offset];
        let nonce_bytes: [u8; NONCE_LENGTH_BYTES] = ciphertext[nonce_offset..payload_offset]
            .try_into()
            .map_err(|_| AppError::Internal("invalid envelope nonce".to_owned()))?;
        let encrypted_payload = &ciphertext[payload_offset..];
        let data_key = self.key_management_service.unwrap_key(wrapped_key)?;
        let cipher = Aes256Gcm::new((&data_key).into());
        let nonce = Nonce::from(nonce_bytes);

        cipher
            .decrypt(&nonce, encrypted_payload)
            .map_err(|error| AppError::Internal(format!("failed to decrypt secret: {error}")))
    }
}

#[async_trait]
impl<K> SecretEncryptor for EnvelopeSecretEncryptor<K>
where
    K: KeyManagementService,
{
    fn encrypt(&self, plaintext: &[u8]) -> AppResult<Vec<u8>> {
        self.encrypt_envelope(plaintext)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> AppResult<Vec<u8>> {
        self.decrypt_envelope(ciphertext)
    }
}

/// AWS KMS envelope encryptor for protecting TOTP secrets in the database.
#[derive(Clone)]
pub struct AwsKmsEnvelopeSecretEncryptor {
    inner: EnvelopeSecretEncryptor<AwsCliKeyManagementService>,
}

impl AwsKmsEnvelopeSecretEncryptor {
    /// Creates an AWS KMS envelope encryptor.
    pub fn new(kms_key_id: &str, legacy_key_hex: Option<&str>) -> AppResult<Self> {
        let key_management_service = AwsCliKeyManagementService::new(kms_key_id)?;
        let legacy_encryptor = legacy_key_hex
            .filter(|value| !value.trim().is_empty())
            .map(AesSecretEncryptor::from_hex)
            .transpose()?;

        Ok(Self {
            inner: EnvelopeSecretEncryptor::new(key_management_service, legacy_encryptor),
        })
    }
}

#[async_trait]
impl SecretEncryptor for AwsKmsEnvelopeSecretEncryptor {
    fn encrypt(&self, plaintext: &[u8]) -> AppResult<Vec<u8>> {
        self.inner.encrypt(plaintext)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> AppResult<Vec<u8>> {
        self.inner.decrypt(ciphertext)
    }
}

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(prefix: &str, bytes: &[u8]) -> AppResult<Self> {
        let path = env::temp_dir().join(format!("qryvanta-{prefix}-{}", Uuid::new_v4()));
        fs::write(&path, bytes).map_err(|error| {
            AppError::Internal(format!(
                "failed to write temporary KMS input file '{}': {error}",
                path.display()
            ))
        })?;

        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        self.path.as_path()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn run_command(program: &str, args: &[String], action: &str) -> AppResult<String> {
    let output = Command::new(program)
        .args(args.iter().map(String::as_str))
        .output()
        .map_err(|error| {
            AppError::Internal(format!(
                "failed to execute {program} while attempting to {action}: {error}"
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        return Err(AppError::Internal(format!(
            "{program} failed while attempting to {action} with status {}{}",
            output.status,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        )));
    }

    String::from_utf8(output.stdout).map_err(|error| {
        AppError::Internal(format!("{program} returned non-UTF-8 output: {error}"))
    })
}

fn decode_base64(value: &str, field_name: &str) -> AppResult<Vec<u8>> {
    base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|error| {
            AppError::Internal(format!("failed to decode {field_name} as base64: {error}"))
        })
}

#[cfg(test)]
mod tests {
    use super::{DATA_KEY_LENGTH_BYTES, EnvelopeSecretEncryptor, KeyManagementService};
    use crate::aes_secret_encryptor::AesSecretEncryptor;
    use qryvanta_application::SecretEncryptor;
    use qryvanta_core::{AppError, AppResult};

    #[derive(Clone, Default)]
    struct FakeKeyManagementService;

    impl KeyManagementService for FakeKeyManagementService {
        fn wrap_key(&self, plaintext_key: &[u8; DATA_KEY_LENGTH_BYTES]) -> AppResult<Vec<u8>> {
            Ok(plaintext_key.iter().map(|value| value ^ 0xA5).collect())
        }

        fn unwrap_key(&self, wrapped_key: &[u8]) -> AppResult<[u8; DATA_KEY_LENGTH_BYTES]> {
            let decoded: Vec<u8> = wrapped_key.iter().map(|value| value ^ 0xA5).collect();
            decoded
                .try_into()
                .map_err(|_| AppError::Internal("wrapped fake KMS key must be 32 bytes".to_owned()))
        }
    }

    #[test]
    fn envelope_encrypt_decrypt_roundtrip() -> AppResult<()> {
        let encryptor = EnvelopeSecretEncryptor::new(FakeKeyManagementService, None);
        let plaintext = b"my-totp-secret";

        let encrypted = encryptor.encrypt(plaintext)?;
        let decrypted = encryptor.decrypt(&encrypted)?;

        assert_eq!(decrypted, plaintext);
        Ok(())
    }

    #[test]
    fn envelope_ciphertext_uses_versioned_magic_header() -> AppResult<()> {
        let encryptor = EnvelopeSecretEncryptor::new(FakeKeyManagementService, None);
        let encrypted = encryptor.encrypt(b"secret")?;

        assert!(encrypted.starts_with(b"QEK1"));
        Ok(())
    }

    #[test]
    fn decrypts_legacy_static_ciphertext_when_fallback_key_is_configured() -> AppResult<()> {
        let legacy_key = [7u8; 32];
        let legacy_encryptor = AesSecretEncryptor::new(&legacy_key);
        let encryptor =
            EnvelopeSecretEncryptor::new(FakeKeyManagementService, Some(legacy_encryptor.clone()));
        let ciphertext = legacy_encryptor.encrypt(b"legacy-secret")?;

        let decrypted = encryptor.decrypt(&ciphertext)?;

        assert_eq!(decrypted, b"legacy-secret");
        Ok(())
    }

    #[test]
    fn rejects_legacy_ciphertext_without_fallback_key() {
        let legacy_key = [7u8; 32];
        let legacy_encryptor = AesSecretEncryptor::new(&legacy_key);
        let encryptor = EnvelopeSecretEncryptor::new(FakeKeyManagementService, None);
        let ciphertext = legacy_encryptor.encrypt(b"legacy-secret");

        assert!(
            encryptor
                .decrypt(&ciphertext.unwrap_or_else(|_| unreachable!()))
                .is_err()
        );
    }

    #[test]
    fn rejects_truncated_envelope_ciphertext() {
        let encryptor = EnvelopeSecretEncryptor::new(FakeKeyManagementService, None);

        assert!(encryptor.decrypt(b"QEK1\x00\x00\x00").is_err());
    }
}
