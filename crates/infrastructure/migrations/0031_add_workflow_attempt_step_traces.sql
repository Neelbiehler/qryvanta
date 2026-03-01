ALTER TABLE workflow_execution_attempts
    ADD COLUMN IF NOT EXISTS step_traces JSONB NOT NULL DEFAULT '[]'::jsonb;
