#!/usr/bin/env bash
set -euo pipefail

API_URL="${API_URL:-${1:-}}"
WEB_URL="${WEB_URL:-${2:-}}"
EXPECT_HTTP_REDIRECT="${EXPECT_HTTP_REDIRECT:-true}"

if [[ -z "$API_URL" || -z "$WEB_URL" ]]; then
  echo "usage: API_URL=https://api.example.com WEB_URL=https://app.example.com $0" >&2
  echo "or: $0 https://api.example.com https://app.example.com" >&2
  exit 1
fi

normalize_base_url() {
  printf '%s' "${1%/}"
}

require_https_url() {
  local url="$1"
  local name="$2"
  if [[ ! "$url" =~ ^https:// ]]; then
    echo "$name must use https:// for ingress conformance checks: $url" >&2
    exit 1
  fi
}

lower() {
  printf '%s' "$1" | tr '[:upper:]' '[:lower:]'
}

header_value() {
  local headers_file="$1"
  local name
  name="$(lower "$2")"
  awk -v target="$name" '
    BEGIN { IGNORECASE = 1 }
    {
      line = $0
      sub(/\r$/, "", line)
      split(line, parts, ":")
      key = tolower(parts[1])
      if (key == target) {
        value = substr(line, index(line, ":") + 1)
        sub(/^[[:space:]]+/, "", value)
        print value
        exit
      }
    }
  ' "$headers_file"
}

assert_status() {
  local expected="$1"
  local actual="$2"
  local context="$3"
  if [[ "$actual" != "$expected" ]]; then
    echo "$context returned unexpected status: expected $expected, got $actual" >&2
    exit 1
  fi
}

assert_one_of_statuses() {
  local actual="$1"
  local context="$2"
  shift 2

  local expected
  for expected in "$@"; do
    if [[ "$actual" == "$expected" ]]; then
      return 0
    fi
  done

  echo "$context returned unexpected status: got $actual, expected one of: $*" >&2
  exit 1
}

assert_header_equals() {
  local headers_file="$1"
  local header_name="$2"
  local expected="$3"
  local actual
  actual="$(header_value "$headers_file" "$header_name")"
  if [[ "$actual" != "$expected" ]]; then
    echo "header $header_name mismatch: expected '$expected', got '${actual:-<missing>}'" >&2
    exit 1
  fi
}

assert_header_contains() {
  local headers_file="$1"
  local header_name="$2"
  local expected_substring="$3"
  local actual
  actual="$(header_value "$headers_file" "$header_name")"
  if [[ -z "$actual" ]]; then
    echo "required header missing: $header_name" >&2
    exit 1
  fi
  if [[ "$actual" != *"$expected_substring"* ]]; then
    echo "header $header_name missing expected fragment '$expected_substring': '$actual'" >&2
    exit 1
  fi
}

check_http_redirect() {
  local https_url="$1"
  local name="$2"
  local http_url
  http_url="http://${https_url#https://}"
  local headers_file
  headers_file="$(mktemp)"

  local status
  status="$(
    curl -sS -o /dev/null -D "$headers_file" \
      -w '%{http_code}' \
      "$http_url"
  )"
  assert_one_of_statuses "$status" "$name plaintext redirect" 301 302 307 308
  assert_header_contains "$headers_file" "location" "$https_url"
  rm -f "$headers_file"
}

check_api_health_headers() {
  local api_base_url="$1"
  local headers_file
  headers_file="$(mktemp)"
  local trace_id="ingress-conformance-$(date +%s)"
  local status
  status="$(
    curl -sS -o /dev/null -D "$headers_file" \
      -H "x-trace-id: $trace_id" \
      -w '%{http_code}' \
      "$(normalize_base_url "$api_base_url")/health"
  )"

  assert_status 200 "$status" "api health"
  assert_header_equals "$headers_file" "x-trace-id" "$trace_id"
  assert_header_equals "$headers_file" "x-content-type-options" "nosniff"
  assert_header_equals "$headers_file" "x-frame-options" "DENY"
  assert_header_equals "$headers_file" "referrer-policy" "strict-origin-when-cross-origin"
  assert_header_contains "$headers_file" "content-security-policy" "frame-ancestors 'none'"
  assert_header_contains "$headers_file" "permissions-policy" "camera=()"
  assert_header_contains "$headers_file" "permissions-policy" "microphone=()"
  rm -f "$headers_file"
}

check_api_cors_preflight() {
  local api_base_url="$1"
  local web_base_url="$2"
  local headers_file
  headers_file="$(mktemp)"
  local status
  status="$(
    curl -sS -o /dev/null -D "$headers_file" \
      -X OPTIONS \
      -H "Origin: $web_base_url" \
      -H "Access-Control-Request-Method: POST" \
      -H "Access-Control-Request-Headers: content-type" \
      -w '%{http_code}' \
      "$(normalize_base_url "$api_base_url")/auth/login"
  )"

  assert_one_of_statuses "$status" "api cors preflight" 200 204
  assert_header_equals "$headers_file" "access-control-allow-origin" "$web_base_url"
  assert_header_equals "$headers_file" "access-control-allow-credentials" "true"
  assert_header_contains "$headers_file" "access-control-allow-methods" "POST"
  assert_header_contains "$headers_file" "access-control-allow-headers" "content-type"
  rm -f "$headers_file"
}

check_web_headers() {
  local web_base_url="$1"
  local headers_file
  headers_file="$(mktemp)"
  local status
  status="$(
    curl -sS -o /dev/null -D "$headers_file" \
      -w '%{http_code}' \
      "$(normalize_base_url "$web_base_url")/"
  )"

  assert_status 200 "$status" "web root"
  assert_header_equals "$headers_file" "x-content-type-options" "nosniff"
  assert_header_equals "$headers_file" "x-frame-options" "DENY"
  assert_header_equals "$headers_file" "referrer-policy" "strict-origin-when-cross-origin"
  assert_header_contains "$headers_file" "content-security-policy" "frame-ancestors 'none'"
  assert_header_contains "$headers_file" "permissions-policy" "camera=()"
  rm -f "$headers_file"
}

API_URL="$(normalize_base_url "$API_URL")"
WEB_URL="$(normalize_base_url "$WEB_URL")"

require_https_url "$API_URL" "API_URL"
require_https_url "$WEB_URL" "WEB_URL"

if [[ "$EXPECT_HTTP_REDIRECT" == "true" ]]; then
  check_http_redirect "$API_URL" "api"
  check_http_redirect "$WEB_URL" "web"
fi

check_api_health_headers "$API_URL"
check_api_cors_preflight "$API_URL" "$WEB_URL"
check_web_headers "$WEB_URL"

echo "ingress conformance passed for API_URL=$API_URL WEB_URL=$WEB_URL"
