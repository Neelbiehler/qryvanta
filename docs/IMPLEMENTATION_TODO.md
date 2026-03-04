# Qryvanta Implementation TODOs

Status legend:
- `[x]` done and verified in repository
- `[ ]` not done yet

This file is the execution checklist for turning Qryvanta into a production-grade metadata business platform with a secure-by-design core.

## 1) Unshakable Core

- [x] CORE-01 Publish-based metadata lifecycle exists (`draft -> checks -> publish`) in service and API layers.
- [x] CORE-02 Runtime CRUD/query flows execute against published schema snapshots.
- [x] CORE-03 Domain/Application/Infrastructure layering is enforced in repository structure.
- [x] CORE-03a Published field contracts block type/relation-target/option-set-reference mutations after publish.
- [x] CORE-04 Add explicit metadata compatibility policy enforcement (safe vs breaking transitions).
- [x] CORE-05 Add deterministic error code catalog for publish/runtime validation failures.
- [x] CORE-06 Add property/fuzz tests for domain invariants and runtime query normalization.

## 2) Long-Term Schema Stability

- [x] STAB-01 SQL migration chain is in place under `crates/infrastructure/migrations`.
- [x] STAB-02 Publish diff endpoints exist for workspace/entity compare workflows.
- [x] STAB-03 Add tested rollback playbooks for migration incidents.
- [x] STAB-04 Add migration test matrix in CI (upgrade path + rollback simulation).
- [x] STAB-05 Document and enforce zero-downtime schema transition patterns.

## 3) Extensibility Without Forking

- [x] EXT-01 Define extension runtime architecture (WASM/sandbox boundary + lifecycle).
- [x] EXT-02 Introduce stable extension API contracts and versioning rules.
- [x] EXT-03 Add capability-based permission model for extension actions.
- [x] EXT-04 Add extension isolation controls (memory/time/network/storage guards).
- [x] EXT-05 Add extension compatibility test harness across platform versions.

## 4) API Discipline

- [x] API-01 Rust-first DTO contracts generate TypeScript transport types.
- [x] API-02 Contract freshness checks are wired through `pnpm contracts:check`.
- [x] API-03 Introduce explicit API versioning policy (`/api/v1` baseline + evolution rules).
- [x] API-04 Publish deprecation policy with timelines and support window.
- [x] API-05 Add generated SDK release/versioning process for external consumers.
- [x] API-06 Evaluate GraphQL surface as optional, separately versioned contract.

## 5) Deterministic Workflow Engine

- [x] WF-01 Retry/dead-letter execution behavior exists with bounded attempts.
- [x] WF-02 Lease-token fencing exists for queued worker job completion/failure writes.
- [x] WF-03 Step traces and per-step retry/backoff controls are implemented.
- [x] WF-04 Worker heartbeat and queue stats endpoints exist for operations.
- [x] WF-05 Add deterministic replay model for full-run reconstruction.
- [x] WF-06 Add workflow idempotency conformance tests for external integrations.
- [x] WF-07 Add deterministic schedule/clock policy (timezone/clock-skew handling).

## 6) Strong Isolation

- [x] ISO-01 Tenant scoping is applied across persistence and service query paths.
- [x] ISO-02 RBAC role/grant/assignment flows are implemented.
- [x] ISO-03 Runtime field-level permissions and temporary access flows are implemented.
- [x] ISO-04 Audit log read/export/governance flows are implemented.
- [x] ISO-05 Add immutable audit storage mode (append-only ledger without destructive purge).
- [x] ISO-06 Add adversarial cross-tenant leak test suite as release gate.
- [x] ISO-07 Add optional physical isolation mode (tenant-per-db/schema deployment profile).

## 7) Observability By Design

- [x] OBS-01 Structured API logging and dependency health endpoints exist.
- [x] OBS-02 Workflow run/attempt step timeline data exists.
- [x] OBS-03 Subsystem health checks exist for Postgres/Redis readiness.
- [x] OBS-04 Add trace-id propagation across API, worker, and outbound integrations.
- [x] OBS-05 Add metrics surface (latency, throughput, queue depth, error ratios).
- [x] OBS-06 Add slow-query detection + alert thresholds.

## 8) Deterministic Builds and Supply Chain

- [x] SUP-01 Dependency lockfiles are committed (`Cargo.lock`, `pnpm-lock.yaml`).
- [x] SUP-02 CI workflow enforces frozen/locked dependency usage.
- [x] SUP-03 CI workflow runs RustSec advisory audit.
- [x] SUP-04 Add SBOM generation and artifact retention in CI/release flow.
- [x] SUP-05 Add signed release artifacts and verification documentation.

