ALTER TABLE app_entity_bindings
    ADD COLUMN IF NOT EXISTS form_field_logical_names TEXT[] NOT NULL DEFAULT '{}'::TEXT[];

ALTER TABLE app_entity_bindings
    ADD COLUMN IF NOT EXISTS list_field_logical_names TEXT[] NOT NULL DEFAULT '{}'::TEXT[];

ALTER TABLE app_entity_bindings
    ADD COLUMN IF NOT EXISTS default_view_mode TEXT NOT NULL DEFAULT 'grid';

ALTER TABLE app_entity_bindings
    DROP CONSTRAINT IF EXISTS chk_app_entity_bindings_default_view_mode;

ALTER TABLE app_entity_bindings
    ADD CONSTRAINT chk_app_entity_bindings_default_view_mode
        CHECK (default_view_mode IN ('grid', 'json'));
