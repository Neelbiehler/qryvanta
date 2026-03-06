//! Helpers for tamper-evident audit log chaining.

use sha2::{Digest, Sha256};

use qryvanta_core::TenantId;

/// Stable chain payload used for audit-log tamper evidence.
pub(crate) struct AuditChainInput<'a> {
    pub(crate) tenant_id: TenantId,
    pub(crate) chain_position: i64,
    pub(crate) previous_entry_hash: Option<&'a str>,
    pub(crate) subject: &'a str,
    pub(crate) action: &'a str,
    pub(crate) resource_type: &'a str,
    pub(crate) resource_id: &'a str,
    pub(crate) detail: Option<&'a str>,
    pub(crate) created_at_utc: &'a str,
}

pub(crate) fn compute_audit_entry_hash(input: &AuditChainInput<'_>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"qryvanta-audit-chain-v1:");
    hasher.update(input.tenant_id.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(input.chain_position.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(input.previous_entry_hash.unwrap_or_default().as_bytes());
    hasher.update(b":");
    hasher.update(input.subject.as_bytes());
    hasher.update(b":");
    hasher.update(input.action.as_bytes());
    hasher.update(b":");
    hasher.update(input.resource_type.as_bytes());
    hasher.update(b":");
    hasher.update(input.resource_id.as_bytes());
    hasher.update(b":");
    hasher.update(input.detail.unwrap_or_default().as_bytes());
    hasher.update(b":");
    hasher.update(input.created_at_utc.as_bytes());
    hex_encode(hasher.finalize().as_slice())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push_str(format!("{byte:02x}").as_str());
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::{AuditChainInput, compute_audit_entry_hash};
    use qryvanta_core::TenantId;

    #[test]
    fn audit_chain_hash_is_stable_for_same_payload() {
        let tenant_id = TenantId::new();
        let left = compute_audit_entry_hash(&AuditChainInput {
            tenant_id,
            chain_position: 1,
            previous_entry_hash: None,
            subject: "alice",
            action: "runtime.record.created",
            resource_type: "runtime_record",
            resource_id: "record-1",
            detail: Some("{\"name\":\"Example\"}"),
            created_at_utc: "2026-03-06T12:00:00.000000Z",
        });
        let right = compute_audit_entry_hash(&AuditChainInput {
            tenant_id,
            chain_position: 1,
            previous_entry_hash: None,
            subject: "alice",
            action: "runtime.record.created",
            resource_type: "runtime_record",
            resource_id: "record-1",
            detail: Some("{\"name\":\"Example\"}"),
            created_at_utc: "2026-03-06T12:00:00.000000Z",
        });

        assert_eq!(left, right);
    }
}
