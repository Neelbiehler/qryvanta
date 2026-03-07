CREATE TABLE IF NOT EXISTS workflow_schedule_ticks (
    tenant_id UUID NOT NULL,
    schedule_key TEXT NOT NULL,
    slot_key TEXT NOT NULL,
    scheduled_for TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    last_error TEXT,
    leased_by TEXT,
    lease_token TEXT,
    lease_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at TIMESTAMPTZ,
    CONSTRAINT pk_workflow_schedule_ticks
        PRIMARY KEY (tenant_id, schedule_key, slot_key),
    CONSTRAINT fk_workflow_schedule_ticks_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES tenants (id)
        ON DELETE CASCADE,
    CONSTRAINT chk_workflow_schedule_ticks_status
        CHECK (status IN ('pending', 'leased', 'completed')),
    CONSTRAINT chk_workflow_schedule_ticks_lease_token_required
        CHECK (
            (status = 'leased' AND leased_by IS NOT NULL AND lease_token IS NOT NULL)
            OR (status <> 'leased')
        )
);

CREATE INDEX IF NOT EXISTS idx_workflow_schedule_ticks_claim
    ON workflow_schedule_ticks (status, lease_expires_at, scheduled_for);

CREATE INDEX IF NOT EXISTS idx_workflow_schedule_ticks_tenant
    ON workflow_schedule_ticks (tenant_id, status, scheduled_for);

ALTER TABLE workflow_schedule_ticks ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_schedule_ticks FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON workflow_schedule_ticks;
CREATE POLICY qryvanta_tenant_isolation ON workflow_schedule_ticks
    USING (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    )
    WITH CHECK (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    );
