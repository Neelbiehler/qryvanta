/// Audit log entry projection for administrative views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLogEntry {
    /// Stable event identifier.
    pub event_id: String,
    /// Actor subject.
    pub subject: String,
    /// Stable action identifier.
    pub action: String,
    /// Event resource type.
    pub resource_type: String,
    /// Event resource identifier.
    pub resource_id: String,
    /// Optional event detail.
    pub detail: Option<String>,
    /// Event timestamp in RFC3339.
    pub created_at: String,
    /// Monotonic chain position within the tenant audit stream.
    pub chain_position: i64,
    /// Previous entry hash in the tenant chain, if any.
    pub previous_entry_hash: Option<String>,
    /// Current entry hash anchoring this audit row.
    pub entry_hash: String,
}

/// Summary of tenant audit chain verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditIntegrityStatus {
    /// Whether the chain verified successfully.
    pub is_valid: bool,
    /// Number of entries scanned during verification.
    pub verified_entries: usize,
    /// Latest verified chain position, if any entries exist.
    pub latest_chain_position: Option<i64>,
    /// Latest verified entry hash, if any entries exist.
    pub latest_entry_hash: Option<String>,
    /// Human-readable verification failures.
    pub failures: Vec<String>,
}

/// Query parameters for audit log listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditLogQuery {
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for offset pagination.
    pub offset: usize,
    /// Optional action filter.
    pub action: Option<String>,
    /// Optional subject filter.
    pub subject: Option<String>,
}

/// Summary payload for one workspace publish run audit event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspacePublishRunAuditInput {
    /// Number of entities selected for the run.
    pub requested_entities: usize,
    /// Number of apps selected for the run.
    pub requested_apps: usize,
    /// Entity logical names selected for this run.
    pub requested_entity_logical_names: Vec<String>,
    /// App logical names selected for this run.
    pub requested_app_logical_names: Vec<String>,
    /// Entity logical names published in this run.
    pub published_entities: Vec<String>,
    /// App logical names validated in this run.
    pub validated_apps: Vec<String>,
    /// Number of blocking issues discovered.
    pub issue_count: usize,
    /// Whether the run completed as publishable.
    pub is_publishable: bool,
}
