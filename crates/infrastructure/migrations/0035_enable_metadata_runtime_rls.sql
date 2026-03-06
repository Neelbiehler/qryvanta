CREATE OR REPLACE FUNCTION qryvanta_current_tenant_id()
RETURNS UUID
LANGUAGE SQL
STABLE
AS $$
    SELECT NULLIF(current_setting('qryvanta.current_tenant_id', true), '')::UUID
$$;

ALTER TABLE entity_definitions ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_definitions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON entity_definitions;
CREATE POLICY qryvanta_tenant_isolation ON entity_definitions
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE entity_fields ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_fields FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON entity_fields;
CREATE POLICY qryvanta_tenant_isolation ON entity_fields
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE entity_published_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_published_versions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON entity_published_versions;
CREATE POLICY qryvanta_tenant_isolation ON entity_published_versions
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE entity_form_published_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_form_published_versions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON entity_form_published_versions;
CREATE POLICY qryvanta_tenant_isolation ON entity_form_published_versions
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE entity_view_published_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE entity_view_published_versions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON entity_view_published_versions;
CREATE POLICY qryvanta_tenant_isolation ON entity_view_published_versions
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE runtime_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE runtime_records FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON runtime_records;
CREATE POLICY qryvanta_tenant_isolation ON runtime_records
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE runtime_record_unique_values ENABLE ROW LEVEL SECURITY;
ALTER TABLE runtime_record_unique_values FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON runtime_record_unique_values;
CREATE POLICY qryvanta_tenant_isolation ON runtime_record_unique_values
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());
