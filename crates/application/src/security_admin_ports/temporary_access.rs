use qryvanta_domain::Permission;

/// Input payload for temporary access grants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTemporaryAccessGrantInput {
    /// Subject principal identifier.
    pub subject: String,
    /// Granted permissions.
    pub permissions: Vec<Permission>,
    /// Justification for temporary access.
    pub reason: String,
    /// Grant duration in minutes.
    pub duration_minutes: u32,
}

/// Temporary access grant projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporaryAccessGrant {
    /// Stable grant id.
    pub grant_id: String,
    /// Subject principal identifier.
    pub subject: String,
    /// Granted permissions.
    pub permissions: Vec<Permission>,
    /// Justification for temporary access.
    pub reason: String,
    /// Grant creator subject.
    pub created_by_subject: String,
    /// Expiration timestamp in RFC3339.
    pub expires_at: String,
    /// Revocation timestamp in RFC3339, when present.
    pub revoked_at: Option<String>,
}

/// Query parameters for temporary access grant listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporaryAccessGrantQuery {
    /// Optional subject filter.
    pub subject: Option<String>,
    /// Whether to return only active (non-revoked, non-expired) grants.
    pub active_only: bool,
    /// Maximum rows returned.
    pub limit: usize,
    /// Number of rows skipped for pagination.
    pub offset: usize,
}
