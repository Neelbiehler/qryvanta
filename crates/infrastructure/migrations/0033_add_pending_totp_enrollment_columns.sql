ALTER TABLE users
    ADD COLUMN IF NOT EXISTS totp_pending_secret_enc BYTEA,
    ADD COLUMN IF NOT EXISTS recovery_codes_pending_hash JSONB;
