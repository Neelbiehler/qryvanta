ALTER TABLE app_entity_bindings
    ADD COLUMN IF NOT EXISTS forms JSONB NOT NULL DEFAULT '[]'::JSONB;

ALTER TABLE app_entity_bindings
    ADD COLUMN IF NOT EXISTS list_views JSONB NOT NULL DEFAULT '[]'::JSONB;

ALTER TABLE app_entity_bindings
    ADD COLUMN IF NOT EXISTS default_form_logical_name TEXT;

ALTER TABLE app_entity_bindings
    ADD COLUMN IF NOT EXISTS default_list_view_logical_name TEXT;

UPDATE app_entity_bindings
SET forms = jsonb_build_array(
    jsonb_build_object(
        'logical_name', 'main_form',
        'display_name', 'Main Form',
        'field_logical_names', form_field_logical_names
    )
)
WHERE forms = '[]'::JSONB;

UPDATE app_entity_bindings
SET list_views = jsonb_build_array(
    jsonb_build_object(
        'logical_name', 'main_view',
        'display_name', 'Main View',
        'field_logical_names', list_field_logical_names
    )
)
WHERE list_views = '[]'::JSONB;

UPDATE app_entity_bindings
SET default_form_logical_name = 'main_form'
WHERE default_form_logical_name IS NULL;

UPDATE app_entity_bindings
SET default_list_view_logical_name = 'main_view'
WHERE default_list_view_logical_name IS NULL;
