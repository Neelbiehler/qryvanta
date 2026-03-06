ALTER TABLE audit_log_entries
    ADD COLUMN IF NOT EXISTS chain_position BIGINT,
    ADD COLUMN IF NOT EXISTS previous_entry_hash TEXT,
    ADD COLUMN IF NOT EXISTS entry_hash TEXT;

WITH RECURSIVE ordered_entries AS (
    SELECT
        id,
        tenant_id,
        subject,
        action,
        resource_type,
        resource_id,
        COALESCE(detail, '') AS detail_value,
        to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at_utc,
        ROW_NUMBER() OVER (
            PARTITION BY tenant_id
            ORDER BY created_at, id
        ) AS chain_position
    FROM audit_log_entries
),
hashed_entries AS (
    SELECT
        ordered_entries.id,
        ordered_entries.tenant_id,
        ordered_entries.chain_position,
        NULL::TEXT AS previous_entry_hash,
        encode(
            digest(
                concat_ws(
                    ':',
                    'qryvanta-audit-chain-v1',
                    ordered_entries.tenant_id::text,
                    ordered_entries.chain_position::text,
                    '',
                    ordered_entries.subject,
                    ordered_entries.action,
                    ordered_entries.resource_type,
                    ordered_entries.resource_id,
                    ordered_entries.detail_value,
                    ordered_entries.created_at_utc
                ),
                'sha256'
            ),
            'hex'
        ) AS entry_hash
    FROM ordered_entries
    WHERE ordered_entries.chain_position = 1

    UNION ALL

    SELECT
        ordered_entries.id,
        ordered_entries.tenant_id,
        ordered_entries.chain_position,
        hashed_entries.entry_hash AS previous_entry_hash,
        encode(
            digest(
                concat_ws(
                    ':',
                    'qryvanta-audit-chain-v1',
                    ordered_entries.tenant_id::text,
                    ordered_entries.chain_position::text,
                    hashed_entries.entry_hash,
                    ordered_entries.subject,
                    ordered_entries.action,
                    ordered_entries.resource_type,
                    ordered_entries.resource_id,
                    ordered_entries.detail_value,
                    ordered_entries.created_at_utc
                ),
                'sha256'
            ),
            'hex'
        ) AS entry_hash
    FROM ordered_entries
    JOIN hashed_entries
      ON hashed_entries.tenant_id = ordered_entries.tenant_id
     AND hashed_entries.chain_position = ordered_entries.chain_position - 1
)
UPDATE audit_log_entries target
SET chain_position = hashed_entries.chain_position,
    previous_entry_hash = hashed_entries.previous_entry_hash,
    entry_hash = hashed_entries.entry_hash
FROM hashed_entries
WHERE hashed_entries.id = target.id;

ALTER TABLE audit_log_entries
    ALTER COLUMN chain_position SET NOT NULL,
    ALTER COLUMN entry_hash SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_audit_log_entries_tenant_chain_position
    ON audit_log_entries (tenant_id, chain_position);
