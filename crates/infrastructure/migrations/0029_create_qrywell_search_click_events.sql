CREATE TABLE IF NOT EXISTS qrywell_search_click_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_subject TEXT NOT NULL,
    query TEXT NOT NULL,
    result_id TEXT NOT NULL,
    title TEXT NOT NULL,
    connector_type TEXT NOT NULL,
    rank INTEGER NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    group_label TEXT,
    clicked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_qrywell_search_click_events_tenant_time
    ON qrywell_search_click_events (tenant_id, clicked_at DESC);
