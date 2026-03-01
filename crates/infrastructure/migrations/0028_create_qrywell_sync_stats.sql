CREATE TABLE IF NOT EXISTS qrywell_sync_stats (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id) ON DELETE CASCADE,
    last_attempt_at TIMESTAMPTZ,
    last_success_at TIMESTAMPTZ,
    last_failure_at TIMESTAMPTZ,
    total_succeeded BIGINT NOT NULL DEFAULT 0,
    total_failed BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
