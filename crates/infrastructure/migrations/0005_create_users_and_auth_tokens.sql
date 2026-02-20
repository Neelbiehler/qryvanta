-- Phase 1: Introduce first-class users table and auth token infrastructure.
-- Supports email+password auth, TOTP MFA, email verification, password reset, and invites.

CREATE TABLE IF NOT EXISTS users (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email               TEXT NOT NULL,
    email_verified      BOOLEAN NOT NULL DEFAULT FALSE,
    password_hash       TEXT,
    totp_secret_enc     BYTEA,
    totp_enabled        BOOLEAN NOT NULL DEFAULT FALSE,
    recovery_codes_hash JSONB,
    failed_login_count  INTEGER NOT NULL DEFAULT 0,
    locked_until        TIMESTAMPTZ,
    password_changed_at TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email_unique
    ON users (LOWER(email));

-- Auth tokens for email verification, password reset, and invites.
CREATE TABLE IF NOT EXISTS auth_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID REFERENCES users(id) ON DELETE CASCADE,
    email       TEXT NOT NULL,
    token_hash  TEXT NOT NULL,
    token_type  TEXT NOT NULL,
    expires_at  TIMESTAMPTZ NOT NULL,
    used_at     TIMESTAMPTZ,
    metadata    JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_auth_tokens_hash
    ON auth_tokens (token_hash);
CREATE INDEX IF NOT EXISTS idx_auth_tokens_user
    ON auth_tokens (user_id);

-- Link tenant_memberships to users table.
ALTER TABLE tenant_memberships
    ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id);

CREATE INDEX IF NOT EXISTS idx_memberships_user_id
    ON tenant_memberships (user_id);

-- Link passkey_credentials to users table.
ALTER TABLE passkey_credentials
    ADD COLUMN IF NOT EXISTS user_id UUID REFERENCES users(id);

CREATE INDEX IF NOT EXISTS idx_passkey_credentials_user_id
    ON passkey_credentials (user_id);

-- Add registration mode to tenants.
ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS registration_mode TEXT NOT NULL DEFAULT 'invite_only';
