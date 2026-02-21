# Qryvanta

> **⚠️ Active Development**
>
> Qryvanta is currently in active development and is **not yet ready for production use**.
>
> Interested in the project? Contact us at [contact@qryvanta.org](mailto:contact@qryvanta.org).

Qryvanta is an open-source, self-hostable, metadata-driven business platform.

Our goal is to build a practical alternative to traditional enterprise business suites with a transparent architecture and portable deployments.

## Why Qryvanta

- Rust-first backend architecture with clear layering and testable boundaries.
- Metadata-defined entities and runtime behavior as the long-term platform model.
- Self-hosting first: local development and deployment paths stay explicit.
- Monorepo workflow with API, web, and docs developed together.

## Monorepo Layout

- `apps/api`: Rust HTTP API binary (Axum)
- `apps/web`: Next.js frontend
- `apps/docs`: Fumadocs documentation website
- `crates/core`: shared primitives and error model
- `crates/domain`: business domain types and validation
- `crates/application`: use-cases and ports
- `crates/infrastructure`: adapter implementations for ports
- `packages/ui`: shared UI component library based on shadcn patterns
- `packages/api-types`: generated TypeScript API contracts from Rust DTOs
- `packages/typescript-config`: shared TypeScript base config

## Quickstart

1. Install prerequisites: Rust stable, Node.js 22+, Docker + Docker Compose, pnpm 10+.
2. Install dependencies: `pnpm install`.
3. Start infrastructure: `docker compose up -d`.
4. Create local env: `cp .env.example .env`.
5. Run checks: `cargo xcheck`.
6. Start development: `pnpm dev`.

Default local ports:

- API: `http://localhost:3001`
- Web: `http://localhost:3000`
- Docs: `http://127.0.0.1:3002`

Keep auth-related URLs on `localhost` during local development to avoid passkey and session-cookie origin mismatches.

## Transactional Email

- Local development uses `EMAIL_PROVIDER=console` (logs auth/invite emails to API output).
- Production should use `EMAIL_PROVIDER=smtp` with non-empty `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD`, and `SMTP_FROM_ADDRESS`.
- Qryvanta mail flows are transactional-only (verification, reset, invite) and do not include marketing tracking.

## Security Administration

- Tenant role admins can manage custom RBAC roles and assignments from the web UI.
- Tenant role admins can switch workspace registration mode between `invite_only` and `open`.
- Authentication bootstrap flows automatically ensure each subject is represented by a tenant-scoped runtime `contact` record.

## Useful Commands

- `pnpm dev` - run API, web, and docs
- `pnpm build` - build all workspaces
- `pnpm lint` - run lint checks
- `pnpm format:web` - format frontend files with Prettier
- `pnpm format:web:check` - verify frontend formatting
- `pnpm test` - run all tests
- `pnpm check` - run static checks + API contract checks
- `cargo xcheck` - Rust checks
- `cargo xclippy` - Rust lints
- `cargo xtest` - Rust tests

## Contributing

Start with:

- `CONTRIBUTING.md` for contributor workflow
- `AGENTS.md` for architecture boundaries and coding guardrails
- `apps/docs/content/docs/development/engineering-standards.mdx` for development standards

If you use local coding-agent presets, bootstrap them with:

```bash
cp -R .agent.example .agent
```

`.agent/` is intentionally ignored and machine-specific.

## Roadmap

- Product roadmap: `docs/ROADMAP.md`
- Documentation roadmap page: `apps/docs/content/docs/development/roadmap.mdx`

## Domains

- Main open-source project site: `qryvanta.org`
- Reserved for future cloud offering: `qryvanta.com`

## License

Licensed under Apache 2.0. See `LICENSE`.
