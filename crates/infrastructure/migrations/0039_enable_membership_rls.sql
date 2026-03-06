CREATE OR REPLACE FUNCTION qryvanta_rls_subject()
RETURNS TEXT
LANGUAGE sql
STABLE
AS $$
    SELECT NULLIF(current_setting('qryvanta.rls_subject', true), '')
$$;

ALTER TABLE tenant_memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_memberships FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_or_subject_lookup ON tenant_memberships;
CREATE POLICY qryvanta_tenant_or_subject_lookup ON tenant_memberships
    USING (
        tenant_id = qryvanta_current_tenant_id()
        OR (
            qryvanta_rls_scope('membership_subject_lookup')
            AND subject = qryvanta_rls_subject()
        )
    )
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());
