use std::str::FromStr;

use qryvanta_core::AppError;
use serde::{Deserialize, Serialize};

/// Product surfaces that partition the Qryvanta UX.
///
/// Each surface targets a distinct persona:
/// - **Admin**: tenant administrators managing roles, audit, and security.
/// - **Maker**: low-code builders defining entities, fields, and app configuration.
/// - **Worker**: operational end-users interacting with published apps and records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    /// Tenant administration: roles, audit log, security settings.
    Admin,
    /// Low-code configuration: entities, fields, schema publishing, app studio.
    Maker,
    /// Operational end-user apps: published app navigation, runtime records.
    Worker,
}

impl Surface {
    /// Returns a stable transport value for this surface.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Maker => "maker",
            Self::Worker => "worker",
        }
    }

    /// Returns all known surfaces.
    #[must_use]
    pub fn all() -> &'static [Self] {
        &[Self::Admin, Self::Maker, Self::Worker]
    }

    /// Returns the permissions that grant access to this surface.
    ///
    /// A subject may enter a surface if they hold **any** of the returned
    /// permissions (logical OR).
    #[must_use]
    pub fn required_permissions(&self) -> &'static [Permission] {
        match self {
            Self::Admin => &[
                Permission::SecurityRoleManage,
                Permission::SecurityAuditRead,
                Permission::SecurityInviteSend,
            ],
            Self::Maker => &[
                Permission::MetadataEntityRead,
                Permission::MetadataEntityCreate,
                Permission::MetadataFieldRead,
                Permission::MetadataFieldWrite,
            ],
            Self::Worker => &[
                Permission::RuntimeRecordRead,
                Permission::RuntimeRecordReadOwn,
                Permission::RuntimeRecordWrite,
                Permission::RuntimeRecordWriteOwn,
            ],
        }
    }
}

impl FromStr for Surface {
    type Err = AppError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "admin" => Ok(Self::Admin),
            "maker" => Ok(Self::Maker),
            "worker" => Ok(Self::Worker),
            _ => Err(AppError::Validation(format!(
                "unknown surface value '{value}'"
            ))),
        }
    }
}

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
    /// Allows reading only runtime records owned by the subject.
    RuntimeRecordReadOwn,
    /// Allows mutating runtime records.
    RuntimeRecordWrite,
    /// Allows mutating only runtime records owned by the subject.
    RuntimeRecordWriteOwn,
    /// Allows reading audit log entries.
    SecurityAuditRead,
    /// Allows managing roles and grants.
    SecurityRoleManage,
    /// Allows sending tenant invite emails.
    SecurityInviteSend,
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
            Self::RuntimeRecordReadOwn => "runtime.record.read.own",
            Self::RuntimeRecordWrite => "runtime.record.write",
            Self::RuntimeRecordWriteOwn => "runtime.record.write.own",
            Self::SecurityAuditRead => "security.audit.read",
            Self::SecurityRoleManage => "security.role.manage",
            Self::SecurityInviteSend => "security.invite.send",
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
            Permission::RuntimeRecordReadOwn,
            Permission::RuntimeRecordWrite,
            Permission::RuntimeRecordWriteOwn,
            Permission::SecurityAuditRead,
            Permission::SecurityRoleManage,
            Permission::SecurityInviteSend,
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
            "runtime.record.read.own" => Ok(Self::RuntimeRecordReadOwn),
            "runtime.record.write" => Ok(Self::RuntimeRecordWrite),
            "runtime.record.write.own" => Ok(Self::RuntimeRecordWriteOwn),
            "security.audit.read" => Ok(Self::SecurityAuditRead),
            "security.role.manage" => Ok(Self::SecurityRoleManage),
            "security.invite.send" => Ok(Self::SecurityInviteSend),
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
    /// Emitted when an app definition is created.
    AppCreated,
    /// Emitted when an entity is bound into an app navigation.
    AppEntityBound,
    /// Emitted when role permissions are updated for an app entity.
    AppRoleEntityPermissionSaved,
    /// Emitted when a workflow definition is created or updated.
    WorkflowSaved,
    /// Emitted when a workflow run reaches a terminal state.
    WorkflowRunCompleted,
    /// Emitted when an entity definition is created.
    MetadataEntityCreated,
    /// Emitted when a metadata field is created or updated.
    MetadataFieldSaved,
    /// Emitted when draft metadata is published.
    MetadataEntityPublished,
    /// Emitted when a workspace publish run completes.
    MetadataWorkspacePublished,
    /// Emitted when a runtime record is created.
    RuntimeRecordCreated,
    /// Emitted when a runtime record is updated.
    RuntimeRecordUpdated,
    /// Emitted when a runtime record is deleted.
    RuntimeRecordDeleted,
    /// Emitted when a custom role is created.
    SecurityRoleCreated,
    /// Emitted when a role is assigned to a subject.
    SecurityRoleAssigned,
    /// Emitted when a role is removed from a subject.
    SecurityRoleUnassigned,
    /// Emitted when runtime field permissions are updated for a subject.
    SecurityRuntimeFieldPermissionsSaved,
    /// Emitted when temporary privileged access is granted.
    SecurityTemporaryAccessGranted,
    /// Emitted when temporary privileged access is revoked.
    SecurityTemporaryAccessRevoked,
    /// Emitted when temporary privileged access is used for authorization.
    SecurityTemporaryAccessUsed,
    /// Emitted when tenant registration mode is updated.
    SecurityTenantRegistrationModeUpdated,
    /// Emitted when audit retention policy is updated.
    SecurityAuditRetentionUpdated,
    /// Emitted when audit entries are purged by retention policy.
    SecurityAuditEntriesPurged,
}

