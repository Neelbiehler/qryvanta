# Qryvanta

> **Active development**
>
> Qryvanta is not ready for production use yet.
>
> Contact: [contact@qryvanta.org](mailto:contact@qryvanta.org)

Qryvanta is an open-source, self-hostable, metadata-driven business platform.

The project is built as a Rust-first monorepo with a Next.js frontend and docs site.

## Project Boundary

- `qryvanta.org` is the OSS product surface.
- `qryvanta.com` is reserved for a future managed-cloud surface.
- This repository focuses on self-hosting-first architecture and explicit runtime behavior.

## Current Baseline

- Metadata entities and fields with versioned publish lifecycle.
- Runtime CRUD/query APIs generated from published metadata definitions.
- App and workspace model for Admin, Maker, and Worker usage paths.
- Authentication with email/password, passkeys, MFA, and server-side sessions.
- Sensitive account recovery and MFA reset flows revoke all active authenticated sessions.
- High-risk tenant security-admin writes now require recent password or MFA step-up verification.
- MFA TOTP secrets can be stored with AWS KMS-backed envelope encryption instead of a single static at-rest key.
- Startup can now reject cross-environment reuse of bootstrap, session, MFA, and worker secrets through fingerprint-based drift checks.
- Tenant audit logs now carry a per-tenant tamper-evident hash chain, and admin surfaces can verify chain integrity on demand.
- Auth lifecycle events and tenant-admin audit actions now use a documented stable security event taxonomy for detections and exports.
- Tenant-scoped RBAC checks and audit/event persistence.
- PostgreSQL RLS now protects metadata definitions/components/publish/runtime, app, RBAC, extension, audit, workflow execution, and Qrywell tenant queue/event tables as defense in depth.
- Tenant membership records are RLS-protected with a narrow subject-lookup bypass for bootstrap/login flows.
- Tenant contact mappings and Qrywell analytics/search-event tables are also RLS-protected.
- Multi-membership identities now get deterministic default-tenant resolution plus authenticated tenant switching with session rotation.
- Authenticated API regression tests now exercise cross-tenant IDOR probes across entity-definition, workspace runtime, and workflow route families.
- Authenticated API regression tests now also cover `GET /auth/me` tenant visibility and `POST /auth/switch-tenant` scope changes.
- Foreign-resource delete paths now fail closed with `404` instead of succeeding as silent no-ops.
- Optional queued workflow execution via `qryvanta-worker`.
- Canonical step-graph workflow definitions shared across API, worker, and maker surfaces.
- Workflow definitions are now draft-first, published as immutable versions, and workflow runs are pinned to the published version that created them.
- Workflow access now uses dedicated `workflow.read` and `workflow.manage` RBAC grants instead of piggybacking on metadata-field permissions.
- Workspace publish checks, diff, history, and selective publish now include workflows alongside entities and apps.
- Runtime-record workflow triggers are delivered through a transactional outbox and drained by inline or queued worker control paths.
- Queued workers now include a built-in schedule-tick dispatcher with persisted slot claiming for native UTC workflow schedules.
- Workflow runtime now includes native webhook ingress at `/api/public/workflows/webhooks/{tenant_id}/{webhook_key}` for first-class webhook triggers.
- Workflow runtime now includes native form ingress at `/api/public/workflows/forms/{tenant_id}/{form_key}` for first-class form submission triggers.
- Workflow runtime now includes native inbound email ingress at `/api/public/workflows/email/{tenant_id}/{mailbox_key}` for first-class email triggers.
- Workflow runtime now includes native approval ingress at `/api/public/workflows/approvals/{tenant_id}/{approval_key}` for first-class approval-event triggers.
- Native workflow actions now include outbound integrations (`send_email`, `http_request`, `webhook`) plus platform-side operations (`update_runtime_record`, `delete_runtime_record`, `assign_owner`, `approval_request`, `delay`) with typed contracts across API, worker, and maker surfaces.
- Maker workflow authoring now uses typed field-row editors for common record/webhook/approval payloads and typed condition-value controls instead of whole-object JSON blobs in those paths.
- Workflow test execution in Maker now uses typed sample-trigger payload editors with trigger-aware defaults, and common step inspectors render local payload previews before execution.
- HTTP request and webhook steps in Maker now expose typed secret-header credential presets for common outbound auth patterns instead of raw secret-header JSON entry.
- Those outbound credential presets now include provider-aware secret-reference builders for 1Password, AWS Secrets Manager, AWS SSM, Vault, and GCP Secret Manager formats.
- HTTP request steps in Maker now support typed object, array, and scalar body authoring for common outbound payloads, with raw JSON kept only for advanced custom body shapes.
- Secret-backed outbound auth headers in Maker now support typed `Authorization` formatting (`Raw`, `Bearer`, `Basic`) plus provider-aware secret-reference builders.
- Workflow publish governance now supports secret-manager-backed outbound header references, blocks inline credential-bearing headers, requires recent step-up for publish/disable of outbound workflows, and redacts sensitive headers from persisted step traces.
- Optional Redis-backed rate limiting and workflow queue-stats caching.

