# Rust Style Guide

Conventions for Qryvanta Rust code.
When in doubt, consistency with existing code wins over personal preference.

## Project Structure

Qryvanta is a workspace, not a single crate.

- `apps/api`: Axum composition root and HTTP handlers
- `crates/core`: shared primitives and error model
- `crates/domain`: business entities and invariants
- `crates/application`: use-cases and repository/service ports
- `crates/infrastructure`: port implementations

Each crate should keep `src/lib.rs` (or `src/main.rs`) as the module root.
Prefer expanding existing modules before creating many tiny files.

## Layering Rules

1. `domain` depends only on `core` and std/external utility crates.
2. `application` depends on `domain` and `core`, never API frameworks.
3. `infrastructure` depends on `application` ports and external systems.
4. `apps/api` owns protocol translation and dependency wiring only.

If logic can be tested without HTTP, it belongs in `domain` or `application`.

## Lints and Quality Gates

Follow workspace lint policy in `Cargo.toml`.

- `unsafe_code = "forbid"`
- `unwrap_used = "deny"`
- `expect_used = "deny"`

Do not ship `dbg!`, `todo!`, or `unimplemented!`.
Use comments for tracked future work:

```rust
// TODO: support metadata publish rollback in transaction boundary.
```

## Imports

Group imports in tiers, alphabetized inside each tier:

1. crate-local (`crate::...`)
2. workspace crates (`qryvanta_*`)
3. external crates
4. standard library

Example:

```rust
use crate::dto::{CreateEntityRequest, EntityResponse};

use qryvanta_application::MetadataService;
use qryvanta_core::{AppError, AppResult};
use qryvanta_domain::EntityDefinition;

use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use std::sync::Arc;
```

## Naming

- Variables/functions: `snake_case`
- Types/traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- No unclear abbreviations (`message`, not `msg`; `repository`, not `repo` in domain code)

Booleans should read naturally:
- `is_active`
- `has_conflict`
- `can_publish`

## Struct and API Design

- Keep fields private by default.
- Expose behavior through methods, not mutable public fields.
- Derive traits only as needed.
- Put constructor validation close to the type (`EntityDefinition::new` pattern).

For public APIs, document invariants with `///` comments.

## Error Handling

Use `qryvanta_core::AppError` and `AppResult<T>` across crates.

- `Validation` for invalid input/invariants
- `NotFound` for missing resources
- `Conflict` for state collisions
- `Internal` for unexpected failures

Guidelines:
- Propagate errors with `?`.
- Map domain/application errors to HTTP status in `apps/api`.
- Never ignore `Result` values.
- Avoid stringly-typed errors when a typed variant exists.

## Axum Handler Patterns

Keep handlers thin:

1. Parse and validate transport DTOs.
2. Call application service.
3. Map domain object to response DTO.
4. Convert `AppError` to HTTP response via a shared mapper.

Do not place business rules directly in handlers.

## Traits and Ports

Application crate owns trait ports (for example repositories).
Infrastructure crate implements those ports.

Use `Send + Sync` bounds for shared runtime components:

```rust
pub trait MetadataRepository: Send + Sync {
    fn save_entity(&self, entity: EntityDefinition) -> AppResult<()>;
    fn list_entities(&self) -> AppResult<Vec<EntityDefinition>>;
}
```

## Async and Concurrency

- Prefer async for IO boundaries (HTTP, DB, messaging).
- For background tasks, use `tokio::spawn` and log failures.
- Protect shared mutable state with `RwLock`/`Mutex` only where necessary.
- Keep lock scope minimal.

## Serialization

Use `serde` derive consistently for DTOs and persistence models.

- `#[serde(rename_all = "snake_case")]` for external enums when needed.
- `#[serde(default)]` for backward-compatible fields.
- Keep transport DTOs separate from domain types if contracts diverge.

## Comments and Documentation

- Use module-level `//!` comments to state module purpose.
- Use `///` on public types/functions.
- Explain why, not what.
- Remove stale comments during refactors.

## Testing

Test focus by layer:

- `domain`: invariants and constructors
- `application`: use-case behavior with fakes/mocks
- `infrastructure`: adapter behavior and edge cases
- `apps/api`: handler/status mapping and payload contracts

Use descriptive test names and assert expected failures explicitly.

`unwrap`/`expect` are acceptable in tests only.

## Security and Reliability

- Never log secrets or credentials.
- Validate all external inputs at boundaries.
- Keep tenant/authorization checks explicit in application paths.
- Prefer explicit retries/timeouts for external dependencies.

## Panics and Unsafe

- No `unsafe`.
- Avoid panics in production paths.
- Use typed errors and recovery paths instead.
