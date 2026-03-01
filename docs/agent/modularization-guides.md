# Modularization Guides

This page captures structural refactor rules for backend and frontend modules.

## Backend Modularization

Goals:

- Split large Rust modules by responsibility without changing behavior.
- Keep layering intact (`domain` -> `application` -> `infrastructure` -> `apps/api`).
- Keep API contracts and route shapes stable during structural refactors.

Patterns:

- Use coarse concern-based modules over many tiny files.
- Extract shared types into dedicated `*_ports.rs` or shared modules.
- Move large test blocks into sibling `tests.rs` modules.

Quality gate:

```bash
cargo fmt --all
cargo xcheck
cargo xclippy
cargo xtest
```

## Frontend Modularization

Goals:

- Keep each file focused on one concern.
- Reuse shared primitives from `@qryvanta/ui`.
- Keep behavior stable while refactoring structure.

Feature split pattern:

1. `model.ts` or `helpers.ts` for pure logic and shared types.
2. One orchestrator component for state and API calls.
3. Focused section components for rendering concerns.
4. Shared primitives in `packages/ui` when used across apps.

Frontend gate:

```bash
pnpm --filter @qryvanta/web check
pnpm --filter @qryvanta/web lint
pnpm --filter @qryvanta/web build
pnpm --filter @qryvanta/ui check
```

For substantial UI refactors, run React Doctor:

```bash
npx -y react-doctor@latest . --verbose --project @qryvanta/web
npx -y react-doctor@latest ./packages/ui --verbose
```
