CREATE TABLE IF NOT EXISTS workflow_runtime_trigger_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    trigger_type TEXT NOT NULL,
    entity_logical_name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    emitted_by_subject TEXT NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}'::JSONB,
    status TEXT NOT NULL DEFAULT 'pending',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    leased_by TEXT,
    lease_token TEXT,
    lease_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at TIMESTAMPTZ,
    CONSTRAINT fk_workflow_runtime_trigger_events_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES tenants (id)
        ON DELETE CASCADE,
    CONSTRAINT chk_workflow_runtime_trigger_events_trigger_type
        CHECK (
            trigger_type IN (
                'runtime_record_created',
                'runtime_record_updated',
                'runtime_record_deleted'
            )
        ),
    CONSTRAINT chk_workflow_runtime_trigger_events_status
        CHECK (status IN ('pending', 'leased', 'completed')),
    CONSTRAINT chk_workflow_runtime_trigger_events_lease_token_required
        CHECK (
            (status = 'leased' AND leased_by IS NOT NULL AND lease_token IS NOT NULL)
            OR (status <> 'leased')
        )
);

CREATE INDEX IF NOT EXISTS idx_workflow_runtime_trigger_events_claim
    ON workflow_runtime_trigger_events (status, lease_expires_at, created_at);

CREATE INDEX IF NOT EXISTS idx_workflow_runtime_trigger_events_tenant
    ON workflow_runtime_trigger_events (tenant_id, status, created_at);

ALTER TABLE workflow_runtime_trigger_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_runtime_trigger_events FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON workflow_runtime_trigger_events;
CREATE POLICY qryvanta_tenant_isolation ON workflow_runtime_trigger_events
    USING (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    )
    WITH CHECK (
        tenant_id = qryvanta_current_tenant_id()
        OR qryvanta_rls_scope('workflow_queue')
    );