## Repository Layout

- `apps/api`: Rust HTTP API (`axum`) and composition root.
- `apps/worker`: Rust workflow worker runtime for queued execution.
- `apps/web`: Next.js authenticated product app.
- `apps/landing`: Next.js public site for `qryvanta.org` messaging.
- `apps/docs`: Fumadocs documentation site.
- `crates/core`: shared primitives and error model.
- `crates/domain`: domain types and invariants.
- `crates/application`: use-cases and ports.
- `crates/infrastructure`: adapter implementations for ports.
- `packages/ui`: shared UI package.
- `packages/api-types`: generated TypeScript API contract types from Rust DTOs.
- `packages/typescript-config`: shared TypeScript config presets.

## Quickstart (First Run)

Prerequisites: Rust stable, Node.js 22+, Docker + Docker Compose, pnpm 10+.

```bash
pnpm install
pnpm infra:up
cp .env.example .env
cargo xcheck
pnpm dev
```

Verify API health:

```bash
curl http://127.0.0.1:3001/health
```

Expected response:

```json
{"status":"ok","ready":true,"postgres":{"status":"ok","detail":null},"redis":{"status":"ok","detail":null}}
```

Local URLs:

- API: `http://localhost:3001`
- Web: `http://localhost:3000`
- Landing: `http://localhost:3003`
- Docs: `http://127.0.0.1:3002`

## Auth and Local Hostnames

For passkeys and session cookies in local development, keep auth URLs on `localhost`:

- `FRONTEND_URL=http://localhost:3000`
- `NEXT_PUBLIC_API_BASE_URL=http://localhost:3001`
- `WEBAUTHN_RP_ORIGIN=http://localhost:3000`
- `TOTP_ENCRYPTION_KEY=<64-char hex key>`

Only set `TRUST_PROXY_HEADERS=true` when the API is behind a trusted reverse proxy and `TRUSTED_PROXY_CIDRS` is restricted to that ingress tier. Forwarded client IP headers from direct clients or untrusted peers are ignored.

Secret-valued startup settings can be provided directly, through `<NAME>_FILE`, or through `<NAME>_SECRET_REF` provider references for 1Password, AWS Secrets Manager/SSM, Vault, and GCP Secret Manager. See the operations configuration docs for supported formats and CLI requirements.

## Worker Runtime

`WORKFLOW_EXECUTION_MODE=inline` is the default local mode.

When using queued execution, run at least one worker process:

```bash
cargo run -p qryvanta-worker
```

For partitioned scale-out, set `WORKER_PARTITION_COUNT` and `WORKER_PARTITION_INDEX` together (for example, `count=4` and indexes `0..3` across worker groups).

Use `WORKER_MAX_CONCURRENCY` to process claimed jobs in parallel per worker loop.

For distributed worker coordination, set `WORKER_COORDINATION_BACKEND=redis` and tune `WORKER_COORDINATION_LEASE_SECONDS` (optional `WORKER_COORDINATION_SCOPE_KEY` override). Active worker cycles auto-renew coordination leases during execution.

Queued worker claims include opaque lease tokens; queue completion/failure writes are fenced by those tokens to reduce stale-worker split-brain effects.

Set `WORKER_LEASE_LOSS_STRATEGY=graceful_drain` to stop new work and cancel mutating in-flight tasks while allowing non-mutating jobs to finish after lease loss (`abort_all` cancels everything immediately).

