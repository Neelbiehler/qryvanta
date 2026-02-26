use async_trait::async_trait;

use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::RegistrationMode;

use super::audit::{AuditLogEntry, AuditLogQuery};
use super::governance::AuditRetentionPolicy;
use super::roles::{CreateRoleInput, RoleAssignment, RoleDefinition};
use super::runtime_permissions::{RuntimeFieldPermissionEntry, SaveRuntimeFieldPermissionsInput};
use super::temporary_access::{
    CreateTemporaryAccessGrantInput, TemporaryAccessGrant, TemporaryAccessGrantQuery,
};

/// Repository port for role and assignment administration.
#[async_trait]
pub trait SecurityAdminRepository: Send + Sync {
    /// Lists all tenant roles with effective grants.
    async fn list_roles(&self, tenant_id: TenantId) -> AppResult<Vec<RoleDefinition>>;

    /// Creates a role and attaches grants.
    async fn create_role(
        &self,
        tenant_id: TenantId,
        input: CreateRoleInput,
    ) -> AppResult<RoleDefinition>;

    /// Assigns an existing role to a subject.
    async fn assign_role_to_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()>;

    /// Removes a role assignment from a subject.
    async fn remove_role_from_subject(
        &self,
        tenant_id: TenantId,
        subject: &str,
        role_name: &str,
    ) -> AppResult<()>;

    /// Lists current role assignments in tenant scope.
    async fn list_role_assignments(&self, tenant_id: TenantId) -> AppResult<Vec<RoleAssignment>>;

    /// Saves runtime field permissions for a subject and entity.
    async fn save_runtime_field_permissions(
        &self,
        tenant_id: TenantId,
        input: SaveRuntimeFieldPermissionsInput,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>>;

    /// Lists runtime field permissions in tenant scope.
    async fn list_runtime_field_permissions(
        &self,
        tenant_id: TenantId,
        subject: Option<&str>,
        entity_logical_name: Option<&str>,
    ) -> AppResult<Vec<RuntimeFieldPermissionEntry>>;

    /// Creates a temporary privileged access grant.
    async fn create_temporary_access_grant(
        &self,
        tenant_id: TenantId,
        created_by_subject: &str,
        input: CreateTemporaryAccessGrantInput,
    ) -> AppResult<TemporaryAccessGrant>;

    /// Revokes a temporary privileged access grant.
    async fn revoke_temporary_access_grant(
        &self,
        tenant_id: TenantId,
        revoked_by_subject: &str,
        grant_id: &str,
        revoke_reason: Option<&str>,
    ) -> AppResult<()>;

    /// Lists temporary privileged access grants.
    async fn list_temporary_access_grants(
        &self,
        tenant_id: TenantId,
        query: TemporaryAccessGrantQuery,
    ) -> AppResult<Vec<TemporaryAccessGrant>>;

    /// Returns the tenant registration mode.
    async fn registration_mode(&self, tenant_id: TenantId) -> AppResult<RegistrationMode>;

    /// Updates and returns tenant registration mode.
    async fn set_registration_mode(
        &self,
        tenant_id: TenantId,
        registration_mode: RegistrationMode,
    ) -> AppResult<RegistrationMode>;

    /// Returns tenant audit retention policy.
    async fn audit_retention_policy(&self, tenant_id: TenantId) -> AppResult<AuditRetentionPolicy>;

    /// Updates and returns tenant audit retention policy.
    async fn set_audit_retention_policy(
        &self,
        tenant_id: TenantId,
        retention_days: u16,
    ) -> AppResult<AuditRetentionPolicy>;
}

/// Repository port for reading tenant audit logs.
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    /// Lists most recent tenant audit entries.
    async fn list_recent_entries(
        &self,
        tenant_id: TenantId,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>>;

    /// Exports tenant audit entries for operational workflows.
    async fn export_entries(
        &self,
        tenant_id: TenantId,
        query: AuditLogQuery,
    ) -> AppResult<Vec<AuditLogEntry>>;

    /// Purges tenant audit entries older than the retention window.
    async fn purge_entries_older_than(
        &self,
        tenant_id: TenantId,
        retention_days: u16,
    ) -> AppResult<u64>;
}
