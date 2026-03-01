CREATE TABLE IF NOT EXISTS qrywell_search_query_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_subject TEXT NOT NULL,
    query TEXT NOT NULL,
    normalized_query TEXT NOT NULL,
    total_hits INTEGER NOT NULL,
    selected_entity TEXT,
    planned_filter_count INTEGER NOT NULL,
    negated_filter_count INTEGER NOT NULL,
    clicked_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_qrywell_search_query_events_tenant_time
    ON qrywell_search_query_events (tenant_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_qrywell_search_query_events_tenant_query
    ON qrywell_search_query_events (tenant_id, normalized_query);
