# @qryvanta/api-types

Generated TypeScript SDK contract types for the Qryvanta HTTP API.

## Install

```bash
pnpm add @qryvanta/api-types
```

## Usage

Import transport request/response types from this package instead of redefining DTO shapes in client code.

```ts
import type {
  CreateEntityRequest,
  EntityResponse,
  ErrorResponse,
} from "@qryvanta/api-types";
```

## Versioning

- Package versions are published with Changesets.
- Additive DTO changes ship as minor/patch updates.
- Breaking transport contract changes ship under a new API major (`/api/v2`) and a corresponding SDK major.

## Local Development

Regenerate contract files from Rust DTOs:

```bash
pnpm contracts:generate
```

Validate generated files are committed:

```bash
pnpm contracts:check
```

Build publishable artifacts:

```bash
pnpm --filter @qryvanta/api-types build
```
