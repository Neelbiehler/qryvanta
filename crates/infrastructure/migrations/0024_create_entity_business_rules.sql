CREATE TABLE IF NOT EXISTS entity_business_rules (
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    entity_logical_name TEXT NOT NULL,
    logical_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    scope TEXT NOT NULL,
    definition_json JSONB NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, entity_logical_name, logical_name),
    CONSTRAINT entity_business_rules_scope_ck CHECK (scope IN ('entity', 'form'))
);

CREATE INDEX IF NOT EXISTS idx_entity_business_rules_tenant_entity
    ON entity_business_rules (tenant_id, entity_logical_name);
