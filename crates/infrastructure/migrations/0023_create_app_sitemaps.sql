CREATE TABLE IF NOT EXISTS app_sitemaps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    app_logical_name TEXT NOT NULL,
    definition_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, app_logical_name),
    CONSTRAINT fk_app_sitemaps_app
        FOREIGN KEY (tenant_id, app_logical_name)
        REFERENCES app_definitions (tenant_id, logical_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_app_sitemaps_tenant_app
    ON app_sitemaps (tenant_id, app_logical_name);