impl AuditAction {
    /// Returns a stable storage value for this action.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AppCreated => "app.created",
            Self::AppEntityBound => "app.entity.bound",
            Self::AppRoleEntityPermissionSaved => "app.role_entity_permission.saved",
            Self::WorkflowSaved => "workflow.saved",
            Self::WorkflowRunCompleted => "workflow.run.completed",
            Self::MetadataEntityCreated => "metadata.entity.created",
            Self::MetadataFieldSaved => "metadata.field.saved",
            Self::MetadataEntityPublished => "metadata.entity.published",
            Self::MetadataWorkspacePublished => "metadata.workspace.published",
            Self::RuntimeRecordCreated => "runtime.record.created",
            Self::RuntimeRecordUpdated => "runtime.record.updated",
            Self::RuntimeRecordDeleted => "runtime.record.deleted",
            Self::SecurityRoleCreated => "security.role.created",
            Self::SecurityRoleAssigned => "security.role.assigned",
            Self::SecurityRoleUnassigned => "security.role.unassigned",
            Self::SecurityRuntimeFieldPermissionsSaved => {
                "security.runtime.field_permissions.saved"
            }
            Self::SecurityTemporaryAccessGranted => "security.temporary_access.granted",
            Self::SecurityTemporaryAccessRevoked => "security.temporary_access.revoked",
            Self::SecurityTemporaryAccessUsed => "security.temporary_access.used",
            Self::SecurityTenantRegistrationModeUpdated => {
                "security.tenant.registration_mode.updated"
            }
            Self::SecurityAuditRetentionUpdated => "security.audit.retention.updated",
            Self::SecurityAuditEntriesPurged => "security.audit.entries.purged",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::{Permission, Surface};

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

    #[test]
    fn surface_roundtrip_storage_value() {
        for surface in Surface::all() {
            let restored = Surface::from_str(surface.as_str());
            assert!(restored.is_ok());
            assert_eq!(restored.unwrap_or(Surface::Worker), *surface);
        }
    }

    #[test]
    fn unknown_surface_is_rejected() {
        let parsed = Surface::from_str("unknown_surface");
        assert!(parsed.is_err());
    }

    #[test]
    fn every_surface_has_at_least_one_permission() {
        for surface in Surface::all() {
            assert!(
                !surface.required_permissions().is_empty(),
                "surface {:?} must have at least one required permission",
                surface
            );
        }
    }
}
