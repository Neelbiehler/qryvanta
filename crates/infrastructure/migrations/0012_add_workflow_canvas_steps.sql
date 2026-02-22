ALTER TABLE workflow_definitions
    ADD COLUMN IF NOT EXISTS action_steps JSONB;
