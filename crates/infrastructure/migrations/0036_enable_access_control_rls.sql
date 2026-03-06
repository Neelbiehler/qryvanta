ALTER TABLE app_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE app_definitions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON app_definitions;
CREATE POLICY qryvanta_tenant_isolation ON app_definitions
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE app_entity_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE app_entity_bindings FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON app_entity_bindings;
CREATE POLICY qryvanta_tenant_isolation ON app_entity_bindings
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE app_role_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE app_role_bindings FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON app_role_bindings;
CREATE POLICY qryvanta_tenant_isolation ON app_role_bindings
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE app_role_entity_permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE app_role_entity_permissions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON app_role_entity_permissions;
CREATE POLICY qryvanta_tenant_isolation ON app_role_entity_permissions
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE app_sitemaps ENABLE ROW LEVEL SECURITY;
ALTER TABLE app_sitemaps FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON app_sitemaps;
CREATE POLICY qryvanta_tenant_isolation ON app_sitemaps
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE rbac_roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE rbac_roles FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON rbac_roles;
CREATE POLICY qryvanta_tenant_isolation ON rbac_roles
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE rbac_role_grants ENABLE ROW LEVEL SECURITY;
ALTER TABLE rbac_role_grants FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON rbac_role_grants;
CREATE POLICY qryvanta_tenant_isolation ON rbac_role_grants
    USING (
        EXISTS (
            SELECT 1
            FROM rbac_roles
            WHERE rbac_roles.id = rbac_role_grants.role_id
              AND rbac_roles.tenant_id = qryvanta_current_tenant_id()
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1
            FROM rbac_roles
            WHERE rbac_roles.id = rbac_role_grants.role_id
              AND rbac_roles.tenant_id = qryvanta_current_tenant_id()
        )
    );

ALTER TABLE rbac_subject_roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE rbac_subject_roles FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON rbac_subject_roles;
CREATE POLICY qryvanta_tenant_isolation ON rbac_subject_roles
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE runtime_subject_field_permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE runtime_subject_field_permissions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON runtime_subject_field_permissions;
CREATE POLICY qryvanta_tenant_isolation ON runtime_subject_field_permissions
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE security_temporary_access_grants ENABLE ROW LEVEL SECURITY;
ALTER TABLE security_temporary_access_grants FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON security_temporary_access_grants;
CREATE POLICY qryvanta_tenant_isolation ON security_temporary_access_grants
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE security_temporary_access_grant_permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE security_temporary_access_grant_permissions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON security_temporary_access_grant_permissions;
CREATE POLICY qryvanta_tenant_isolation ON security_temporary_access_grant_permissions
    USING (
        EXISTS (
            SELECT 1
            FROM security_temporary_access_grants
            WHERE security_temporary_access_grants.id = security_temporary_access_grant_permissions.grant_id
              AND security_temporary_access_grants.tenant_id = qryvanta_current_tenant_id()
        )
    )
    WITH CHECK (
        EXISTS (
            SELECT 1
            FROM security_temporary_access_grants
            WHERE security_temporary_access_grants.id = security_temporary_access_grant_permissions.grant_id
              AND security_temporary_access_grants.tenant_id = qryvanta_current_tenant_id()
        )
    );

ALTER TABLE extension_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE extension_definitions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON extension_definitions;
CREATE POLICY qryvanta_tenant_isolation ON extension_definitions
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE audit_log_entries ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_log_entries FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON audit_log_entries;
CREATE POLICY qryvanta_tenant_isolation ON audit_log_entries
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());
