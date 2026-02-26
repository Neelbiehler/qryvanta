use qryvanta_domain::RegistrationMode;

use super::types::{
    AuditLogEntryResponse, AuditPurgeResultResponse, AuditRetentionPolicyResponse,
    RoleAssignmentResponse, RoleResponse, RuntimeFieldPermissionResponse,
    TemporaryAccessGrantResponse, TenantRegistrationModeResponse,
};

impl From<qryvanta_application::RoleDefinition> for RoleResponse {
    fn from(value: qryvanta_application::RoleDefinition) -> Self {
        Self {
            role_id: value.role_id,
            name: value.name,
            is_system: value.is_system,
            permissions: value
                .permissions
                .into_iter()
                .map(|permission| permission.as_str().to_owned())
                .collect(),
        }
    }
}

impl From<qryvanta_application::AuditLogEntry> for AuditLogEntryResponse {
    fn from(value: qryvanta_application::AuditLogEntry) -> Self {
        Self {
            event_id: value.event_id,
            subject: value.subject,
            action: value.action,
            resource_type: value.resource_type,
            resource_id: value.resource_id,
            detail: value.detail,
            created_at: value.created_at,
        }
    }
}

impl From<qryvanta_application::RoleAssignment> for RoleAssignmentResponse {
    fn from(value: qryvanta_application::RoleAssignment) -> Self {
        Self {
            subject: value.subject,
            role_id: value.role_id,
            role_name: value.role_name,
            assigned_at: value.assigned_at,
        }
    }
}

impl From<RegistrationMode> for TenantRegistrationModeResponse {
    fn from(value: RegistrationMode) -> Self {
        Self {
            registration_mode: value.as_str().to_owned(),
        }
    }
}

impl From<qryvanta_application::RuntimeFieldPermissionEntry> for RuntimeFieldPermissionResponse {
    fn from(value: qryvanta_application::RuntimeFieldPermissionEntry) -> Self {
        Self {
            subject: value.subject,
            entity_logical_name: value.entity_logical_name,
            field_logical_name: value.field_logical_name,
            can_read: value.can_read,
            can_write: value.can_write,
            updated_at: value.updated_at,
        }
    }
}

impl From<qryvanta_application::TemporaryAccessGrant> for TemporaryAccessGrantResponse {
    fn from(value: qryvanta_application::TemporaryAccessGrant) -> Self {
        Self {
            grant_id: value.grant_id,
            subject: value.subject,
            permissions: value
                .permissions
                .into_iter()
                .map(|permission| permission.as_str().to_owned())
                .collect(),
            reason: value.reason,
            created_by_subject: value.created_by_subject,
            expires_at: value.expires_at,
            revoked_at: value.revoked_at,
        }
    }
}

impl From<qryvanta_application::AuditRetentionPolicy> for AuditRetentionPolicyResponse {
    fn from(value: qryvanta_application::AuditRetentionPolicy) -> Self {
        Self {
            retention_days: value.retention_days,
        }
    }
}

impl From<qryvanta_application::AuditPurgeResult> for AuditPurgeResultResponse {
    fn from(value: qryvanta_application::AuditPurgeResult) -> Self {
        Self {
            deleted_count: value.deleted_count,
            retention_days: value.retention_days,
        }
    }
}
