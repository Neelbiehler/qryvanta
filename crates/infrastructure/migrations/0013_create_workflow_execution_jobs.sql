CREATE TABLE IF NOT EXISTS workflow_execution_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    run_id UUID NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'pending',
    leased_by TEXT,
    lease_expires_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_workflow_execution_jobs_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES tenants (id)
        ON DELETE CASCADE,
    CONSTRAINT fk_workflow_execution_jobs_run
        FOREIGN KEY (run_id)
        REFERENCES workflow_execution_runs (id)
        ON DELETE CASCADE,
    CONSTRAINT chk_workflow_execution_jobs_status
        CHECK (status IN ('pending', 'leased', 'completed', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_workflow_execution_jobs_claim
    ON workflow_execution_jobs (status, lease_expires_at, created_at);
