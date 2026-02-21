CREATE TABLE IF NOT EXISTS entity_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_logical_name TEXT NOT NULL,
    logical_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    field_type TEXT NOT NULL,
    is_required BOOLEAN NOT NULL DEFAULT false,
    is_unique BOOLEAN NOT NULL DEFAULT false,
    default_value JSONB,
    relation_target_entity TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, entity_logical_name, logical_name),
    CONSTRAINT fk_entity_fields_entity
        FOREIGN KEY (tenant_id, entity_logical_name)
        REFERENCES entity_definitions (tenant_id, logical_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_entity_fields_tenant_entity
    ON entity_fields (tenant_id, entity_logical_name);

CREATE TABLE IF NOT EXISTS entity_published_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_logical_name TEXT NOT NULL,
    version INTEGER NOT NULL,
    schema_json JSONB NOT NULL,
    published_by_subject TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, entity_logical_name, version),
    CONSTRAINT fk_entity_published_versions_entity
        FOREIGN KEY (tenant_id, entity_logical_name)
        REFERENCES entity_definitions (tenant_id, logical_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_entity_published_versions_latest
    ON entity_published_versions (tenant_id, entity_logical_name, version DESC);

CREATE TABLE IF NOT EXISTS runtime_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_logical_name TEXT NOT NULL,
    data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_runtime_records_entity
        FOREIGN KEY (tenant_id, entity_logical_name)
        REFERENCES entity_definitions (tenant_id, logical_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_runtime_records_tenant_entity_created
    ON runtime_records (tenant_id, entity_logical_name, created_at DESC);

CREATE TABLE IF NOT EXISTS runtime_record_unique_values (
    tenant_id UUID NOT NULL,
    entity_logical_name TEXT NOT NULL,
    field_logical_name TEXT NOT NULL,
    field_value_hash TEXT NOT NULL,
    record_id UUID NOT NULL REFERENCES runtime_records(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, entity_logical_name, field_logical_name, field_value_hash)
);

CREATE INDEX IF NOT EXISTS idx_runtime_record_unique_values_record
    ON runtime_record_unique_values (record_id);
