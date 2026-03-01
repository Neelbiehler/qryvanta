# Contribution Workflow

## Required Reading

- `AGENTS.md`
- `RUST_STYLE_GUIDE.md`
- `CONTRIBUTING.md`

## Local Setup

```bash
pnpm install
pnpm infra:up
cp .env.example .env
cargo xcheck
```

Optional local coding-agent presets:

```bash
cp -R .agent.example .agent
```

`.agent/` is gitignored and intended for machine-local configuration.

## Pull Request Expectations

- Keep pull requests focused and explain why the change exists.
- Add or update tests for behavior changes.
- Keep domain logic in `crates/domain` and use-cases in `crates/application`.
- Update docs in the same PR for externally meaningful changes.

## Validation Gate

```bash
cargo fmt --all
cargo xcheck
cargo xclippy
cargo xtest
pnpm check
pnpm lint
pnpm test
```

## Frontend Quality Gate

Run React Doctor for significant frontend or UI package changes:

```bash
npx -y react-doctor@latest . --verbose --project @qryvanta/web
npx -y react-doctor@latest ./packages/ui --verbose
```

Resolve high-signal findings in the same change set unless explicitly deferred.

## Shared Package Releases

`@qryvanta/ui` is published from `packages/ui`.

```bash
pnpm --filter @qryvanta/ui check
pnpm --filter @qryvanta/ui build
pnpm changeset
```

Release automation runs in `.github/workflows/release-packages.yml` and requires `NPM_TOKEN`.
