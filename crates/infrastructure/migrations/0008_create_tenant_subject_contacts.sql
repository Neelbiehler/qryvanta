CREATE TABLE IF NOT EXISTS tenant_subject_contacts (
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    subject TEXT NOT NULL,
    contact_record_id UUID NOT NULL REFERENCES runtime_records(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, subject),
    UNIQUE (tenant_id, contact_record_id)
);

CREATE INDEX IF NOT EXISTS idx_tenant_subject_contacts_record
    ON tenant_subject_contacts (contact_record_id);
