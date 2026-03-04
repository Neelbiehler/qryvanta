#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

PROFILE="mixed"
SKIP_SEED="false"

usage() {
  cat <<'USAGE'
Usage: ./scripts/perf/run-benchmarks.sh [options]

Options:
  --profile <workspace|runtime|workflow|mixed>   Benchmark scenario profile (default: mixed)
  --duration <value>                             k6 scenario duration (default: 90s)
  --skip-seed                                    Skip pnpm dev:seed before running benchmarks
  --help                                         Show this help output

Environment overrides:
  QRYVANTA_API_BASE_URL          API base URL (default: local k6=127.0.0.1, docker k6=host.docker.internal)
  QRYVANTA_FRONTEND_URL          Origin/Referer header for mutation requests (default: http://localhost:3000)
  QRYVANTA_BENCH_EMAIL           Login email (default: admin@qryvanta.local)
  QRYVANTA_BENCH_PASSWORD        Login password (default: admin)
  QRYVANTA_BENCH_BOOTSTRAP_TOKEN Optional auth bootstrap token (preferred for repeated benchmark runs)
  QRYVANTA_BENCH_BOOTSTRAP_SUBJECT Subject for bootstrap auth (default: perf.benchmark@qryvanta.local)
  BENCHMARK_DURATION             Scenario duration (default: 90s)
  BENCHMARK_WORKSPACE_RATE       workspace read requests/second (default: 15)
  BENCHMARK_RUNTIME_QUERY_RATE   runtime query requests/second (default: 20)
  BENCHMARK_WORKFLOW_EXEC_RATE   workflow execute requests/second (default: 5)
  BENCHMARK_PREALLOCATED_VUS     pre-allocated VUs per scenario (default: 20)
  BENCHMARK_MAX_VUS              max VUs per scenario (default: 80)
  BENCHMARK_RESULTS_DIR          output directory for JSON summaries (default: benchmarks/results)
  BENCHMARK_HEALTHCHECK_URL      probe URL checked before run (default: http://127.0.0.1:3001/health)
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="${2:-}"
      shift 2
      ;;
    --duration)
      export BENCHMARK_DURATION="${2:-}"
      shift 2
      ;;
    --skip-seed)
      SKIP_SEED="true"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

case "${PROFILE}" in
  workspace|runtime|workflow|mixed) ;;
  *)
    echo "Invalid profile '${PROFILE}'. Use workspace|runtime|workflow|mixed." >&2
    exit 1
    ;;
esac

if [[ "${SKIP_SEED}" != "true" ]]; then
  echo "Seeding deterministic dev dataset..."
  pnpm dev:seed >/dev/null
fi

HEALTHCHECK_URL="${BENCHMARK_HEALTHCHECK_URL:-http://127.0.0.1:3001/health}"
HEALTH_STATUS="$(curl --silent --show-error --output /dev/null --write-out "%{http_code}" "${HEALTHCHECK_URL}" || true)"

if [[ "${HEALTH_STATUS}" == "000" ]]; then
  echo "API probe failed at ${HEALTHCHECK_URL}" >&2
  echo "Start the API first (for example: pnpm dev:api) and retry." >&2
  exit 1
fi

if [[ "${HEALTH_STATUS}" -ge 500 ]]; then
  echo "Warning: probe endpoint returned HTTP ${HEALTH_STATUS} at ${HEALTHCHECK_URL}. Continuing benchmark run." >&2
fi

if command -v k6 >/dev/null 2>&1; then
  RUN_MODE="local"
  K6_RUNNER=(k6)
else
  if ! command -v docker >/dev/null 2>&1; then
    echo "k6 and docker are both unavailable. Install k6 or run with docker available." >&2
    exit 1
  fi
  RUN_MODE="docker"
  K6_RUNNER=(
    docker run --rm
    -v "${REPO_ROOT}:/work"
    -w /work
    --add-host=host.docker.internal:host-gateway
    grafana/k6:0.49.0
  )
fi

if [[ -z "${QRYVANTA_API_BASE_URL:-}" ]]; then
  if [[ "${RUN_MODE}" == "local" ]]; then
    export QRYVANTA_API_BASE_URL="http://127.0.0.1:3001"
  else
    export QRYVANTA_API_BASE_URL="http://host.docker.internal:3001"
  fi
fi

export BENCHMARK_PROFILE="${PROFILE}"
RESULTS_DIR="${BENCHMARK_RESULTS_DIR:-benchmarks/results}"
mkdir -p "${RESULTS_DIR}"
TIMESTAMP="$(date +"%Y%m%d-%H%M%S")"
export BENCHMARK_SUMMARY_PATH="${RESULTS_DIR}/perf-${PROFILE}-${TIMESTAMP}.json"

echo "Running PERF-04 benchmark suite"
echo "  profile: ${BENCHMARK_PROFILE}"
echo "  mode: ${RUN_MODE}"
echo "  api: ${QRYVANTA_API_BASE_URL}"
if [[ -n "${QRYVANTA_BENCH_BOOTSTRAP_TOKEN:-}" ]]; then
  echo "  auth: bootstrap"
else
  echo "  auth: login"
fi
echo "  duration: ${BENCHMARK_DURATION:-90s}"
echo "  summary: ${BENCHMARK_SUMMARY_PATH}"

"${K6_RUNNER[@]}" run benchmarks/k6/platform-load.js

echo "Benchmark run completed."
