# Qryvanta Roadmap

This roadmap turns Qryvanta from a metadata CRUD skeleton into an open-source, self-hostable alternative to closed, proprietary business platform suites.

## Product Objective

Build a metadata-driven platform where teams can define business data models, generate operational apps, automate workflows, and self-host everything.

## Enterprise Foundation Track

Status:
- Complete

Target outcome:
- Production-safe security controls established early in the platform lifecycle.

Scope:
1. Introduce tenant-scoped RBAC policy checks in application services.
2. Add append-only audit logs for metadata write operations.
3. Assign bootstrap identities to a system owner role with baseline grants.

Exit criteria:
- Metadata API operations are blocked when subject permissions are missing.
- Metadata writes create audit events with tenant and actor context.
- Tenant bootstrap creates deterministic owner grants for first membership.

## Phase 1: Foundation

Status:
- Complete

Target outcome:
- Persistent metadata and basic platform identity/security primitives.

Scope:
1. Introduce Postgres-backed persistence and migrations.
2. Add repository adapters in `crates/infrastructure` for durable metadata storage.
3. Keep application ports stable while swapping implementation.
4. Add auth bootstrap (initial admin, local JWT).
5. Add tenant model primitives for future isolation.

Exit criteria:
- Entity definitions persist across restarts.
- API remains backward-compatible for existing metadata endpoints.
- Basic auth middleware protects write endpoints.

## Phase 2: Metadata Runtime

Status:
- Complete

Target outcome:
- Rich schema model and runtime APIs generated from published metadata.

Scope:
1. Expand metadata model: field types, required flags, defaults, uniqueness, relations.
2. Add metadata lifecycle: draft -> published.
3. Introduce record APIs that operate on published entity metadata.
4. Add validation engine driven by metadata rules.

Exit criteria:
- A published entity can accept/create/list/update records.
- Validation errors are deterministic and mapped through `AppError::Validation`.

## Phase 3: App Builder

Status:
- In progress

Target outcome:
- UI composition from metadata.

Scope:
1. Build admin pages for entities, fields, forms, and views in `apps/web`.
2. Render form and list pages from API-provided metadata.
3. Add saved filters and column sets for list views.

Exit criteria:
- A user can define an entity in UI and use generated CRUD screens.

## Phase 4: Automation Runtime

Status:
- In progress

Target outcome:
- Event and schedule-driven workflows.

Scope:
1. Add workflow model (trigger + actions + conditions).
2. Implement execution runtime and worker loop.
3. Add retry policy and dead-letter handling.
4. Store execution history for observability.
5. Support queued worker execution with authenticated worker claim channel.

Exit criteria:
- Workflow triggers execute reliably with traceable run history.

## Phase 5: Security and Multi-Tenancy

Status:
- Backlog

Target outcome:
- Production-safe isolation and governance controls.

Scope:
1. Implement tenant isolation strategy at repository/query layer.
2. Add RBAC with entity/action-level permissions.
3. Add immutable audit logs for write operations and workflow runs.

Exit criteria:
- Tenant boundary tests pass.
- Unauthorized actions are blocked consistently.

## Phase 6: Self-Hosting and Operations

Status:
- Backlog

Target outcome:
- Repeatable, documented deployments.

Scope:
1. Provide Docker Compose stack for API, web, DB, cache, and workers.
2. Add env-based configuration and health/readiness probes.
3. Document backup/restore and upgrade/migration steps.

Exit criteria:
- A new user can run Qryvanta locally and in a server environment from docs.

## Immediate Backlog (Start Here)

1. ST-07: Optional remote workflow workers with secure execution boundaries (complete).
2. ST-13: Internal scaling layers for queue/cache/partition-ready runtime (active).
3. ST-11: Connected environment promotion (Dev/Staging/Prod instances).
4. ST-12: Solutions packaging (git-backed import/export + deployment pipeline).
5. ST-16: Security adversarial test pack (Claude Sec when available).
