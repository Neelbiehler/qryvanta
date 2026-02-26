/// Audit retention policy projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditRetentionPolicy {
    /// Retention window in days.
    pub retention_days: u16,
}

/// Audit purge operation result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditPurgeResult {
    /// Number of deleted entries.
    pub deleted_count: u64,
    /// Effective retention window in days.
    pub retention_days: u16,
}
