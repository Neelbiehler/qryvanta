//! Secret-loading helpers for startup configuration.

use std::env;
use std::fs;
use std::process::Command;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{AppError, AppResult};

const FILE_SUFFIX: &str = "_FILE";
const SECRET_REF_SUFFIX: &str = "_SECRET_REF";

/// A stable fingerprint record for cross-environment secret reuse checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretFingerprintRecord {
    /// Environment label that owns the fingerprint.
    pub environment: String,
    /// Canonical secret/config name being fingerprinted.
    pub secret_name: String,
    /// Stable SHA-256 fingerprint of the secret value.
    pub fingerprint: String,
}

impl SecretFingerprintRecord {
    /// Builds a fingerprint record for a named secret in one environment.
    #[must_use]
    pub fn from_secret(environment: &str, secret_name: &str, secret_value: &str) -> Self {
        Self {
            environment: environment.to_owned(),
            secret_name: secret_name.to_owned(),
            fingerprint: secret_fingerprint(secret_name, secret_value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SecretCommand {
    program: &'static str,
    args: Vec<String>,
}

/// Computes a stable SHA-256 fingerprint for a named secret.
#[must_use]
pub fn secret_fingerprint(secret_name: &str, secret_value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"qryvanta-secret-fingerprint:v1:");
    hasher.update(secret_name.as_bytes());
    hasher.update([0]);
    hasher.update(secret_value.as_bytes());
    hex_encode(hasher.finalize().as_slice())
}

/// Fails when any current secret fingerprint matches the same secret in a different environment.
pub fn detect_reused_secret_fingerprints(
    current_environment: &str,
    current_records: &[SecretFingerprintRecord],
    known_records: &[SecretFingerprintRecord],
) -> AppResult<()> {
    let collisions: Vec<String> = current_records
        .iter()
        .flat_map(|current_record| {
            known_records
                .iter()
                .filter(|known_record| {
                    known_record.environment != current_environment
                        && known_record.secret_name == current_record.secret_name
                        && known_record.fingerprint == current_record.fingerprint
                })
                .map(|known_record| {
                    format!(
                        "{} matches configured fingerprint for environment '{}'",
                        current_record.secret_name, known_record.environment
                    )
                })
        })
        .collect();

    if collisions.is_empty() {
        return Ok(());
    }

    Err(AppError::Validation(format!(
        "secret reuse guard detected cross-environment secret drift: {}",
        collisions.join("; ")
    )))
}

/// Loads a required secret from direct env, `*_FILE`, or `*_SECRET_REF`.
pub fn required_secret(name: &str) -> AppResult<String> {
    optional_secret(name)?.ok_or_else(|| AppError::Validation(format!("{name} is required")))
}

/// Loads a required secret and rejects empty or whitespace-only values.
pub fn required_non_empty_secret(name: &str) -> AppResult<String> {
    let value = required_secret(name)?;
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{name} must not be empty")));
    }

    Ok(value)
}

/// Loads an optional secret from direct env, `*_FILE`, or `*_SECRET_REF`.
pub fn optional_secret(name: &str) -> AppResult<Option<String>> {
    let direct_value = env::var(name).ok();
    let file_path = env::var(format!("{name}{FILE_SUFFIX}")).ok();
    let secret_reference = env::var(format!("{name}{SECRET_REF_SUFFIX}")).ok();

    resolve_optional_secret(name, direct_value, file_path, secret_reference)
}

fn resolve_optional_secret(
    name: &str,
    direct_value: Option<String>,
    file_path: Option<String>,
    secret_reference: Option<String>,
) -> AppResult<Option<String>> {
    let configured_sources = [
        direct_value.as_ref(),
        file_path.as_ref(),
        secret_reference.as_ref(),
    ];

    let configured_source_count = configured_sources
        .into_iter()
        .flatten()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .count();

    if configured_source_count > 1 {
        return Err(AppError::Validation(format!(
            "{name}, {name}{FILE_SUFFIX}, and {name}{SECRET_REF_SUFFIX} are mutually exclusive"
        )));
    }

    if let Some(value) = direct_value {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        return Ok(Some(value));
    }

    if let Some(path) = file_path {
        let trimmed_path = path.trim();
        if trimmed_path.is_empty() {
            return Ok(None);
        }

        let value = fs::read_to_string(trimmed_path).map_err(|error| {
            AppError::Validation(format!(
                "failed to read {name}{FILE_SUFFIX} path '{trimmed_path}': {error}"
            ))
        })?;
        return Ok(Some(strip_trailing_line_endings(value)));
    }

    if let Some(reference) = secret_reference {
        let trimmed_reference = reference.trim();
        if trimmed_reference.is_empty() {
            return Ok(None);
        }
        return resolve_secret_reference(trimmed_reference)
            .map(Some)
            .map_err(|error| match error {
                AppError::Validation(message) => {
                    AppError::Validation(format!("failed to resolve {name}: {message}"))
                }
                other => other,
            });
    }

    Ok(None)
}

/// Validates a standalone secret reference format without resolving it.
pub fn validate_secret_reference(reference: &str) -> AppResult<()> {
    let trimmed_reference = reference.trim();
    if trimmed_reference.is_empty() {
        return Err(AppError::Validation(
            "secret reference must not be empty".to_owned(),
        ));
    }

    parse_secret_reference(trimmed_reference).map(|_| ())
}

/// Resolves one standalone secret reference through the configured CLI integration.
pub fn resolve_secret_reference(reference: &str) -> AppResult<String> {
    let trimmed_reference = reference.trim();
    if trimmed_reference.is_empty() {
        return Err(AppError::Validation(
            "secret reference must not be empty".to_owned(),
        ));
    }

    let command = parse_secret_reference(trimmed_reference)?;
    let output = Command::new(command.program)
        .args(command.args.iter().map(String::as_str))
        .output()
        .map_err(|error| {
            AppError::Validation(format!(
                "failed to execute secret resolver '{}': {error}",
                command.program
            ))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        return Err(AppError::Validation(format!(
            "secret resolver exited with status {}{}",
            output.status,
            if stderr.is_empty() {
                String::new()
            } else {
                format!(": {stderr}")
            }
        )));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|error| {
        AppError::Validation(format!(
            "secret resolver returned non-UTF-8 output: {error}"
        ))
    })?;

    Ok(strip_trailing_line_endings(stdout))
}

fn strip_trailing_line_endings(mut value: String) -> String {
    while value.ends_with('\n') || value.ends_with('\r') {
        value.pop();
    }

    value
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push_str(format!("{byte:02x}").as_str());
    }
    encoded
}

fn parse_secret_reference(reference: &str) -> AppResult<SecretCommand> {
    if reference.starts_with("op://") {
        return Ok(SecretCommand {
            program: "op",
            args: vec!["read".to_owned(), reference.to_owned()],
        });
    }

    if let Some(secret_id) = reference.strip_prefix("aws-sm://") {
        if secret_id.trim().is_empty() {
            return Err(AppError::Validation(
                "aws-sm:// secret references must include a secret id".to_owned(),
            ));
        }

        return Ok(SecretCommand {
            program: "aws",
            args: vec![
                "secretsmanager".to_owned(),
                "get-secret-value".to_owned(),
                "--secret-id".to_owned(),
                secret_id.to_owned(),
                "--query".to_owned(),
                "SecretString".to_owned(),
                "--output".to_owned(),
                "text".to_owned(),
            ],
        });
    }

    if let Some(parameter_name) = reference.strip_prefix("aws-ssm://") {
        if parameter_name.trim().is_empty() {
            return Err(AppError::Validation(
                "aws-ssm:// secret references must include a parameter name".to_owned(),
            ));
        }

        return Ok(SecretCommand {
            program: "aws",
            args: vec![
                "ssm".to_owned(),
                "get-parameter".to_owned(),
                "--name".to_owned(),
                parameter_name.to_owned(),
                "--with-decryption".to_owned(),
                "--query".to_owned(),
                "Parameter.Value".to_owned(),
                "--output".to_owned(),
                "text".to_owned(),
            ],
        });
    }

    if let Some(vault_reference) = reference.strip_prefix("vault://") {
        let (path, field) = vault_reference.rsplit_once('#').ok_or_else(|| {
            AppError::Validation(
                "vault:// secret references must include a '#field' suffix".to_owned(),
            )
        })?;

        if path.trim().is_empty() || field.trim().is_empty() {
            return Err(AppError::Validation(
                "vault:// secret references must include both a path and field".to_owned(),
            ));
        }

        return Ok(SecretCommand {
            program: "vault",
            args: vec![
                "kv".to_owned(),
                "get".to_owned(),
                format!("-field={field}"),
                path.to_owned(),
            ],
        });
    }

    if let Some(gcp_reference) = reference.strip_prefix("gcp-sm://") {
        let segments: Vec<&str> = gcp_reference.split('/').collect();
        if segments.len() != 6
            || segments[0] != "projects"
            || segments[2] != "secrets"
            || segments[4] != "versions"
            || segments[1].trim().is_empty()
            || segments[3].trim().is_empty()
            || segments[5].trim().is_empty()
        {
            return Err(AppError::Validation(
                "gcp-sm:// secret references must use 'gcp-sm://projects/<project>/secrets/<secret>/versions/<version>'"
                    .to_owned(),
            ));
        }

        return Ok(SecretCommand {
            program: "gcloud",
            args: vec![
                "secrets".to_owned(),
                "versions".to_owned(),
                "access".to_owned(),
                segments[5].to_owned(),
                format!("--secret={}", segments[3]),
                format!("--project={}", segments[1]),
            ],
        });
    }

    Err(AppError::Validation(format!(
        "unsupported secret reference '{reference}': supported schemes are op://, aws-sm://, aws-ssm://, vault://, and gcp-sm://"
    )))
}

#[cfg(test)]
mod tests {
    use super::{
        SecretFingerprintRecord, detect_reused_secret_fingerprints, parse_secret_reference,
        resolve_optional_secret, secret_fingerprint, strip_trailing_line_endings,
        validate_secret_reference,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| unreachable!())
            .as_nanos();
        std::env::temp_dir().join(format!("qryvanta-{name}-{nanos}.tmp"))
    }

    #[test]
    fn strips_trailing_newlines_without_trimming_other_bytes() {
        assert_eq!(
            strip_trailing_line_endings(" secret-value \n\r".to_owned()),
            " secret-value "
        );
    }

    #[test]
    fn parses_one_password_secret_references() {
        let command =
            parse_secret_reference("op://vault/item/password").unwrap_or_else(|_| unreachable!());

        assert_eq!(command.program, "op");
        assert_eq!(command.args, vec!["read", "op://vault/item/password"]);
    }

    #[test]
    fn parses_aws_secrets_manager_references() {
        let command = parse_secret_reference("aws-sm://prod/qryvanta/session")
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(command.program, "aws");
        assert_eq!(
            command.args,
            vec![
                "secretsmanager",
                "get-secret-value",
                "--secret-id",
                "prod/qryvanta/session",
                "--query",
                "SecretString",
                "--output",
                "text",
            ]
        );
    }

    #[test]
    fn parses_vault_references() {
        let command = parse_secret_reference("vault://kv/qryvanta/prod#session_secret")
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(command.program, "vault");
        assert_eq!(
            command.args,
            vec!["kv", "get", "-field=session_secret", "kv/qryvanta/prod"]
        );
    }

    #[test]
    fn parses_gcp_secret_manager_references() {
        let command = parse_secret_reference(
            "gcp-sm://projects/prod-project/secrets/session-secret/versions/latest",
        )
        .unwrap_or_else(|_| unreachable!());

        assert_eq!(command.program, "gcloud");
        assert_eq!(
            command.args,
            vec![
                "secrets",
                "versions",
                "access",
                "latest",
                "--secret=session-secret",
                "--project=prod-project",
            ]
        );
    }

    #[test]
    fn parses_aws_ssm_references() {
        let command = parse_secret_reference("aws-ssm:///prod/qryvanta/session")
            .unwrap_or_else(|_| unreachable!());

        assert_eq!(command.program, "aws");
        assert_eq!(
            command.args,
            vec![
                "ssm",
                "get-parameter",
                "--name",
                "/prod/qryvanta/session",
                "--with-decryption",
                "--query",
                "Parameter.Value",
                "--output",
                "text",
            ]
        );
    }

    #[test]
    fn resolves_file_backed_secrets() {
        let path = unique_temp_path("secret");
        fs::write(&path, "resolved-from-file\n").unwrap_or_else(|_| unreachable!());

        let result = resolve_optional_secret(
            "SESSION_SECRET",
            None,
            Some(path.to_string_lossy().into_owned()),
            None,
        )
        .unwrap_or_else(|_| unreachable!());

        fs::remove_file(&path).unwrap_or_else(|_| unreachable!());

        assert_eq!(result.as_deref(), Some("resolved-from-file"));
    }

    #[test]
    fn rejects_multiple_secret_sources_for_one_setting() {
        let result = resolve_optional_secret(
            "SESSION_SECRET",
            Some("direct".to_owned()),
            Some("/tmp/session_secret".to_owned()),
            None,
        );

        assert!(result.is_err());
    }

    #[test]
    fn rejects_vault_references_without_field_suffix() {
        let result = parse_secret_reference("vault://kv/qryvanta/prod");
        assert!(result.is_err());
    }

    #[test]
    fn computes_stable_secret_fingerprints() {
        let left = secret_fingerprint("SESSION_SECRET", "same-secret");
        let right = secret_fingerprint("SESSION_SECRET", "same-secret");

        assert_eq!(left, right);
    }

    #[test]
    fn detects_cross_environment_secret_reuse() {
        let current = [SecretFingerprintRecord::from_secret(
            "production",
            "SESSION_SECRET",
            "shared-secret",
        )];
        let known = [SecretFingerprintRecord::from_secret(
            "staging",
            "SESSION_SECRET",
            "shared-secret",
        )];

        assert!(detect_reused_secret_fingerprints("production", &current, &known).is_err());
    }

    #[test]
    fn ignores_same_environment_secret_fingerprints() {
        let current = [SecretFingerprintRecord::from_secret(
            "production",
            "SESSION_SECRET",
            "shared-secret",
        )];
        let known = [SecretFingerprintRecord::from_secret(
            "production",
            "SESSION_SECRET",
            "shared-secret",
        )];

        assert!(detect_reused_secret_fingerprints("production", &current, &known).is_ok());
    }

    #[test]
    fn rejects_unsupported_secret_reference_schemes() {
        let result = parse_secret_reference("azure-kv://vault/secret");
        assert!(result.is_err());
    }

    #[test]
    fn validates_supported_secret_references() {
        let result = validate_secret_reference("op://vault/item/password");
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_blank_secret_references() {
        let result = validate_secret_reference("   ");
        assert!(result.is_err());
    }
}
