# ADR-0001: Enterprise Security Foundation

## Status

Accepted

## Context

Qryvanta is evolving toward enterprise-grade metadata runtime and app-builder capabilities.
The current baseline has authentication and tenant membership primitives, but it needs
consistent authorization and immutable auditability to support production workloads.

## Decision

Adopt a security-first baseline with three mandatory controls:

1. Tenant-scoped RBAC checks in application use-cases.
2. Append-only audit logging for state-changing behavior.
3. Default owner role assignment when tenant membership is bootstrapped.

Implementation constraints:

- Authorization policy checks live in `crates/application`.
- Permissions and action identifiers live in `crates/domain`.
- Storage and query adapters live in `crates/infrastructure`.
- API handlers remain transport-only and delegate to application services.

## Consequences

Positive:

- Explicit tenant and permission checks are enforced outside HTTP routes.
- Security-sensitive writes become traceable by actor and tenant.
- Bootstrap identities receive deterministic permissions via system role binding.

Tradeoffs:

- More migrations and repository complexity.
- Additional test surface for policy and audit semantics.

## Follow-up

- Add field-level and record-level authorization policies.
- Add role-management APIs and UI administration screens.
- Add audit retention controls and export workflows.
