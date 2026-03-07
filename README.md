# Qryvanta

Open-source, self-hostable business software built from metadata.

Define entities, fields, apps, and workflows. Publish them. Qryvanta turns that metadata into runtime APIs, product surfaces, and worker-driven automation without hiding the architecture behind a closed platform.

> **Active development**
>
> Qryvanta is not ready for production use yet.
>
> Contact: [contact@qryvanta.org](mailto:contact@qryvanta.org)

## Why This Project Exists

Teams that need business software usually get pushed into one of two bad options:

- buy a closed platform and accept its boundaries
- build a custom stack and own every schema, admin screen, and workflow path by hand

Qryvanta is building a third option. The code is open. The runtime is explicit. The deployment story starts with self-hosting.

## What Qryvanta Does

- Define metadata for entities and fields, then publish immutable versions.
- Generate runtime CRUD and query APIs from published metadata.
- Build tenant-scoped apps, forms, views, and workspaces on top of the same model.
- Run workflows through inline or queued workers with native triggers and typed actions.

This repository is the OSS product surface for `qryvanta.org`.

`qryvanta.com` is the planned managed-cloud surface for teams that want hosted operations later. The OSS repository stays self-hosting first.

## Why Teams Look At Qryvanta

- Open source first. You can inspect the architecture, run it yourself, and keep control of deployment.
- Rust first. The API, worker, and shared runtime are built on a Rust core aimed at long-lived business systems.
- Metadata driven. Platform behavior changes through published metadata instead of hardcoded business schemas.
- Cloud optional. The open-source path is the default here, with a managed option planned separately.

## What Exists Today

Current baseline in the repository:

- Metadata entities and fields with draft and publish lifecycle.
- Runtime record APIs generated from published metadata definitions.
- Product app paths for admin, maker, and worker flows.
- Authentication with email/password, passkeys, MFA, server-side sessions, and tenant switching.
- Tenant-scoped RBAC, audit logging, security event taxonomy, and PostgreSQL RLS coverage across major runtime tables.
- Workflow definitions with immutable published versions, native schedules, webhooks, forms, inbound email, approval triggers, and queued execution through `qryvanta-worker`.
- Shared Rust to TypeScript API contracts generated into `@qryvanta/api-types`.

For feature details and current behavior, use the docs site and roadmap:

- Product docs: [`apps/docs/content/docs`](apps/docs/content/docs)
- Repository roadmap: [`docs/ROADMAP.md`](docs/ROADMAP.md)

## Quickstart

Prerequisites: Rust stable, Node.js 22+, Docker + Docker Compose, pnpm 10+.

```bash
pnpm install
pnpm infra:up
cp .env.example .env
cargo xcheck
pnpm dev
```

Verify the API:

```bash
curl http://127.0.0.1:3001/health
```

Expected response:

```json
{"status":"ok","ready":true,"postgres":{"status":"ok","detail":null},"redis":{"status":"ok","detail":null}}
```

Local URLs:

- API: `http://localhost:3001`
- Web app: `http://localhost:3000`
- Landing site: `http://localhost:3003`
- Docs: `http://127.0.0.1:3002`

If you want a seeded development tenant:

```bash
pnpm dev:seed
```

Default seeded users:

- `admin@qryvanta.local` / `admin`
- `user@qryvanta.local` / `admin`

## Local Runtime Notes

For passkeys and session cookies in local development, keep auth URLs on `localhost`:

- `FRONTEND_URL=http://localhost:3000`
- `NEXT_PUBLIC_API_BASE_URL=http://localhost:3001`
- `WEBAUTHN_RP_ORIGIN=http://localhost:3000`
- `TOTP_ENCRYPTION_KEY=<64-char hex key>`

`WORKFLOW_EXECUTION_MODE=inline` is the default local mode.

To run queued workflow execution, start a worker:

```bash
cargo run -p qryvanta-worker
```

Optional integrations supported in this repository:

- Redis for shared sessions, rate limiting, and queue stats caching
- Qrywell-backed search sync and analytics
- SMTP for transactional email delivery
- Secret references for 1Password, AWS Secrets Manager, AWS SSM, Vault, and GCP Secret Manager

See the docs site for configuration details and self-hosting notes.

## Repository Layout

- `apps/api`: Axum API and composition root
- `apps/worker`: queued workflow worker runtime
- `apps/web`: authenticated product app
- `apps/landing`: public site for `qryvanta.org`
- `apps/docs`: Fumadocs documentation site
- `crates/core`: shared primitives and error model
- `crates/domain`: business invariants and value objects
- `crates/application`: use-cases and ports
- `crates/infrastructure`: adapters for database, queue, and external systems
- `packages/ui`: shared UI package
- `packages/api-types`: generated TypeScript transport contracts

## Daily Commands

- `pnpm dev`: run API, web, landing, and docs
- `pnpm dev:seed`: seed a realistic development tenant
- `pnpm infra:up`: start local Postgres and Redis
- `pnpm infra:down`: stop local infrastructure
- `pnpm dev:docs`: run docs only
- `pnpm dev:landing`: run landing only
- `pnpm build`: build JS workspaces
- `pnpm check`: static checks and contract checks
- `pnpm lint`: lint checks
- `pnpm test`: workspace tests
- `cargo xcheck`: Rust checks
- `cargo xclippy`: Rust lints
- `cargo xtest`: Rust tests

## Documentation

- Product and self-hosting docs: [`apps/docs/content/docs`](apps/docs/content/docs)
- Contributor workflow: [`CONTRIBUTING.md`](CONTRIBUTING.md)
- Agent and architecture guardrails: [`AGENTS.md`](AGENTS.md)
- Engineering standards: [`apps/docs/content/docs/development/engineering-standards.mdx`](apps/docs/content/docs/development/engineering-standards.mdx)

If you use local coding-agent presets:

```bash
cp -R .agent.example .agent
```

`.agent/` is machine-local and git-ignored.

## License

Apache 2.0. See [`LICENSE`](LICENSE).
