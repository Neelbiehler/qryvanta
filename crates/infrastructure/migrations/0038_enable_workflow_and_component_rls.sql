CREATE OR REPLACE FUNCTION qryvanta_rls_scope(scope_name TEXT)
RETURNS BOOLEAN
LANGUAGE sql
STABLE
AS $$
    SELECT current_setting('qryvanta.rls_scope', true) = scope_name
$$;

ALTER TABLE entity_option_sets ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_option_sets FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON entity_option_sets;
CREATE POLICY qryvanta_tenant_isolation ON entity_option_sets
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE entity_forms ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_forms FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON entity_forms;
CREATE POLICY qryvanta_tenant_isolation ON entity_forms
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE entity_views ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_views FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON entity_views;
CREATE POLICY qryvanta_tenant_isolation ON entity_views
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE entity_business_rules ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_business_rules FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON entity_business_rules;
CREATE POLICY qryvanta_tenant_isolation ON entity_business_rules
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE workflow_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_definitions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON workflow_definitions;
CREATE POLICY qryvanta_tenant_isolation ON workflow_definitions
    USING (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    )
    WITH CHECK (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    );

ALTER TABLE workflow_execution_runs ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_execution_runs FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON workflow_execution_runs;
CREATE POLICY qryvanta_tenant_isolation ON workflow_execution_runs
    USING (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    )
    WITH CHECK (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    );

ALTER TABLE workflow_execution_attempts ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_execution_attempts FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON workflow_execution_attempts;
CREATE POLICY qryvanta_tenant_isolation ON workflow_execution_attempts
    USING (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    )
    WITH CHECK (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    );

ALTER TABLE workflow_execution_jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_execution_jobs FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON workflow_execution_jobs;
CREATE POLICY qryvanta_tenant_isolation ON workflow_execution_jobs
    USING (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    )
    WITH CHECK (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    );

ALTER TABLE qrywell_sync_jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE qrywell_sync_jobs FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON qrywell_sync_jobs;
CREATE POLICY qryvanta_tenant_isolation ON qrywell_sync_jobs
    USING (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('qrywell_sync')
    )
    WITH CHECK (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('qrywell_sync')
    );
