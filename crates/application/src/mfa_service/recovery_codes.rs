/// Generates 8 random recovery codes, each 8 alphanumeric characters.
pub(super) fn generate_recovery_codes() -> Vec<String> {
    const CODE_COUNT: usize = 8;
    const CODE_LENGTH: usize = 8;
    const ALPHABET: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";

    let mut codes = Vec::with_capacity(CODE_COUNT);

    for _ in 0..CODE_COUNT {
        let mut bytes = [0u8; CODE_LENGTH];
        getrandom::fill(&mut bytes).unwrap_or(());

        let code: String = bytes
            .iter()
            .map(|byte| {
                let index = (*byte as usize) % ALPHABET.len();
                ALPHABET[index] as char
            })
            .collect();

        codes.push(code);
    }

    codes
}

/// Hashes recovery codes for storage using SHA-256.
pub(super) fn hash_recovery_codes(codes: &[String]) -> serde_json::Value {
    let hashed: Vec<String> = codes.iter().map(|code| hash_single_code(code)).collect();
    serde_json::json!(hashed)
}

/// Hashes a single recovery code with SHA-256.
pub(super) fn hash_single_code(code: &str) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;

    let normalized = code.trim().to_lowercase();
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();

    result
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}
