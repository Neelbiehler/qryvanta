#!/usr/bin/env bash
set -euo pipefail

PG_ADMIN_URL="${PG_ADMIN_URL:-postgres://qryvanta:qryvanta@127.0.0.1:5432/postgres}"
PRIMARY_DB_NAME="${PRIMARY_DB_NAME:-qryvanta_ci}"
RESTORE_DB_NAME="${RESTORE_DB_NAME:-qryvanta_ci_restore}"
BACKUP_PATH="${BACKUP_PATH:-/tmp/qryvanta_ci.backup}"

create_db() {
  local db_name="$1"
  psql "$PG_ADMIN_URL" -v ON_ERROR_STOP=1 -c "DROP DATABASE IF EXISTS ${db_name};"
  psql "$PG_ADMIN_URL" -v ON_ERROR_STOP=1 -c "CREATE DATABASE ${db_name};"
}

run_migrate() {
  local db_url="$1"
  DATABASE_URL="$db_url" \
  AUTH_BOOTSTRAP_TOKEN="ci-bootstrap-token" \
  SESSION_SECRET="ci-session-secret-ci-session-secret-32" \
  TOTP_ENCRYPTION_KEY="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef" \
  cargo run -p qryvanta-api -- migrate
}

PRIMARY_DB_URL="postgres://qryvanta:qryvanta@127.0.0.1:5432/${PRIMARY_DB_NAME}"
RESTORE_DB_URL="postgres://qryvanta:qryvanta@127.0.0.1:5432/${RESTORE_DB_NAME}"

create_db "$PRIMARY_DB_NAME"
run_migrate "$PRIMARY_DB_URL"

pg_dump \
  --format=custom \
  --no-owner \
  --no-privileges \
  --file "$BACKUP_PATH" \
  "$PRIMARY_DB_URL"

# Probe mutation should disappear after restore.
psql "$PRIMARY_DB_URL" -v ON_ERROR_STOP=1 -c "CREATE TABLE IF NOT EXISTS __rollback_probe(id INTEGER PRIMARY KEY);"
psql "$PRIMARY_DB_URL" -v ON_ERROR_STOP=1 -c "INSERT INTO __rollback_probe (id) VALUES (1) ON CONFLICT (id) DO NOTHING;"

create_db "$RESTORE_DB_NAME"

pg_restore \
  --clean \
  --if-exists \
  --no-owner \
  --no-privileges \
  --dbname "$RESTORE_DB_URL" \
  "$BACKUP_PATH"

psql "$RESTORE_DB_URL" -v ON_ERROR_STOP=1 -c "SELECT COUNT(*) FROM _sqlx_migrations;"
psql "$RESTORE_DB_URL" -v ON_ERROR_STOP=1 -c "SELECT 1 FROM entity_definitions LIMIT 1;"

probe_exists="$(psql "$RESTORE_DB_URL" -tA -c "SELECT to_regclass('public.__rollback_probe') IS NOT NULL;")"
if [[ "$probe_exists" == "t" ]]; then
  echo "rollback drill failed: __rollback_probe table still exists after restore" >&2
  exit 1
fi

# Idempotency: applying migrations again must remain clean.
run_migrate "$RESTORE_DB_URL"

echo "migration drill completed: forward migration, restore rollback simulation, idempotent replay"
