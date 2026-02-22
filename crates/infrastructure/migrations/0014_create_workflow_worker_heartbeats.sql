CREATE TABLE IF NOT EXISTS workflow_worker_heartbeats (
    worker_id TEXT PRIMARY KEY,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_claimed_jobs BIGINT NOT NULL DEFAULT 0,
    last_executed_jobs BIGINT NOT NULL DEFAULT 0,
    last_failed_jobs BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_workflow_worker_heartbeats_last_seen
    ON workflow_worker_heartbeats (last_seen_at DESC);
