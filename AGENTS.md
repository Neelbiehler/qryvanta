# AGENTS.md

Implementation guide for coding agents working on Qryvanta.
Read `RUST_STYLE_GUIDE.md` before writing Rust code.
Use `.agent` for optional local agent customization.
If `.agent` is missing, copy from `.agent.example` before frontend work.

## What Qryvanta Is

Qryvanta is an open-source, self-hostable, metadata-driven business platform.
The long-term goal is to provide a practical alternative to closed, proprietary business platform suites with a transparent architecture and portable deployments.

Current baseline:
- Rust-first monorepo with clear layering
- Axum API (`apps/api`)
- Next.js frontend (`apps/web`)
- Turborepo orchestration

Target platform capabilities:
- Metadata-defined entities, fields, and relationships
- Runtime record APIs generated from published metadata
- App builder UX (forms, views, dashboards)
- Workflow and automation runtime
- Security, RBAC, and multi-tenant isolation
- Self-hosting-first operations and packaging

## Architecture Boundaries

Follow these rules strictly:

1. `crates/domain` holds business invariants and value objects only.
2. `crates/application` defines use-cases and ports; no framework coupling.
3. `crates/infrastructure` implements ports (DB, queues, external systems).
4. `apps/api` composes dependencies and translates HTTP <-> application.
5. `apps/web` consumes API contracts and stays independently deployable.

Do not move domain logic into API handlers or infrastructure adapters.

## Shared API Types (Rust -> TypeScript)

Qryvanta uses Rust-first API contracts.

1. API request/response DTOs are defined in Rust in `apps/api/src`.
2. TypeScript contract types are generated into `packages/api-types/src/generated`.
3. Frontend code must import API transport types from `@qryvanta/api-types` instead of redefining DTO shapes locally.
4. When changing DTOs, regenerate contracts in the same change set and ensure contract checks pass.

Commands:

- `pnpm contracts:generate` — regenerate TypeScript types from Rust DTOs.
- `pnpm contracts:check` — fail if generated types are stale.

## Shared Package Publishing (`@qryvanta/ui`)

Qryvanta publishes the shared UI library from `packages/ui` so external repositories (including `qryvanta.com`) can consume one canonical component package.

Rules:

1. Treat `packages/ui` as a public package surface with stable exports.
2. Any change affecting `@qryvanta/ui` behavior or API must include a changeset.
3. Keep cloud-only product flows out of `@qryvanta/ui`; compose those in consumer apps.
4. Do not publish manually from ad-hoc local state when the release workflow can be used.

Commands:

- `pnpm --filter @qryvanta/ui check` — validate TypeScript API surface.
- `pnpm --filter @qryvanta/ui build` — build publishable `dist` artifacts.
- `pnpm changeset` — create package release notes and bump intent.
- `pnpm changeset:version` — apply version updates from pending changesets.
- `pnpm changeset:publish` — publish pending package releases.

Automation:

- `.github/workflows/release-packages.yml` runs Changesets release automation on `main`.
- The workflow requires `NPM_TOKEN` repository secret for npm publish access.

## Documentation-First Requirement

Documentation is a required deliverable for every feature.

1. Every new feature, behavior change, API contract change, config key, migration, or operational change must be documented in `apps/docs/content/docs` when it affects users or self-hosting operators.
2. Maintainer-only guidance (coding-agent workflows, contribution process, refactor patterns) must live in repository docs under `docs/`.
3. Product docs must be structured under the correct section (`quickstart`, `workspace`, `concepts`, `operations`) and included in navigation via `meta.json` when adding a new page.
4. Pull requests that change behavior without documentation updates are incomplete unless the change is purely internal refactoring with no external impact.
5. Agent implementations must update docs in the same change set, not as a later follow-up.

## Monorepo Map

```
apps/
├── api                 — Rust HTTP API binary
├── web                 — Next.js frontend
└── docs                — Fumadocs documentation website

crates/
├── core                — shared primitives and error model
├── domain              — business domain types and validation
├── application         — use-cases and ports
└── infrastructure      — adapter implementations

packages/
├── ui                  — shared ui library based on shadcn
└── typescript-config   — shared TypeScript config

```

## Delivery Phases

Phase 1 (foundation):
1. Introduce persistent storage and migrations.
2. Replace in-memory metadata repository with DB-backed implementation.
3. Add auth bootstrap and tenancy primitives.
4. Keep API/web running locally with one command.

Phase 2 (metadata runtime):
1. Expand metadata model (field types, constraints, relations).
2. Add draft/published metadata lifecycle.
3. Serve runtime record CRUD/query APIs based on published metadata.

Phase 3 (app builder):
1. Build entity/field/view/form admin UI.
2. Render list and form UIs from metadata.
3. Add saved views and query presets.

Phase 4 (automation):
1. Implement workflow triggers and actions.
2. Add execution history, retries, and failure handling.
3. Support scheduled and event-driven automations.

Phase 5 (security and operations):
1. RBAC across tenant, entity, and action levels.
2. Full audit trail for write operations.
3. Containerized self-hosting setup and backup/restore docs.

## Anti-Patterns

- Do not bypass application ports from API handlers.
- Do not hardcode entity schemas in runtime paths.
- Do not leak tenant scope in repository queries.
- Do not silently ignore `Result` values.
- Do not use `unwrap`/`expect` in production code.
- Do not introduce `unsafe`.

## Definition of Done

A feature is done only when:

1. Domain and application logic are implemented in the correct layer.
2. API and web integration is complete for the feature path.
3. Tests exist for domain invariants and application behavior.
4. Lint, check, and test commands pass.
5. Docs are updated (`README.md` and relevant pages in `apps/docs/content/docs`).

## Commands

- `pnpm dev` — run API, web, and docs
- `pnpm dev:docs` — run docs app only
- `pnpm test` — run all tests
- `pnpm lint` — run all linters
- `cargo fmt --all` — format Rust
- `cargo xcheck` — Rust checks
- `cargo xclippy` — Rust lints
- `cargo xtest` — Rust tests
