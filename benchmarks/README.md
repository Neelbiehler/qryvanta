# Qryvanta Benchmarks

This directory contains reproducible performance/load suites for core platform paths.

## PERF-04 Suite

- k6 scenario file: `benchmarks/k6/platform-load.js`
- runner script: `scripts/perf/run-benchmarks.sh`

Run:

```bash
pnpm perf:benchmark
```

Run one profile:

```bash
pnpm perf:benchmark -- --profile runtime --duration 120s
```

For repeated local runs under strict login throttling, optionally set `QRYVANTA_BENCH_BOOTSTRAP_TOKEN` and `QRYVANTA_BENCH_BOOTSTRAP_SUBJECT` to use bootstrap auth.

Results are written to `benchmarks/results/` (git-ignored).
