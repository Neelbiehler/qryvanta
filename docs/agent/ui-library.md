# UI Library (`@qryvanta/ui`)

Qryvanta ships reusable web primitives from `packages/ui` as `@qryvanta/ui`.

`@qryvanta/ui` is publishable so external repos, including `qryvanta.com`, can consume the same component package.

## Usage

Import server-safe components from package root. Import client-only components from explicit subpaths.

Example client-only entry points:

- `@qryvanta/ui/dropdown-menu`
- `@qryvanta/ui/dialog`
- `@qryvanta/ui/toast`
- `@qryvanta/ui/split-pane`
- `@qryvanta/ui/tree-view`
- `@qryvanta/ui/data-grid`

## Component Extraction Rule

Move a component into `@qryvanta/ui` when it is:

- Generic and not domain-specific.
- Used by at least two frontend surfaces.
- Stable enough to maintain as package API.

## Release Flow

1. Update `packages/ui`.
2. Run checks and build.
3. Create a changeset.
4. Merge to `main`.

```bash
pnpm --filter @qryvanta/ui check
pnpm --filter @qryvanta/ui build
pnpm changeset
```

Publishing is automated by `.github/workflows/release-packages.yml`.
