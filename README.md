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
- Tenant-scoped RBAC checks and audit/event persistence.
- Optional queued workflow execution via `qryvanta-worker`.
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

## Transactional Email

- Local default: `EMAIL_PROVIDER=console` (email content goes to API logs).
- SMTP mode: set `EMAIL_PROVIDER=smtp` and provide `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, `SMTP_FROM_ADDRESS`.
- Qryvanta email scope is transactional only: verification, reset, and invite flows.

## Daily Commands

- `pnpm dev`: run API, web, landing, and docs.
- `pnpm infra:up`: start local Postgres + Redis.
- `pnpm infra:down`: stop local infrastructure.
- `pnpm dev:docs`: run docs app only.
- `pnpm dev:landing`: run landing app only.
- `pnpm build`: build JS workspaces.
- `pnpm check`: static checks and contract checks.
- `pnpm lint`: lint checks.
- `pnpm test`: workspace tests.
- `cargo xcheck`: Rust checks.
- `cargo xclippy`: Rust lints.
- `cargo xtest`: Rust tests.

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
