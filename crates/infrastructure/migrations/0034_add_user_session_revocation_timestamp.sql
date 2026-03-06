ALTER TABLE users
    ADD COLUMN IF NOT EXISTS auth_sessions_revoked_after TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_users_auth_sessions_revoked_after
    ON users (auth_sessions_revoked_after)
    WHERE auth_sessions_revoked_after IS NOT NULL;
