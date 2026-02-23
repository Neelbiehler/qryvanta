ALTER TABLE workflow_worker_heartbeats
    ADD COLUMN IF NOT EXISTS partition_count INTEGER,
    ADD COLUMN IF NOT EXISTS partition_index INTEGER;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'chk_workflow_worker_heartbeats_partition_pair'
    ) THEN
        ALTER TABLE workflow_worker_heartbeats
            ADD CONSTRAINT chk_workflow_worker_heartbeats_partition_pair
            CHECK (
                (partition_count IS NULL AND partition_index IS NULL)
                OR (
                    partition_count IS NOT NULL
                    AND partition_index IS NOT NULL
                    AND partition_count > 0
                    AND partition_index >= 0
                    AND partition_index < partition_count
                )
            );
    END IF;
END;
$$;

CREATE INDEX IF NOT EXISTS idx_workflow_worker_heartbeats_partition_last_seen
    ON workflow_worker_heartbeats (partition_count, partition_index, last_seen_at DESC);
