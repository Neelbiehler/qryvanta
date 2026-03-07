# Workflow Platform Upgrade Plan

This document turns the current workflow gap analysis into an implementation plan grounded in the existing Qryvanta codebase.

## Target State

Workflows become a first-class platform subsystem with these properties:

- The step graph is the only canonical workflow definition model.
- Triggers are durable and delivered asynchronously from persisted domain events.
- Actions are native typed workflow operations, not disguised record creation templates.
- Workflow-originated writes have defined chaining semantics with recursion protection.
- Scheduler, webhook, inbound email, and form ingress are native trigger sources.
- Workflows participate in draft, publish, version history, rollback, and promotion flows.
- Workflow management has explicit permissions, audit semantics, and credential controls.
- The editor authors typed triggers and actions instead of raw JSON blobs.
- Reliability coverage exercises the end-to-end event -> queue -> worker -> action pipeline.

## Recommended Delivery Order

1. Canonical workflow model.
2. Durable event delivery and chaining semantics.
3. Native action types.
4. Native trigger sources and scheduler service.
5. Publish/versioning plus security/governance.
6. Typed editor authoring UX.
7. Reliability and operational test expansion.

## Architectural Decisions To Lock In

### Canonical model

- Persist only `trigger` plus `steps`.
- Remove persisted `action_type`, `action_entity_logical_name`, and `action_payload` after a one-way migration/backfill.
- Require at least one executable step at validation time.
- Keep condition as a graph node, not a transport-only wrapper.

### Chaining semantics

- Workflow-originated runtime writes should emit workflow events by default.
- Every emitted event should carry causal metadata:
  - `origin`
  - `workflow_run_id`
  - `workflow_step_path`
  - `chain_depth`
  - `ancestor_event_ids`
- Enforce a hard max depth and loop detection before dispatch.
- Record chain-termination reasons in run traces when dispatch is suppressed.

### Trigger delivery

- Record mutations write workflow events into a transactional outbox in the same database transaction.
- Dispatch moves out of API handlers and into an asynchronous event consumer.
- Delivery is at-least-once; action handlers and downstream integrations must stay idempotent.

## Epic WF-PLAT-01: Canonical Step Graph

Outcome:
- Workflows are defined, validated, transported, and persisted as `trigger + steps` only.

Primary code changes:
- `crates/domain/src/workflow.rs`
  - Remove `WorkflowAction` from `WorkflowDefinition` and `WorkflowDefinitionInput`.
  - Expand `WorkflowStep` validation as the single executable model.
  - Make `steps` required instead of optional.
- `crates/application/src/workflow_ports/execution.rs`
  - Remove `action` from `SaveWorkflowInput`.
  - Make `steps` required in service input.
- `crates/application/src/workflow_service/definitions.rs`
  - Stop auditing legacy action type.
  - Audit trigger type and step count only.
- `crates/application/src/workflow_service/execution.rs`
  - Execute directly from canonical steps without `effective_steps()` fallback.
- `crates/application/src/workflow_ports/repository.rs`
  - Keep repository contract centered on `WorkflowDefinition` with canonical steps only.
- `crates/infrastructure/src/postgres_workflow_repository/definitions.rs`
  - Stop reading and writing legacy action columns.
  - Persist only trigger metadata and step graph JSON.
- `crates/infrastructure/migrations/`
  - Add a migration that backfills `action_steps` from legacy action fields where needed.
  - Add a follow-up migration that drops legacy action columns and tightens constraints.
- `apps/api/src/dto/workflows/types.rs`
  - Remove `action_*` fields from save/list responses.
  - Keep step DTOs and add any missing typed step DTO variants introduced later.
- `apps/api/src/dto/workflows/conversions.rs`
  - Remove `first_action_from_steps` compatibility logic.
  - Require step graph on save.
- `apps/api/src/handlers/worker/claim.rs`
  - Stop sending legacy `workflow_action` payloads to workers.
- `apps/web/src/components/automation/workflow-studio/hooks/use-workflow-editor.ts`
  - Remove first-action synthesis before save.
- `apps/web/src/components/automation/workflow-studio/model.ts`
  - Remove legacy `ActionType` assumptions tied to save payloads.
- `packages/api-types/src/generated/*`
  - Regenerate transport contracts after DTO changes.

Acceptance criteria:
- No workflow path persists or requires legacy action fields.
- Existing workflows migrate forward without losing executable behavior.
- Worker claim payloads and API responses reflect only canonical steps.

## Epic WF-PLAT-02: Durable Event Delivery And Chaining

Outcome:
- Runtime writes cannot silently drop workflow dispatch, and workflow-triggered writes have explicit recursion control.

Primary code changes:
- `crates/application/src/metadata_service/runtime_records_write.rs`
  - Emit domain events for create, update, and delete from service-level writes.
  - Cover both checked and unchecked workflow write paths.
