/// Field-level runtime permission update item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldPermissionInput {
    /// Field logical name.
    pub field_logical_name: String,
    /// Read access marker.
    pub can_read: bool,
    /// Write access marker.
    pub can_write: bool,
}

/// Input payload for subject runtime field permission updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveRuntimeFieldPermissionsInput {
    /// Subject principal identifier.
    pub subject: String,
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Field permission entries to upsert.
    pub fields: Vec<RuntimeFieldPermissionInput>,
}

/// Runtime field permission projection returned to callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeFieldPermissionEntry {
    /// Subject principal identifier.
    pub subject: String,
    /// Entity logical name.
    pub entity_logical_name: String,
    /// Field logical name.
    pub field_logical_name: String,
    /// Read access marker.
    pub can_read: bool,
    /// Write access marker.
    pub can_write: bool,
    /// Last update timestamp in RFC3339.
    pub updated_at: String,
}
