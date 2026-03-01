CREATE TABLE IF NOT EXISTS qrywell_sync_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    entity_logical_name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    operation TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 12,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_qrywell_sync_jobs_operation CHECK (operation IN ('upsert', 'delete')),
    CONSTRAINT chk_qrywell_sync_jobs_status CHECK (status IN ('pending', 'processing', 'failed')),
    CONSTRAINT uq_qrywell_sync_jobs_record UNIQUE (tenant_id, entity_logical_name, record_id)
);

CREATE INDEX IF NOT EXISTS idx_qrywell_sync_jobs_schedule
    ON qrywell_sync_jobs (status, next_attempt_at, created_at);
