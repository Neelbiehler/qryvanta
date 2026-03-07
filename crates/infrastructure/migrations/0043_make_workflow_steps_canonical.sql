UPDATE workflow_definitions
SET action_steps = CASE action_type
    WHEN 'log_message' THEN jsonb_build_array(
        jsonb_build_object(
            'type',
            'log_message',
            'message',
            COALESCE(action_payload ->> 'message', '')
        )
    )
    WHEN 'create_runtime_record' THEN jsonb_build_array(
        jsonb_build_object(
            'type',
            'create_runtime_record',
            'entity_logical_name',
            COALESCE(action_entity_logical_name, ''),
            'data',
            COALESCE(action_payload, '{}'::JSONB)
        )
    )
    ELSE '[]'::JSONB
END
WHERE action_steps IS NULL;

ALTER TABLE workflow_definitions
    RENAME COLUMN action_steps TO steps;

ALTER TABLE workflow_definitions
    ALTER COLUMN steps SET NOT NULL;

ALTER TABLE workflow_definitions
    DROP CONSTRAINT IF EXISTS chk_workflow_definitions_action_type;

ALTER TABLE workflow_definitions
    DROP COLUMN action_type,
    DROP COLUMN action_entity_logical_name,
    DROP COLUMN action_payload;

ALTER TABLE workflow_definitions
    ADD CONSTRAINT chk_workflow_definitions_steps_json_array
        CHECK (jsonb_typeof(steps) = 'array');