## 9) Performance Headroom

- [x] PERF-01 Stateless API composition is in place for horizontal scaling.
- [x] PERF-02 Async-first Rust runtime stack is in place.
- [x] PERF-03 Worker partitioning and queue stats cache controls are implemented.
- [x] PERF-04 Add reproducible load/performance benchmark suite.
- [x] PERF-05 Add explicit backpressure controls for heavy runtime queries/workflow bursts.
- [x] PERF-06 Publish reference scaling profiles (single node, small cluster, large cluster).

## 10) Data Exportability

- [x] DATA-01 Add full metadata export API and CLI flow.
- [x] DATA-02 Add full runtime data export API and CLI flow.
- [x] DATA-03 Add deterministic import with validation, remapping, and dry-run.
- [x] DATA-04 Define portable package format for metadata + data bundles.
- [x] DATA-05 Add portability verification tests (export -> import -> equivalence check).

## Delivery Order

- [x] STEP-00 Create this canonical implementation checklist.
- [x] STEP-01 Add baseline CI quality/security gates for deterministic builds.
- [x] STEP-02a Harden published-field compatibility guards and add regression tests.
- [x] STEP-02 Implement metadata compatibility rules and enforcement gates (CORE-04).
- [x] STEP-03 Implement migration rollback test matrix and runbooks (STAB-03, STAB-04).
- [x] STEP-04 Implement immutable-audit mode and adversarial tenant leak tests (ISO-05, ISO-06).
- [x] STEP-05 Implement trace-id propagation + metrics + slow-query signals (OBS-04..OBS-06).
- [x] STEP-06 Implement export/import portability surface (DATA-01..DATA-05).
- [x] STEP-07 Implement extension runtime foundation (EXT-01..EXT-05).
- [x] STEP-08 Implement API `v1` baseline route support and publish versioning policy docs (API-03).
- [x] STEP-09 Publish explicit API deprecation/support-window policy documentation (API-04).
- [x] STEP-10 Publish and automate generated TypeScript SDK release/versioning flow (API-05).
- [x] STEP-11 Evaluate GraphQL as an optional separately versioned API surface and publish decision record (API-06).
- [x] STEP-12 Add deterministic API error code catalog and wire publish/runtime validation mappings (CORE-05).
- [x] STEP-13 Add property-based tests for domain invariants and runtime query normalization (CORE-06).
- [x] STEP-14 Document and enforce zero-downtime migration patterns in docs and CI guard script (STAB-05).
- [x] STEP-15 Add deterministic workflow replay model and API endpoint for full-run reconstruction (WF-05).
- [x] STEP-16 Add external integration idempotency conformance tests across retries and step paths (WF-06).
- [x] STEP-17 Add deterministic schedule trigger normalization policy with timezone and clock-skew fields (WF-07).
- [x] STEP-18 Add SBOM generation in CI with retained CycloneDX/SPDX artifacts and operations docs (SUP-04).
- [x] STEP-19 Add signed release artifact generation/verification with Sigstore and operator documentation (SUP-05).
- [x] STEP-20 Add physical isolation deployment profiles with tenant-scoped worker claim filtering and operations docs (ISO-07).
- [x] STEP-21 Add reproducible k6 load/performance suite with deterministic seed flow and operations docs (PERF-04).
- [x] STEP-22 Add runtime-query and workflow-dispatch backpressure controls with configurable in-flight caps (PERF-05).
- [x] STEP-23 Publish reference scaling profiles and deployment templates for single-node/small-cluster/large-cluster (PERF-06).
- [x] STEP-24 Split runtime query request parsing/validation into focused modules (`scope`, `conditions`, `links`) to reduce handler monolith size.
- [x] STEP-25 Split API config loader into modular env/choice/isolation/validation components.
- [x] STEP-26 Split workspace app handlers into separate navigation and record handler modules.
- [x] STEP-27 Extract worker runtime configuration parsing and enum policies into dedicated worker config module.
- [x] STEP-28 Split metadata portability implementation into export/import/validation/transform modules with narrower files.
- [x] STEP-29 Run post-refactor compile/test checks for API, worker, and application crates.
- [x] STEP-30 Further split portability import flow into dedicated metadata-apply and runtime-import modules.
- [x] STEP-31 Extract worker job execution and lease-loss cancellation logic from `main.rs` into dedicated module.
