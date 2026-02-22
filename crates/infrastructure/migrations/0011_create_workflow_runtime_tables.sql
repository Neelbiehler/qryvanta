CREATE TABLE IF NOT EXISTS workflow_definitions (
    tenant_id UUID NOT NULL,
    logical_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    trigger_type TEXT NOT NULL,
    trigger_entity_logical_name TEXT,
    action_type TEXT NOT NULL,
    action_entity_logical_name TEXT,
    action_payload JSONB NOT NULL DEFAULT '{}'::JSONB,
    max_attempts SMALLINT NOT NULL DEFAULT 3,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, logical_name),
    CONSTRAINT fk_workflow_definitions_tenant
        FOREIGN KEY (tenant_id)
        REFERENCES tenants (id)
        ON DELETE CASCADE,
    CONSTRAINT chk_workflow_definitions_trigger_type
        CHECK (trigger_type IN ('manual', 'runtime_record_created')),
    CONSTRAINT chk_workflow_definitions_action_type
        CHECK (action_type IN ('log_message', 'create_runtime_record')),
    CONSTRAINT chk_workflow_definitions_max_attempts
        CHECK (max_attempts > 0 AND max_attempts <= 10)
);

CREATE INDEX IF NOT EXISTS idx_workflow_definitions_trigger_lookup
    ON workflow_definitions (
        tenant_id,
        is_enabled,
        trigger_type,
        trigger_entity_logical_name,
        logical_name
    );

CREATE TABLE IF NOT EXISTS workflow_execution_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    workflow_logical_name TEXT NOT NULL,
    trigger_type TEXT NOT NULL,
    trigger_entity_logical_name TEXT,
    trigger_payload JSONB NOT NULL DEFAULT '{}'::JSONB,
    status TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    dead_letter_reason TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ,
    CONSTRAINT fk_workflow_execution_runs_workflow
        FOREIGN KEY (tenant_id, workflow_logical_name)
        REFERENCES workflow_definitions (tenant_id, logical_name)
        ON DELETE CASCADE,
    CONSTRAINT chk_workflow_execution_runs_status
        CHECK (status IN ('running', 'succeeded', 'dead_lettered'))
);

CREATE INDEX IF NOT EXISTS idx_workflow_execution_runs_lookup
    ON workflow_execution_runs (tenant_id, workflow_logical_name, started_at DESC);

CREATE TABLE IF NOT EXISTS workflow_execution_attempts (
    run_id UUID NOT NULL,
    tenant_id UUID NOT NULL,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    error_message TEXT,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (run_id, attempt_number),
    CONSTRAINT fk_workflow_execution_attempts_run
        FOREIGN KEY (run_id)
        REFERENCES workflow_execution_runs (id)
        ON DELETE CASCADE,
    CONSTRAINT chk_workflow_execution_attempts_status
        CHECK (status IN ('succeeded', 'failed'))
);

CREATE INDEX IF NOT EXISTS idx_workflow_execution_attempts_lookup
    ON workflow_execution_attempts (tenant_id, run_id, attempt_number);
