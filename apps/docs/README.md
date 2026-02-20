# Qryvanta Docs

Fumadocs-powered documentation site for Qryvanta.

## Run

From repo root:

```bash
pnpm --filter @qryvanta/docs dev
```

Or use the root shortcut:

```bash
pnpm dev:docs
```

Default URL: `http://localhost:6025`

## Structure

- `content/docs`: MDX content and `meta.json` navigation trees
- `src/lib/source.ts`: source loader and page helpers
- `src/lib/layout.shared.tsx`: shared nav/layout options
- `src/app/docs`: docs route layout and document page rendering

## Content Guidelines

- Keep docs aligned with `AGENTS.md` and `RUST_STYLE_GUIDE.md`.
- Update docs in the same PR when architecture or behavior changes.
- Prefer explicit operational steps and acceptance criteria over abstract descriptions.
