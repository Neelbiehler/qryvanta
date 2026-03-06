ALTER TABLE users
    ADD COLUMN IF NOT EXISTS default_tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_users_default_tenant_id
    ON users (default_tenant_id)
    WHERE default_tenant_id IS NOT NULL;