For high-frequency ops polling, set `WORKFLOW_QUEUE_STATS_CACHE_TTL_SECONDS` to a small value (for example `2`-`5`) to enable API-side in-memory queue stats caching.

## Redis Runtime (Optional)

- `REDIS_URL` enables shared Redis integrations.
- Set `RATE_LIMIT_STORE=redis` to move auth/API throttling state out of Postgres.
- Set `WORKFLOW_QUEUE_STATS_CACHE_BACKEND=redis` to share queue stats cache across API replicas.
- Set `SESSION_STORE=redis` to move session storage out of Postgres.

## Qrywell Search Integration (Optional)

- Set `QRYWELL_API_BASE_URL` to enable Qrywell-backed search proxy from Qryvanta API.
- Optional `QRYWELL_API_KEY` is forwarded to Qrywell as `x-qrywell-api-key`.
- Call `POST /api/search/qrywell` from authenticated product surfaces to retrieve tenant-scoped search hits.
- Runtime record create/update/delete now queue durable Qrywell sync jobs with retry/backoff processing.
- Use `POST /api/search/qrywell/sync/{entity_logical_name}` for manual backfill of existing records.
- Use `POST /api/search/qrywell/sync-all` for full-tenant backfill across all entities.
- Use `GET /api/search/qrywell/queue-health` to monitor pending/processing/failed sync jobs.
- Use `POST /api/search/qrywell/events/click` to collect result interaction analytics for relevance tuning.
- Use `GET /api/search/qrywell/analytics` for query quality signals (top queries, rank click share, zero-click, low-relevance clicks).
- Query forwarding uses tenant metadata to derive schema-aware facet filters (entity + option-set values) without hardcoded business field names.
- Tune sync worker behavior using `QRYWELL_SYNC_POLL_INTERVAL_MS`, `QRYWELL_SYNC_BATCH_SIZE`, and `QRYWELL_SYNC_MAX_ATTEMPTS`.

## Transactional Email

- Local default: `EMAIL_PROVIDER=console` (email content goes to API logs).
- SMTP mode: set `EMAIL_PROVIDER=smtp` and provide `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_FROM_ADDRESS`.
- Qryvanta email scope is transactional only: verification, reset, and invite flows.

## Daily Commands

- `pnpm dev`: run API, web, landing, and docs.
- `pnpm dev:seed`: seed a realistic CRM/ERP development tenant dataset (includes users `admin@qryvanta.local`/`admin` and `user@qryvanta.local`/`admin`, roles, apps, forms, views, workflows, and sitemaps).
- `pnpm infra:up`: start local Postgres + Redis.
- `pnpm infra:down`: stop local infrastructure.
- `pnpm dev:docs`: run docs app only.
- `pnpm dev:landing`: run landing app only.
- `pnpm build`: build JS workspaces.
- `pnpm check`: static checks and contract checks.
- `pnpm api-types:build`: build publishable `@qryvanta/api-types` SDK artifacts.
- `pnpm lint`: lint checks.
- `pnpm test`: workspace tests.
- `pnpm perf:benchmark`: run reproducible PERF-04 k6 benchmark suite (`-- --profile runtime` for single-profile run).
- `cargo xcheck`: Rust checks.
- `cargo xclippy`: Rust lints.
- `cargo xtest`: Rust tests.
- `just perf-benchmark mixed 120s`: run the same benchmark suite via `just` with explicit profile/duration.
- `just portability-export <tenant_id> <output_path>`: export tenant portability bundle.
- `just portability-import <tenant_id> <input_path>`: import tenant portability bundle.

## Documentation and Standards

- Docs site content: `apps/docs/content/docs`
- Architecture and workflow guardrails: `AGENTS.md`
- Contributor workflow: `CONTRIBUTING.md`
- Engineering standards: `apps/docs/content/docs/development/engineering-standards.mdx`

If you use local coding-agent presets:

```bash
cp -R .agent.example .agent
```

`.agent/` is machine-local and git-ignored.

## Roadmap

- Product roadmap document: `docs/ROADMAP.md`
- Docs roadmap page: `apps/docs/content/docs/development/roadmap.mdx`

## License

Apache 2.0. See `LICENSE`.
