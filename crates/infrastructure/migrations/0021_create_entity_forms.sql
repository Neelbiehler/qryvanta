CREATE TABLE IF NOT EXISTS entity_forms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    entity_logical_name TEXT NOT NULL,
    logical_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    form_type TEXT NOT NULL,
    definition_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, entity_logical_name, logical_name),
    CONSTRAINT fk_entity_forms_entity
        FOREIGN KEY (tenant_id, entity_logical_name)
        REFERENCES entity_definitions (tenant_id, logical_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_entity_forms_tenant_entity
    ON entity_forms (tenant_id, entity_logical_name);
