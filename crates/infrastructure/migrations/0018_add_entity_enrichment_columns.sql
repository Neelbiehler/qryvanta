ALTER TABLE entity_definitions
    ADD COLUMN IF NOT EXISTS description TEXT,
    ADD COLUMN IF NOT EXISTS plural_display_name TEXT,
    ADD COLUMN IF NOT EXISTS icon TEXT;