- `crates/application/src/workflow_service/dispatch.rs`
  - Split event intake from event execution.
  - Accept persisted outbox events instead of direct handler calls.
  - Apply recursion guard, depth limit, and loop detection.
- `crates/application/src/workflow_ports/repository.rs`
  - Add workflow event outbox enqueue, claim, ack, retry, and dead-letter operations.
- `crates/application/src/workflow_ports/runtime_records.rs`
  - Extend runtime record port to support update and delete once native actions land.
- `crates/infrastructure/migrations/`
  - Add workflow event outbox tables, leases, retry counters, attempt timestamps, and causal metadata columns.
- `crates/infrastructure/src/postgres_metadata_repository/runtime_records/write.rs`
  - Write outbox rows in the same transaction as runtime record mutations.
- `crates/infrastructure/src/postgres_workflow_repository/queue.rs`
  - Add outbox consumer persistence if workflow repository owns the queue path.
- `apps/api/src/handlers/apps/workspace/records.rs`
  - Remove best-effort post-write workflow dispatch calls.
- `apps/api/src/handlers/runtime/handlers.rs`
  - Remove best-effort post-write workflow dispatch calls from runtime endpoints as well.
- `apps/worker`
  - Add or extend a worker loop to consume outbox events and dispatch workflow runs asynchronously.

Acceptance criteria:
- A successful record mutation always persists a corresponding workflow event.
- API handlers no longer decide whether workflow dispatch happens.
- Workflow-created writes can trigger follow-on workflows until recursion guards stop the chain.
- Duplicate delivery is safe and observable.

## Epic WF-PLAT-03: Native Action Types

Outcome:
- The engine executes typed workflow actions directly instead of inferring external behavior from magic entity names.

Initial native actions:
- `send_email`
- `http_request`
- `webhook`
- `create_record`
- `update_record`
- `delete_record`
- `assign_owner`
- `delay`
- `approval_request`

Primary code changes:
- `crates/domain/src/workflow.rs`
  - Replace narrow `WorkflowAction` and `WorkflowStep` variants with typed native action configs.
- `crates/application/src/workflow_service/execution/actions.rs`
  - Dispatch per native action type instead of by entity-name convention.
- `crates/application/src/workflow_service/execution.rs`
  - Capture typed input/output traces for each action.
- `crates/application/src/workflow_ports/action_dispatcher.rs`
  - Expand dispatcher contracts around explicit action requests and typed config payloads.
- `crates/application/src/workflow_ports/runtime_records.rs`
  - Support create, update, delete, and owner assignment paths needed by native actions.
- `crates/infrastructure/src/http_workflow_action_dispatcher.rs`
  - Execute `http_request`, `webhook`, and `send_email` as explicit action categories.
- `apps/api/src/dto/workflows/types.rs`
  - Add typed DTOs for each native action config.
- `apps/api/src/dto/workflows/conversions.rs`
  - Map transport DTOs to native domain step variants.
- `apps/web/src/components/automation/workflow-studio/model/flow-templates.ts`
  - Stop representing action templates as `create_runtime_record` presets.
- `apps/web/src/components/automation/workflow-studio/model.ts`
  - Add draft types for native actions.
- `apps/web/src/components/automation/workflow-studio/panels/`
  - Add typed inspector panels per action.
- `apps/docs/content/docs/concepts/automation-basics.mdx`
  - Replace “templates still use primitives” language with native action behavior.
- `apps/docs/content/docs/operations/workflow-integration-runbook.mdx`
  - Document action-specific reliability, retries, and idempotency semantics.

Acceptance criteria:
- No external integration depends on entity names like `email_outbox` or `webhook_dispatch`.
- Each native action validates config before save and reports typed traces after execution.
- Legacy template behavior is migrated to native actions with parity tests.

## Epic WF-PLAT-04: Native Trigger Sources And Scheduler

Outcome:
- Webhook, inbound email, form submission, approval, and schedule triggers are first-class ingress paths.

Primary code changes:
- `crates/domain/src/workflow.rs`
  - Expand `WorkflowTrigger` beyond manual/runtime CRUD/schedule tick when native ingress is implemented.
- `crates/application/src/workflow_service/dispatch.rs`
  - Normalize payload envelopes for native ingress events.
- `apps/api/src/handlers/workflows.rs`
  - Keep manual execution separate from system ingress.
- `apps/api/src/api_router/`
  - Add protected and public ingress routes as needed for webhook and email/form adapters.
- `apps/worker`
  - Add a scheduler loop that emits schedule events without relying on a protected API call.
- `apps/web/src/components/automation/workflow-studio/model/flow-templates.ts`
  - Convert trigger presets into real trigger types instead of record conventions.
- `apps/web/src/components/automation/workflow-studio/panels/trigger-config-panel.tsx`
  - Add typed trigger configuration UI for scheduler, webhook auth, mailbox source, and form source.
- `apps/docs/content/docs/concepts/automation-basics.mdx`
  - Document native trigger availability and required deployment components.
