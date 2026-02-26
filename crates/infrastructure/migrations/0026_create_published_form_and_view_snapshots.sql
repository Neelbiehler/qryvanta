CREATE TABLE IF NOT EXISTS entity_form_published_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_logical_name TEXT NOT NULL,
    published_schema_version INTEGER NOT NULL,
    logical_name TEXT NOT NULL,
    definition_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, entity_logical_name, published_schema_version, logical_name),
    CONSTRAINT fk_entity_form_published_versions_entity
        FOREIGN KEY (tenant_id, entity_logical_name)
        REFERENCES entity_definitions (tenant_id, logical_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_entity_form_published_versions_latest
    ON entity_form_published_versions (tenant_id, entity_logical_name, published_schema_version DESC);

CREATE TABLE IF NOT EXISTS entity_view_published_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_logical_name TEXT NOT NULL,
    published_schema_version INTEGER NOT NULL,
    logical_name TEXT NOT NULL,
    definition_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, entity_logical_name, published_schema_version, logical_name),
    CONSTRAINT fk_entity_view_published_versions_entity
        FOREIGN KEY (tenant_id, entity_logical_name)
        REFERENCES entity_definitions (tenant_id, logical_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_entity_view_published_versions_latest
    ON entity_view_published_versions (tenant_id, entity_logical_name, published_schema_version DESC);
