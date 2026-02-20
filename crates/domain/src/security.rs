use std::str::FromStr;

use qryvanta_core::AppError;
use serde::{Deserialize, Serialize};

/// Permissions enforced by application policy checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Allows reading metadata entity definitions.
    MetadataEntityRead,
    /// Allows creating metadata entity definitions.
    MetadataEntityCreate,
    /// Allows reading metadata field definitions.
    MetadataFieldRead,
    /// Allows updating metadata field definitions.
    MetadataFieldWrite,
    /// Allows reading runtime records.
    RuntimeRecordRead,
    /// Allows mutating runtime records.
    RuntimeRecordWrite,
    /// Allows reading audit log entries.
    SecurityAuditRead,
    /// Allows managing roles and grants.
    SecurityRoleManage,
}

impl Permission {
    /// Returns a stable storage value for this permission.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MetadataEntityRead => "metadata.entity.read",
            Self::MetadataEntityCreate => "metadata.entity.create",
            Self::MetadataFieldRead => "metadata.field.read",
            Self::MetadataFieldWrite => "metadata.field.write",
            Self::RuntimeRecordRead => "runtime.record.read",
            Self::RuntimeRecordWrite => "runtime.record.write",
            Self::SecurityAuditRead => "security.audit.read",
            Self::SecurityRoleManage => "security.role.manage",
        }
    }

    /// Returns all known permissions.
    #[must_use]
    pub fn all() -> &'static [Self] {
        const ALL: &[Permission] = &[
            Permission::MetadataEntityRead,
            Permission::MetadataEntityCreate,
            Permission::MetadataFieldRead,
            Permission::MetadataFieldWrite,
            Permission::RuntimeRecordRead,
            Permission::RuntimeRecordWrite,
            Permission::SecurityAuditRead,
            Permission::SecurityRoleManage,
        ];

        ALL
    }

    /// Parses a transport value into a permission.
    pub fn from_transport(value: &str) -> Result<Self, AppError> {
        Self::from_str(value)
    }
}

impl FromStr for Permission {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "metadata.entity.read" => Ok(Self::MetadataEntityRead),
            "metadata.entity.create" => Ok(Self::MetadataEntityCreate),
            "metadata.field.read" => Ok(Self::MetadataFieldRead),
            "metadata.field.write" => Ok(Self::MetadataFieldWrite),
            "runtime.record.read" => Ok(Self::RuntimeRecordRead),
            "runtime.record.write" => Ok(Self::RuntimeRecordWrite),
            "security.audit.read" => Ok(Self::SecurityAuditRead),
            "security.role.manage" => Ok(Self::SecurityRoleManage),
            _ => Err(AppError::Validation(format!(
                "unknown permission value '{value}'"
            ))),
        }
    }
}

/// Stable audit actions emitted by application use-cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    /// Emitted when an entity definition is created.
    MetadataEntityCreated,
    /// Emitted when a custom role is created.
    SecurityRoleCreated,
    /// Emitted when a role is assigned to a subject.
    SecurityRoleAssigned,
    /// Emitted when a role is removed from a subject.
    SecurityRoleUnassigned,
}

impl AuditAction {
    /// Returns a stable storage value for this action.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MetadataEntityCreated => "metadata.entity.created",
            Self::SecurityRoleCreated => "security.role.created",
            Self::SecurityRoleAssigned => "security.role.assigned",
            Self::SecurityRoleUnassigned => "security.role.unassigned",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Permission;

    #[test]
    fn permission_roundtrip_storage_value() {
        let permission = Permission::MetadataEntityCreate;
        let restored = Permission::from_str(permission.as_str());
        assert!(restored.is_ok());
        assert_eq!(
            restored.unwrap_or(Permission::MetadataEntityRead),
            permission
        );
    }

    #[test]
    fn unknown_permission_is_rejected() {
        let parsed = Permission::from_str("metadata.entity.unknown");
        assert!(parsed.is_err());
    }
}
