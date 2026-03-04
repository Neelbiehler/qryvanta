CREATE TABLE IF NOT EXISTS extension_definitions (
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    logical_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    package_version TEXT NOT NULL,
    runtime_api_version TEXT NOT NULL,
    runtime_kind TEXT NOT NULL,
    package_sha256 TEXT NOT NULL,
    lifecycle_state TEXT NOT NULL,
    requested_capabilities TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    isolation_policy_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, logical_name)
);

CREATE INDEX IF NOT EXISTS idx_extension_definitions_tenant_lifecycle
    ON extension_definitions (tenant_id, lifecycle_state);
