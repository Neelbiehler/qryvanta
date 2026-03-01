# API Contract Types

Qryvanta uses Rust-first API transport contracts exported to TypeScript.

## Source of Truth

- Rust DTOs: `apps/api/src/dto.rs`
- API error payload: `apps/api/src/error.rs`

DTO structs derive `TS` from `ts-rs` and export generated types to `packages/api-types/src/generated`.

## Commands

Generate contracts:

```bash
pnpm contracts:generate
```

Verify generated contracts are current:

```bash
pnpm contracts:check
```

`pnpm check` includes `contracts:check`.

## DTO Change Flow

1. Add or modify Rust DTO structs.
2. Derive `TS` and set `#[ts(export, export_to = "...")]`.
3. Run `pnpm contracts:generate`.
4. Export the generated type from `packages/api-types/src/index.ts`.
5. Update frontend imports to use `@qryvanta/api-types`.