- `apps/docs/content/docs/operations/workflow-integration-runbook.mdx`
  - Replace manual schedule dispatch guidance with scheduler service operations.

Acceptance criteria:
- Schedule triggers run from a built-in scheduler service.
- Webhook, email, and form ingress produce native workflow events without runtime-record shims.
- Trigger payload envelopes are stable and documented.

## Epic WF-PLAT-05: Publish, Versioning, Security, And Governance

Outcome:
- Workflows participate in release management and have dedicated governance controls.

Primary code changes:
- `crates/application/src/workflow_service/definitions.rs`
  - Split draft save from publish/enable operations.
- `crates/application/src/metadata_service/publish.rs`
  - Integrate workflows into workspace publish selection and execution.
- `crates/application/src/metadata_service/publish_validation.rs`
  - Add workflow publish validation for triggers, action configs, credentials, and referenced entities/apps.
- `apps/api/src/handlers/publish/handlers.rs`
  - Include workflows in workspace checks, diff, publish, and history responses.
- `apps/api/src/dto/publish.rs`
  - Extend publish DTOs for workflow surfaces.
- `apps/api/src/dto/security/types.rs`
  - Add explicit workflow permissions and governance DTOs as needed.
- `crates/domain`
  - Add workflow draft/published version concepts and workflow-specific permissions.
- `crates/infrastructure/src/postgres_workflow_repository/definitions.rs`
  - Persist draft and published workflow versions plus rollback metadata.
- `crates/infrastructure/migrations/`
  - Add workflow version tables, publish history, credential reference fields, and policy constraints.
- `apps/docs/content/docs/workspace/publish-and-access.mdx`
  - Document workflow release flow.
- `apps/docs/content/docs/operations/security-hardening.mdx`
  - Document secret handling, high-risk action approval, and audit semantics.

Acceptance criteria:
- Workflow edits are draft-first.
- Publish checks fail closed for invalid actions, missing credentials, or unsafe configuration.
- Workflow edits, publishes, rollbacks, and external dispatches are auditable.

## Epic WF-PLAT-06: Typed Workflow Editor

Outcome:
- Makers author workflows through schema-aware typed forms rather than raw JSON payload editing.

Primary code changes:
- `apps/web/src/components/automation/workflow-studio/hooks/use-workflow-editor.ts`
  - Compile typed trigger/action forms into DTOs without free-form JSON synthesis.
- `apps/web/src/components/automation/workflow-studio/model.ts`
  - Replace `dataJson` and `valueJson` string fields with typed draft config models.
- `apps/web/src/components/automation/workflow-studio/panels/workflow-builder-panel.tsx`
  - Show typed node summaries and validation status.
- `apps/web/src/components/automation/workflow-studio/panels/trigger-config-panel.tsx`
  - Render trigger-specific typed forms.
- `apps/web/src/components/automation/workflow-studio/panels/expression-builder-popover.tsx`
  - Keep token insertion, but target typed inputs and mappings.
- `apps/web/src/components/automation/workflow-studio/hooks/use-runtime-schemas.ts`
  - Feed schema-aware mapping pickers.
- `apps/web/src/components/automation/workflow-studio/model/flow-templates.ts`
  - Reduce templates to insertion shortcuts, not semantic emulation layers.
- `packages/api-types/src/generated/*`
  - Regenerate client types for typed trigger/action DTOs.

Acceptance criteria:
- Common actions and triggers can be configured without raw JSON textareas.
- Credential selection, test payloads, mapping previews, and output previews are available for native actions.
- Save-time validation errors map back to typed fields.

## Epic WF-PLAT-07: Reliability And End-To-End Coverage

Outcome:
- The workflow subsystem is covered as an operational chain, not just as unit-tested executor fragments.

Primary code changes:
- `crates/application/src/workflow_service/tests.rs`
  - Add outbox, duplicate delivery, retry, lease-loss, chained workflow, and loop-guard scenarios.
- `crates/infrastructure/src/postgres_workflow_repository/tests.rs`
  - Add durable outbox and delivery-state tests against Postgres behavior.
- `apps/api/src/api_router/tests.rs`
  - Add end-to-end tests for ingress -> outbox -> run history visibility.
- `apps/docs/content/docs/operations/workflow-integration-runbook.mdx`
  - Add failure modes, replay expectations, and downstream backpressure guidance.

Acceptance criteria:
- Coverage includes record event -> outbox -> queue -> worker -> action dispatch -> replay/history.
- Duplicate delivery, 429/5xx retry behavior, and lease-loss behavior are tested.
- Runbook guidance matches actual runtime semantics.

## Suggested First Slice

The smallest meaningful tranche is:

1. Finish WF-PLAT-01.
2. Land WF-PLAT-02 for durable outbox-backed runtime CRUD triggers.
3. In the same tranche, decide chaining semantics and enforce recursion guards.

That slice removes the most dangerous architectural debt first:

- dual workflow models
- silent dispatch loss
- undefined workflow-to-workflow behavior
