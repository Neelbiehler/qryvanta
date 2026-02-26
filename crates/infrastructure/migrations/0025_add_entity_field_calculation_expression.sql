ALTER TABLE entity_fields
    ADD COLUMN IF NOT EXISTS calculation_expression TEXT;
