ALTER TABLE workflow_definitions
    DROP CONSTRAINT IF EXISTS chk_workflow_definitions_trigger_type;

ALTER TABLE workflow_definitions
    ADD CONSTRAINT chk_workflow_definitions_trigger_type
        CHECK (
            trigger_type IN (
                'manual',
                'runtime_record_created',
                'runtime_record_updated',
                'runtime_record_deleted',
                'schedule_tick',
                'webhook_received'
            )
        );
