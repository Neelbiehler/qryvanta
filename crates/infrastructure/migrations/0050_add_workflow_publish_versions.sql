CREATE TABLE IF NOT EXISTS workflow_published_versions (
    tenant_id UUID NOT NULL,
    logical_name TEXT NOT NULL,
    version INTEGER NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    trigger_type TEXT NOT NULL,
    trigger_entity_logical_name TEXT,
    steps JSONB NOT NULL,
    max_attempts SMALLINT NOT NULL,
    published_by_subject TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, logical_name, version),
    CONSTRAINT fk_workflow_published_versions_workflow
        FOREIGN KEY (tenant_id, logical_name)
        REFERENCES workflow_definitions (tenant_id, logical_name)
        ON DELETE CASCADE,
    CONSTRAINT chk_workflow_published_versions_trigger_type
        CHECK (
            trigger_type IN (
                'manual',
                'runtime_record_created',
                'runtime_record_updated',
                'runtime_record_deleted',
                'schedule_tick',
                'webhook_received',
                'form_submitted',
                'inbound_email_received',
                'approval_event_received'
            )
        ),
    CONSTRAINT chk_workflow_published_versions_max_attempts
        CHECK (max_attempts > 0 AND max_attempts <= 10),
    CONSTRAINT chk_workflow_published_versions_steps_json_array
        CHECK (jsonb_typeof(steps) = 'array')
);

ALTER TABLE workflow_published_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_published_versions FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS qryvanta_tenant_isolation ON workflow_published_versions;
CREATE POLICY qryvanta_tenant_isolation ON workflow_published_versions
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID)
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id')::UUID);

ALTER TABLE workflow_definitions
    ADD COLUMN IF NOT EXISTS lifecycle_state TEXT NOT NULL DEFAULT 'draft',
    ADD COLUMN IF NOT EXISTS current_published_version INTEGER;

DROP INDEX IF EXISTS idx_workflow_definitions_trigger_lookup;

INSERT INTO workflow_published_versions (
    tenant_id,
    logical_name,
    version,
    display_name,
    description,
    trigger_type,
    trigger_entity_logical_name,
    steps,
    max_attempts,
    published_by_subject,
    published_at
)
SELECT
    tenant_id,
    logical_name,
    1,
    display_name,
    description,
    trigger_type,
    trigger_entity_logical_name,
    steps,
    max_attempts,
    'workflow-migration',
    updated_at
FROM workflow_definitions
ON CONFLICT (tenant_id, logical_name, version) DO NOTHING;

UPDATE workflow_definitions
SET
    lifecycle_state = CASE
        WHEN is_enabled THEN 'published'
        ELSE 'disabled'
    END,
    current_published_version = COALESCE(current_published_version, 1)
WHERE current_published_version IS NULL;

ALTER TABLE workflow_definitions
    DROP COLUMN IF EXISTS is_enabled;

ALTER TABLE workflow_definitions
    ADD CONSTRAINT chk_workflow_definitions_lifecycle_state
        CHECK (lifecycle_state IN ('draft', 'published', 'disabled'));

CREATE INDEX IF NOT EXISTS idx_workflow_definitions_publish_lookup
    ON workflow_definitions (
        tenant_id,
        lifecycle_state,
        logical_name,
        current_published_version
    );

CREATE INDEX IF NOT EXISTS idx_workflow_published_versions_trigger_lookup
    ON workflow_published_versions (
        tenant_id,
        trigger_type,
        trigger_entity_logical_name,
        logical_name,
        version
    );

ALTER TABLE workflow_execution_runs
    ADD COLUMN IF NOT EXISTS workflow_version INTEGER;

UPDATE workflow_execution_runs runs
SET workflow_version = definitions.current_published_version
FROM workflow_definitions definitions
WHERE definitions.tenant_id = runs.tenant_id
  AND definitions.logical_name = runs.workflow_logical_name
  AND runs.workflow_version IS NULL;

ALTER TABLE workflow_execution_runs
    ALTER COLUMN workflow_version SET NOT NULL;

ALTER TABLE workflow_execution_runs
    DROP CONSTRAINT IF EXISTS fk_workflow_execution_runs_workflow;

ALTER TABLE workflow_execution_runs
    ADD CONSTRAINT fk_workflow_execution_runs_workflow_version
        FOREIGN KEY (tenant_id, workflow_logical_name, workflow_version)
        REFERENCES workflow_published_versions (tenant_id, logical_name, version)
        ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS idx_workflow_execution_runs_version_lookup
    ON workflow_execution_runs (tenant_id, workflow_logical_name, workflow_version, started_at DESC);
