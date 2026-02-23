ALTER TABLE workflow_execution_jobs
    ADD COLUMN IF NOT EXISTS lease_token TEXT;

UPDATE workflow_execution_jobs
SET lease_token = gen_random_uuid()::TEXT
WHERE status = 'leased'
  AND lease_token IS NULL;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_workflow_execution_jobs_lease_token_required'
    ) THEN
        ALTER TABLE workflow_execution_jobs
            ADD CONSTRAINT chk_workflow_execution_jobs_lease_token_required
            CHECK (
                status <> 'leased'
                OR lease_token IS NOT NULL
            );
    END IF;
END;
$$;
