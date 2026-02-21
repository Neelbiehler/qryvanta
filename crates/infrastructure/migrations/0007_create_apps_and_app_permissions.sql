CREATE TABLE IF NOT EXISTS app_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    logical_name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (tenant_id, logical_name)
);

CREATE INDEX IF NOT EXISTS idx_app_definitions_tenant
    ON app_definitions (tenant_id, logical_name);

CREATE TABLE IF NOT EXISTS app_entity_bindings (
    tenant_id UUID NOT NULL,
    app_logical_name TEXT NOT NULL,
    entity_logical_name TEXT NOT NULL,
    navigation_label TEXT,
    navigation_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, app_logical_name, entity_logical_name),
    CONSTRAINT fk_app_entity_bindings_app
        FOREIGN KEY (tenant_id, app_logical_name)
        REFERENCES app_definitions (tenant_id, logical_name)
        ON DELETE CASCADE,
    CONSTRAINT fk_app_entity_bindings_entity
        FOREIGN KEY (tenant_id, entity_logical_name)
        REFERENCES entity_definitions (tenant_id, logical_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_app_entity_bindings_lookup
    ON app_entity_bindings (tenant_id, app_logical_name, navigation_order, entity_logical_name);

CREATE TABLE IF NOT EXISTS app_role_bindings (
    tenant_id UUID NOT NULL,
    app_logical_name TEXT NOT NULL,
    role_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, app_logical_name, role_id),
    CONSTRAINT fk_app_role_bindings_app
        FOREIGN KEY (tenant_id, app_logical_name)
        REFERENCES app_definitions (tenant_id, logical_name)
        ON DELETE CASCADE,
    CONSTRAINT fk_app_role_bindings_role
        FOREIGN KEY (role_id)
        REFERENCES rbac_roles (id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_app_role_bindings_role
    ON app_role_bindings (role_id, tenant_id, app_logical_name);

CREATE TABLE IF NOT EXISTS app_role_entity_permissions (
    tenant_id UUID NOT NULL,
    app_logical_name TEXT NOT NULL,
    role_id UUID NOT NULL,
    entity_logical_name TEXT NOT NULL,
    can_read BOOLEAN NOT NULL DEFAULT false,
    can_create BOOLEAN NOT NULL DEFAULT false,
    can_update BOOLEAN NOT NULL DEFAULT false,
    can_delete BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, app_logical_name, role_id, entity_logical_name),
    CONSTRAINT fk_app_role_entity_permissions_app
        FOREIGN KEY (tenant_id, app_logical_name)
        REFERENCES app_definitions (tenant_id, logical_name)
        ON DELETE CASCADE,
    CONSTRAINT fk_app_role_entity_permissions_role
        FOREIGN KEY (role_id)
        REFERENCES rbac_roles (id)
        ON DELETE CASCADE,
    CONSTRAINT fk_app_role_entity_permissions_entity
        FOREIGN KEY (tenant_id, entity_logical_name)
        REFERENCES entity_definitions (tenant_id, logical_name)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_app_role_entity_permissions_lookup
    ON app_role_entity_permissions (tenant_id, app_logical_name, entity_logical_name);
