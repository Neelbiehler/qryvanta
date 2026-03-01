# Qrywell Search Integration

Qryvanta can use Qrywell as an external search engine for retrieval over indexed knowledge.

## Configuration

- `QRYWELL_API_BASE_URL` (for example `http://127.0.0.1:4201`)
- `QRYWELL_API_KEY` (optional, forwarded as `x-qrywell-api-key`)
- `QRYWELL_SYNC_POLL_INTERVAL_MS` (default `3000`)
- `QRYWELL_SYNC_BATCH_SIZE` (default `25`)
- `QRYWELL_SYNC_MAX_ATTEMPTS` (default `12`)

## Qryvanta Endpoints

- `POST /api/search/qrywell`
- `POST /api/search/qrywell/events/click`
- `GET /api/search/qrywell/analytics`
- `POST /api/search/qrywell/sync/{entity_logical_name}`
- `POST /api/search/qrywell/sync-all`
- `GET /api/search/qrywell/queue-health`

## Runtime Behavior

- Qryvanta forwards query requests to Qrywell `POST /v0/search`.
- Viewer context is passed from authenticated session (`user_id`, `tenant_id`, `roles`).
- Qryvanta builds metadata-driven facet filters from tenant schema.
- Optional debug telemetry is returned when `include_debug=true`.

## Sync Model

Runtime record writes enqueue sync jobs in Postgres (`qrywell_sync_jobs`):

- Create -> upsert job
- Update -> upsert job
- Delete -> delete job

Queue health includes pending/processing/failed counts and recent failure details.

## Ingestion Endpoints on Qrywell

- `POST /v0/connectors/qryvanta/records:ingest`
- `POST /v0/connectors/qryvanta/records:delete`
