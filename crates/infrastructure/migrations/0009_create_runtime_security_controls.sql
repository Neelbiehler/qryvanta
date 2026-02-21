ALTER TABLE runtime_records
    ADD COLUMN IF NOT EXISTS created_by_subject TEXT;

UPDATE runtime_records
SET created_by_subject = 'system'
WHERE created_by_subject IS NULL;

ALTER TABLE runtime_records
    ALTER COLUMN created_by_subject SET NOT NULL;

CREATE INDEX IF NOT EXISTS idx_runtime_records_owner_lookup
    ON runtime_records (tenant_id, entity_logical_name, created_by_subject, created_at DESC);

CREATE TABLE IF NOT EXISTS runtime_subject_field_permissions (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    subject TEXT NOT NULL,
    entity_logical_name TEXT NOT NULL,
    field_logical_name TEXT NOT NULL,
    can_read BOOLEAN NOT NULL DEFAULT false,
    can_write BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, subject, entity_logical_name, field_logical_name)
);

CREATE INDEX IF NOT EXISTS idx_runtime_subject_field_permissions_lookup
    ON runtime_subject_field_permissions (tenant_id, subject, entity_logical_name);

CREATE TABLE IF NOT EXISTS security_temporary_access_grants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    subject TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_by_subject TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    revoked_by_subject TEXT,
    revoke_reason TEXT
);

CREATE INDEX IF NOT EXISTS idx_security_temp_access_active
    ON security_temporary_access_grants (tenant_id, subject, expires_at)
    WHERE revoked_at IS NULL;

CREATE TABLE IF NOT EXISTS security_temporary_access_grant_permissions (
    grant_id UUID NOT NULL REFERENCES security_temporary_access_grants(id) ON DELETE CASCADE,
    permission TEXT NOT NULL,
    PRIMARY KEY (grant_id, permission)
);

ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS audit_retention_days INTEGER NOT NULL DEFAULT 365;
