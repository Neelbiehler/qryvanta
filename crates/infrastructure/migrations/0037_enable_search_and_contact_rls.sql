ALTER TABLE tenant_subject_contacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_subject_contacts FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON tenant_subject_contacts;
CREATE POLICY qryvanta_tenant_isolation ON tenant_subject_contacts
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE qrywell_sync_stats ENABLE ROW LEVEL SECURITY;
ALTER TABLE qrywell_sync_stats FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON qrywell_sync_stats;
CREATE POLICY qryvanta_tenant_isolation ON qrywell_sync_stats
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE qrywell_search_click_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE qrywell_search_click_events FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON qrywell_search_click_events;
CREATE POLICY qryvanta_tenant_isolation ON qrywell_search_click_events
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());

ALTER TABLE qrywell_search_query_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE qrywell_search_query_events FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON qrywell_search_query_events;
CREATE POLICY qryvanta_tenant_isolation ON qrywell_search_query_events
    USING (tenant_id = qryvanta_current_tenant_id())
    WITH CHECK (tenant_id = qryvanta_current_tenant_id());
