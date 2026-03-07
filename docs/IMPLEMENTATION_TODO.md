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

## Workflow Platform

- [x] WF-PLAT-01 Make the step graph the only canonical workflow definition model.
- [x] WF-PLAT-02 Add transactional outbox delivery for workflow-triggering runtime events and define chaining semantics.
- [x] WF-PLAT-03 Replace magic-entity templates with native typed workflow actions.
- [x] WF-PLAT-04 Add native trigger ingress for scheduler, webhook, inbound email, form, and approval flows.
- [x] WF-PLAT-05 Integrate workflows into publish/versioning, workflow-specific permissions, and governance controls.
- [x] WF-PLAT-05A Add draft/publish workflow lifecycle with immutable published versions pinned to workflow runs.
- [x] WF-PLAT-05B Add explicit workflow RBAC (`workflow.read`, `workflow.manage`).
- [x] WF-PLAT-05C Integrate workflows into workspace publish checks, diff, history, and selective publish.
- [x] WF-PLAT-05D Add governance policy for outbound workflows: publish-time inline credential blocking, trace redaction, and recent step-up for outbound publish/disable.
- [x] WF-PLAT-05E Add real secret/credential references for outbound workflow actions instead of inline-header bans.
- [x] WF-PLAT-06 Upgrade the workflow editor from JSON blobs to typed trigger/action authoring.
- [x] WF-PLAT-06A Replace remaining free-form JSON step/condition authoring with typed trigger/action forms.
- [x] WF-PLAT-06A1 Replace create/update/webhook/approval object-payload blobs and condition JSON values with typed field/value editors.
- [x] WF-PLAT-06A2 Add typed object-body authoring and mapping pickers for common HTTP request payloads, keeping raw JSON only for advanced body shapes.
- [x] WF-PLAT-06A3 Add typed array/scalar HTTP body authoring so raw JSON is reserved for advanced custom payloads.
- [x] WF-PLAT-06B Add credential selectors, test payload tooling, schema-aware mapping, and previewable action outputs in the workflow editor.
- [x] WF-PLAT-06B1 Add typed test-payload builders with trigger-aware sample presets and inline payload previews for common steps.
- [x] WF-PLAT-06B2 Add typed outbound credential selectors for HTTP request and webhook secret-backed auth headers.
- [x] WF-PLAT-06B3 Add provider-aware secret-reference builders for supported outbound credential schemes (`op://`, `aws-sm://`, `aws-ssm://`, `vault://`, `gcp-sm://`).
- [x] WF-PLAT-06B4 Add typed `Authorization` value-format selectors (`Raw`, `Bearer`, `Basic`) for secret-backed outbound auth headers.
- [x] WF-PLAT-07 Add end-to-end reliability coverage for outbox, queue, worker, retries, replay, and downstream failure modes.
- [x] WF-PLAT-07A Add end-to-end tests for record event -> outbox -> queue/worker -> action dispatch -> replay/history.
- [x] WF-PLAT-07A1 Add application-layer queued-flow coverage for runtime event -> drain -> claimed job -> action dispatch -> replay/history.
- [x] WF-PLAT-07A2 Add API-level ingress -> run list -> attempts -> replay coverage for native public webhook triggers.
- [x] WF-PLAT-07A3 Add API-level workspace record create -> runtime trigger -> run list/attempts/replay coverage.
- [x] WF-PLAT-07A4 Add queued API-level runtime trigger coverage for internal worker drain/claim control paths plus run history visibility.
- [x] WF-PLAT-07B Add duplicate delivery, lease-loss, and retry-behavior coverage for workflow runtime queues.
- [x] WF-PLAT-07B1 Add runtime-event release/requeue coverage for transient dispatch failures during outbox drain.
- [x] WF-PLAT-07B2 Add stale lease-token fencing coverage for claimed queued jobs.
- [x] WF-PLAT-07B3 Add Postgres queue coverage for expired job lease reclaim with new fencing tokens.
- [x] WF-PLAT-07B4 Add Postgres scheduler coverage for expired schedule-tick lease reclaim with new fencing tokens.
- [x] WF-PLAT-07B5 Add Postgres runtime-trigger outbox coverage for release/reclaim/complete semantics.
- [x] WF-PLAT-07C Add downstream failure-path coverage for outbound actions, including `429`, `5xx`, idempotent retry, and dead-letter handling.
- [x] WF-PLAT-07C1 Add explicit downstream `429` dead-letter coverage for outbound HTTP actions.
- [x] WF-PLAT-07C2 Add explicit downstream `5xx` dead-letter coverage for outbound webhook actions.
- [x] WF-PLAT-07C3 Add explicit provider-failure dead-letter coverage for outbound email actions.

Detailed file-level plan:
- `docs/internal/workflow-platform-upgrade-plan.md`
