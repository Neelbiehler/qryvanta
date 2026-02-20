# Contributing

Thanks for contributing to Qryvanta. This repository is built for open collaboration, with strict architecture boundaries and documentation-first delivery.

## Prerequisites

1. Rust stable (`rustup`)
2. Node.js 22+
3. pnpm 10+
4. Docker + Docker Compose
5. pre-commit (optional but recommended)

## Required Reading

- `AGENTS.md`
- `RUST_STYLE_GUIDE.md`
- `apps/docs/content/docs/development/engineering-standards.mdx`

## Setup

```bash
pnpm install
docker compose up -d
cp .env.example .env
cargo xcheck
```

If you use local coding-agent presets for frontend work:

```bash
cp -R .agent.example .agent
```

`.agent/` is intentionally ignored so each contributor can keep local customizations.

## Standard Commands

```bash
pnpm dev      # run API + web via turbo
pnpm build    # build all workspaces
pnpm check    # static checks
pnpm lint     # lint checks
pnpm test     # tests
```

## Branches and Pull Requests

- Use short-lived topic branches from `main`.
- Keep pull requests focused and small when possible.
- Include tests for behavior changes.
- Update docs in the same PR for any externally meaningful change.
- Regenerate API contract types when Rust DTOs change.

Recommended branch naming examples:

- `feat/metadata-publish-lifecycle`
- `fix/web-auth-redirect`
- `docs/open-source-contributing`

## Rust Quality Gate

Before opening a PR:

```bash
cargo fmt --all
cargo xcheck
cargo xclippy
cargo xtest
```

## API Contract Types

When API DTOs change, run:

```bash
pnpm contracts:generate
pnpm contracts:check
```

## Documentation Policy

Documentation updates are mandatory for new features, behavior changes, API contract updates, configuration changes, and operational changes.

Primary docs location:

- `apps/docs/content/docs`

Repository-level updates when relevant:

- `README.md`
- `AGENTS.md`
