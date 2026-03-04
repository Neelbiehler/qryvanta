#!/usr/bin/env bash
set -euo pipefail

MIGRATIONS_DIR="${MIGRATIONS_DIR:-crates/infrastructure/migrations}"
# Guard starts at migration 0033 onward to avoid rewriting historical migrations
# that are already applied in existing environments.
ZERO_DOWNTIME_BASELINE_SEQ="${ZERO_DOWNTIME_BASELINE_SEQ:-32}"

if [[ ! -d "$MIGRATIONS_DIR" ]]; then
  echo "migration guard failed: directory '$MIGRATIONS_DIR' does not exist" >&2
  exit 1
fi

violations=0
checked_files=0

check_line() {
  local file="$1"
  local line_no="$2"
  local line="$3"

  local upper_line
  upper_line="$(printf '%s' "$line" | tr '[:lower:]' '[:upper:]')"

  # Allow explicit documented exceptions on the same line.
  if [[ "$upper_line" == *"QV:ALLOW-NON-ZDT"* ]]; then
    return 0
  fi

  if [[ "$upper_line" =~ (^|[[:space:]])DROP[[:space:]]+TABLE([[:space:]]|$) ]]; then
    echo "zdt guard: ${file}:${line_no}: DROP TABLE is not allowed in forward migrations"
    violations=$((violations + 1))
  fi

  if [[ "$upper_line" =~ (^|[[:space:]])DROP[[:space:]]+COLUMN([[:space:]]|$) ]]; then
    echo "zdt guard: ${file}:${line_no}: DROP COLUMN is not allowed in forward migrations"
    violations=$((violations + 1))
  fi

  if [[ "$upper_line" =~ (^|[[:space:]])RENAME[[:space:]]+COLUMN([[:space:]]|$) ]]; then
    echo "zdt guard: ${file}:${line_no}: RENAME COLUMN is not allowed in forward migrations"
    violations=$((violations + 1))
  fi

  if [[ "$upper_line" =~ ALTER[[:space:]]+TABLE.*ALTER[[:space:]]+COLUMN.*SET[[:space:]]+NOT[[:space:]]+NULL ]]; then
    echo "zdt guard: ${file}:${line_no}: SET NOT NULL is blocked; use expand/backfill/contract pattern"
    violations=$((violations + 1))
  fi

  if [[ "$upper_line" =~ ADD[[:space:]]+COLUMN.*NOT[[:space:]]+NULL ]] && [[ ! "$upper_line" =~ DEFAULT ]]; then
    echo "zdt guard: ${file}:${line_no}: ADD COLUMN ... NOT NULL requires DEFAULT or staged rollout"
    violations=$((violations + 1))
  fi
}

while IFS= read -r file; do
  basename_file="$(basename "$file")"
  if [[ ! "$basename_file" =~ ^([0-9]{4})_.*\.sql$ ]]; then
    continue
  fi

  seq_num="${BASH_REMATCH[1]}"
  seq_num_decimal=$((10#$seq_num))
  if (( seq_num_decimal <= ZERO_DOWNTIME_BASELINE_SEQ )); then
    continue
  fi

  checked_files=$((checked_files + 1))

  line_no=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    line_no=$((line_no + 1))
    # Skip full-line SQL comments for noise reduction.
    if [[ "$line" =~ ^[[:space:]]*-- ]]; then
      continue
    fi
    check_line "$file" "$line_no" "$line"
  done < "$file"
done < <(find "$MIGRATIONS_DIR" -maxdepth 1 -type f -name "*.sql" | sort)

if (( checked_files == 0 )); then
  echo "zdt guard: no migrations above baseline ${ZERO_DOWNTIME_BASELINE_SEQ} to validate"
fi

if (( violations > 0 )); then
  echo "zdt guard failed with ${violations} violation(s)" >&2
  exit 1
fi

echo "zdt guard passed"
