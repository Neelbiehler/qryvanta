# Qryvanta Security and Enterprise Backlog

Status legend:
- `[ ]` planned
- `[~]` in progress
- `[x]` completed and retained as an ongoing control

This file is now a forward-looking implementation backlog focused on enterprise security, operational trust, and large-tenant readiness.

## Identity and Session Security

- [x] AUTH-01 Rotate server-side session identifiers on every successful authentication path.
- [x] AUTH-02 Enforce idle and absolute session expiration on authenticated API traffic.
- [x] AUTH-03 Require a strong dedicated `TOTP_ENCRYPTION_KEY` and fail startup on weak placeholders.
- [x] AUTH-04 Add authenticated session revocation across all active sessions after password reset and MFA reset.
- [x] AUTH-05 Add step-up authentication requirements for high-risk admin actions.
- [x] AUTH-06 Add tenant-aware session selection flow for identities that belong to multiple tenants.

## MFA and Recovery

- [x] MFA-01 Keep TOTP enrollment in pending state until proof-of-possession succeeds.
- [x] MFA-02 Add dedicated MFA attempt throttling keyed by user and challenge state.
- [x] MFA-03 Add admin-visible audit events for MFA enrollment start, confirm, disable, and recovery-code regeneration.
- [x] MFA-04 Add operator runbook for forced MFA reset after account recovery.

## Tenant Isolation

- [x] ISO-01 Keep tenant scoping explicit in repository and service paths.
- [~] ISO-02 Add PostgreSQL row-level security policies for tenant-owned tables as defense in depth.
- [x] ISO-02A Enforce RLS on metadata publish/runtime tables with tenant-scoped transaction context.
- [x] ISO-02B Extend RLS coverage to app, RBAC/security-admin, extension, and audit tables.
- [x] ISO-02C Extend RLS coverage to workflow, analytics, tenant-contact, and remaining tenant-owned tables.
- [x] ISO-02C1 Enforce RLS on tenant contact mappings and Qrywell analytics/search-event stats tables.
- [x] ISO-02C2 Extend RLS coverage to workflow queues, worker coordination, and any remaining tenant-owned operational tables.
- [x] ISO-02D Review auth tenancy tables (`tenant_memberships`, session data, subject-first bootstrap lookups) and either add safe RLS/bypass semantics or explicitly document why they remain service-managed.
- [x] ISO-03 Add automated authenticated IDOR regression coverage across runtime/entity/workflow route families.
- [x] ISO-03A Add router-level authenticated IDOR tests for entity definition read/update/publish surfaces.
- [x] ISO-03B Add router-level authenticated IDOR tests for workspace runtime record, form, view, and schema surfaces.
- [x] ISO-03C Add router-level authenticated IDOR tests for workflow list/run/replay/retry surfaces.
- [x] ISO-03D Make foreign-resource delete paths fail closed and cover them with authenticated router tests.
- [x] ISO-03E Add a scheduler-specific tenant-isolation regression for cross-tenant schedule dispatch variants.
- [x] ISO-04 Add formal tenant-switching semantics and default-tenant selection rules.

## Perimeter and Request Trust

- [x] NET-01 Refuse proxy-derived client IP trust unless explicitly enabled.
- [x] NET-02 Add default API and web security response headers.
- [x] NET-03 Add trusted proxy allowlist support instead of a single boolean toggle.
- [x] NET-04 Add deployment conformance tests for ingress/CDN header preservation and TLS forwarding.

## Secrets and Key Management

- [x] KEY-01 Publish key rotation procedures for bootstrap, worker, session, and MFA encryption secrets.
- [x] KEY-02 Add startup support for loading secrets from external secret managers.
- [x] KEY-03 Add envelope encryption / KMS integration for sensitive application secrets at rest.
- [x] KEY-04 Add drift detection for reused secrets across environments.

## Audit and Detection

- [x] AUD-01 Preserve immutable-audit mode as an operator option.
- [x] AUD-02 Add tamper-evident audit log chaining or external ledger export.
- [x] AUD-03 Add structured security event taxonomy for login, MFA, invites, secret rotation, and tenant admin actions.
- [ ] AUD-04 Add alertable anomaly signals for brute-force attempts, repeated invitation abuse, and tenant-crossing access failures.

## Supply Chain and Runtime Assurance

- [ ] SUP-01 Add image scanning and signed container provenance to the release workflow.
- [ ] SUP-02 Add dependency freshness SLOs and exception handling for pinned vulnerable transitive packages.
- [ ] SUP-03 Add hardened production container images that run as non-root with minimal packages.
- [ ] SUP-04 Add reproducible disaster-recovery drills for backup restore plus secret rotation validation.

## Verification Program

- [x] VER-01 Expand unit coverage around MFA pending-state transitions and security middleware.
- [x] VER-02 Add integration tests for secret-validation startup failures and trusted-proxy behavior.
- [x] VER-03 Add browser-level security header assertions for the Next.js app.
- [ ] VER-04 Add an external penetration test before any public multi-tenant production launch.
