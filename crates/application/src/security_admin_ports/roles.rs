use qryvanta_domain::Permission;

/// Role definition returned to callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleDefinition {
    /// Stable role identifier.
    pub role_id: String,
    /// Unique role name in tenant scope.
    pub name: String,
    /// Indicates a system-managed role.
    pub is_system: bool,
    /// Effective role grants.
    pub permissions: Vec<Permission>,
}

/// Assignment projection mapping a subject to a role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleAssignment {
    /// Subject identifier.
    pub subject: String,
    /// Role identifier.
    pub role_id: String,
    /// Role name.
    pub role_name: String,
    /// Assignment timestamp in RFC3339.
    pub assigned_at: String,
}

/// Input payload for creating custom roles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRoleInput {
    /// Unique role name in tenant scope.
    pub name: String,
    /// Grants to attach to the role.
    pub permissions: Vec<Permission>,
}
