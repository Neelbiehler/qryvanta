# Qryvanta Maker Portal – Strategy

## Vision

The Qryvanta Maker Portal becomes the open-source alternative to Microsoft Power Platform's Maker Experience. Makers define their data model, compose Model-Driven Apps from it, and publish them – all visually, without code. The runtime automatically generates UI, navigation, and business logic from the published metadata.

---

## 1. What Power Platform Gets Right (and What Qryvanta Should Adopt)

Power Platform's strength lies in a clear three-step flow:

**Dataverse → Maker Studio → Runtime App.** The maker first models tables, columns, and relationships. Then they compose an app by adding tables as pages, configuring forms and views, and building a sitemap. Finally they publish, and the runtime renders a consistent UI automatically.

The decisive principles:

- **Data-first, UI-second:** Data modeling determines 80% of the app. The UI follows from it.
- **Visual WYSIWYG:** The maker sees how the app will look while configuring it.
- **Component-based:** Forms, views, charts, and dashboards are reusable building blocks.
- **Publish lifecycle:** Changes are drafts until explicitly published.
- **Solution-based:** Everything lives in "solutions" that can be transported between environments.

---

## 2. Current State of Qryvanta & Gaps

### What Already Exists
- Metadata entities & fields with versioned publish lifecycle ✅
- Runtime CRUD/query APIs generated from published metadata definitions ✅
- App and workspace model (Admin / Maker / Worker) ✅
- Tenant-scoped RBAC ✅
- Rust/Axum backend, Next.js frontend ✅

### What's Missing (the Poor UX/DX)
- **No visual app designer:** The maker cannot click-compose an app from components.
- **No form designer:** Forms are not built visually – at best configured via JSON/config.
- **No view designer:** No way to visually configure list views with columns, filters, and sorting.
- **No sitemap/navigation composition:** The maker cannot define which entities appear in the app navigation.
- **No live preview:** Makers don't see how their changes look until they publish.
- **No relationship visualizer:** Entity relationships (lookups, 1:N, N:N) are not visually configurable.
- **No business rules engine:** No no-code way to define field validation, show/hide, or auto-set logic.
- **No dashboard/chart composition.**
- **Developer experience (DX):** No CLI/SDK/API for programmatic metadata manipulation; everything only through the (unfinished) UI.

---

## 3. Target Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Maker Portal (Next.js)                │
│                                                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │ Data      │  │ App      │  │ Form     │  │ View   │  │
│  │ Modeler   │  │ Designer │  │ Designer │  │ Builder│  │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │
│  │ Business  │  │ Dashboard│  │ Sitemap  │  │ Publish│  │
│  │ Rules     │  │ Builder  │  │ Editor   │  │ Center │  │
│  └──────────┘  └──────────┘  └──────────┘  └────────┘  │
└───────────────────────┬─────────────────────────────────┘
                        │ Metadata API
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Metadata Service (Rust/Axum)                │
│                                                         │
│  Entity Definitions ─ Field Definitions ─ Relationships │
│  Form Definitions ─ View Definitions ─ App Definitions  │
│  Business Rules ─ Dashboard Definitions ─ Sitemaps      │
│  Versioned Draft/Publish Lifecycle                      │
└───────────────────────┬─────────────────────────────────┘
                        │ Published Metadata
                        ▼
┌─────────────────────────────────────────────────────────┐
│              Runtime Engine (Rust/Axum)                  │
│                                                         │
│  Dynamic CRUD ─ Query Engine ─ Business Rule Execution  │
│  Form Rendering Metadata ─ View Rendering Metadata      │
│  Navigation Resolution ─ RBAC Enforcement               │
└───────────────────────┬─────────────────────────────────┘
                        │ Runtime API
                        ▼
┌─────────────────────────────────────────────────────────┐
│            Worker App (Next.js – Model-Driven)          │
│                                                         │
│  Auto-generated Forms, Views, Dashboards, Navigation    │
│  Responsive, consistent UX across all Model-Driven Apps │
└─────────────────────────────────────────────────────────┘
```

---

## 4. Component Strategy in Detail

### 4.1 Data Modeler (Entity Designer)

**Purpose:** Visual editor for entities, fields, and relationships – the equivalent of the Dataverse Table Designer.

**Features:**
- Create/edit entities with display name, plural name, icon, description
- Add fields with type picker (Text, Number, Date, Currency, Lookup, Choice, Boolean, File, Image, Auto-Number, Formula)
- Visually define relationships: 1:N (master-detail), N:N (junction), self-referential
- ERD-style visualization of all entity relationships in the workspace
- Configure calculated/rollup fields
- Draft status visible: "2 unpublished changes"

**UX principle:** Canvas-based relationship graph + detail panel on the right. Like a mini ERD tool.

**Metadata API contract:**
```
POST   /api/v1/metadata/entities
PATCH  /api/v1/metadata/entities/{id}
POST   /api/v1/metadata/entities/{id}/fields
POST   /api/v1/metadata/entities/{id}/relationships
POST   /api/v1/metadata/entities/{id}/publish
```

### 4.2 App Designer

**Purpose:** Composition of a model-driven app from entities, forms, views, and dashboard components. Defines what the worker sees.

**Features:**
- Create app with name, icon, description
- Add pages → select entity → default form and view are set automatically
- Sitemap editor: arrange groups and pages via drag & drop
- Per page: select/override form and view
- App preview: live preview of how the app will appear to workers
- Multi-app support: different apps per workspace with different entity subsets

**UX principle:** Left sidebar = sitemap (drag & drop). Right = preview of the selected page. Similar to the Power Apps Modern App Designer.

**Metadata API contract:**
```
POST   /api/v1/metadata/apps
PATCH  /api/v1/metadata/apps/{id}
PUT    /api/v1/metadata/apps/{id}/sitemap
POST   /api/v1/metadata/apps/{id}/publish
```

### 4.3 Form Designer

**Purpose:** Visual drag & drop editor for entity forms. Defines layout, tabs, sections, and which fields appear where.

**Features:**
- Drag & drop entity fields onto a form canvas
- Organize tabs and sections
- Column layout per section (1, 2, or 3 columns)
- Embed sub-grids (show related records inline)
- Quick-view forms for lookup fields
- Conditional visibility via business rules (field X visible when Y = true)
- Header area with key fields
- Multiple forms per entity (Main Form, Quick Create, Quick View)

**UX principle:** True WYSIWYG. The maker drags fields onto a form that looks exactly like it will in the worker app. Right property panel for field settings.

**Metadata API contract:**
```
POST   /api/v1/metadata/entities/{id}/forms
PATCH  /api/v1/metadata/forms/{id}
GET    /api/v1/metadata/forms/{id}  → { tabs, sections, controls, layout }
```

### 4.4 View Builder

**Purpose:** Configuration of list views (grids) for entities.

**Features:**
- Select and arrange columns (drag & drop)
- Set column widths
- Configure sorting (default sort)
- Filter builder: visual query builder (field + operator + value, AND/OR groups)
- System views (All Records, My Records, Active Records) vs. personal views
- Toggle inline editing mode
- Configure quick-find (which fields are searched)

**UX principle:** Live preview of the view with sample data. Filters as visual blocks, not SQL.

**Metadata API contract:**
```
POST   /api/v1/metadata/entities/{id}/views
PATCH  /api/v1/metadata/views/{id}
GET    /api/v1/metadata/views/{id}  → { columns, sort, filters, fetchXml-equivalent }
```

### 4.5 Business Rules Engine

**Purpose:** No-code definition of business logic executed at runtime.

**Features:**
- Visually define condition → action pairs:
  - **Show/Hide Field** based on another field
  - **Set Required / Not Required**
  - **Set Default Value**
  - **Set Field Value** (calculated)
  - **Lock/Unlock Field**
  - **Show Error Message** (validation)
- Scope: form-level (client-side) or entity-level (server-side)
- Visual flow editor with if/then/else blocks

**UX principle:** Flowchart-style editor. Simple condition builders, no formula language needed for basics.

**Metadata API contract:**
```
POST   /api/v1/metadata/entities/{id}/business-rules
PATCH  /api/v1/metadata/business-rules/{id}
```

### 4.6 Dashboard & Chart Builder

**Purpose:** Assembly of dashboards from charts, views, and KPI tiles.

**Features:**
- Grid layout: place widgets via drag & drop
- Chart types: bar, line, pie, funnel, KPI card
- Data source = entity + view (filtered data basis)
- Aggregations: count, sum, avg, min, max grouped by field
- Dashboard configurable as app homepage

### 4.7 Publish Center

**Purpose:** Central management of the publish lifecycle.

**Features:**
- Overview of all draft changes per workspace
- Diff view: what has changed?
- Selective publish: publish only specific entities/apps
- Publish history with rollback option
- Dependency check: "App X references Entity Y, which is not yet published"
- Pre-publish validation: missing required fields in forms, invalid relationships, etc.

---

## 5. Worker App – Model-Driven Runtime

The worker app renders automatically from published metadata:

**Navigation:** The app's sitemap definition generates the left sidebar. Groups → areas, pages → nav items.

**Views (lists):** Table grid with configured columns, filters, sorting. Inline editing when enabled. Command bar with New, Delete, Bulk Actions.

**Forms (detail):** Tabs, sections, fields in the configured layout. Sub-grids for related records. Business rules run client-side. Header with key fields.

**Dashboards:** Configured charts and KPI tiles as homepage or standalone page.

**Consistent UX:** Every model-driven app looks the same regardless of the entity. This is a feature, not a bug – workers need no onboarding per app.

---

## 6. Developer Experience (DX)

### 6.1 CLI Tool: `qry`

```bash
# Export metadata (solution export equivalent)
qry metadata export --workspace my-ws --output ./metadata

# Import metadata (CI/CD pipeline)
qry metadata import --workspace my-ws --input ./metadata

# Scaffold an entity
qry entity create --name "Invoice" --fields "number:auto,date:date,total:currency"

# Trigger publish
qry publish --workspace my-ws --all

# Show diff
qry metadata diff --workspace my-ws
```

### 6.2 Metadata-as-Code (YAML/JSON)

For git-based workflows: metadata as declarative files that can be versioned and reviewed via PR.

```yaml
# entities/invoice.yaml
entity:
  name: Invoice
  display_name: Invoice
  plural_name: Invoices
  primary_field: number
  fields:
    - name: number
      type: auto_number
      format: "INV-{0000}"
    - name: date
      type: date
      required: true
    - name: total
      type: currency
      min: 0
    - name: customer
      type: lookup
      target_entity: customer
      relationship: many_to_one
```

### 6.3 TypeScript SDK for Custom Components

For cases where no-code isn't enough: custom controls as React components that embed into forms.

```typescript
import { useEntityRecord, useEntityView } from '@qryvanta/sdk';

// Custom component embedded in a form slot
export function InvoiceLineItems({ recordId }: { recordId: string }) {
  const { data, loading } = useEntityView('invoice_line', {
    filter: { invoice_id: recordId },
    columns: ['product', 'quantity', 'price', 'total'],
  });
  // Custom rendering...
}
```

### 6.4 Plugin/Hook System

Server-side extensions as hooks in the workflow system:

```yaml
# hooks/invoice-created.yaml
hook:
  trigger: entity.invoice.after_create
  action: workflow
  workflow: calculate-invoice-totals
```

---

## 7. Prioritized Roadmap

### 7.1 Alignment with Repository Delivery Phases (`AGENTS.md`)

To stay consistent with the implementation contract in `AGENTS.md`, this Maker Portal plan is mapped to the platform phases used across the monorepo.

| Platform Phase (`AGENTS.md`) | Platform Scope | Maker Portal Contribution |
|---|---|---|
| Phase 1 (foundation) | Storage, tenancy, baseline runtime composition | Baseline complete for metadata-backed CRUD and publish flow |
| Phase 2 (metadata runtime) | Rich metadata model + published runtime APIs | Complete metadata/runtime coverage required for forms, views, relations, and app navigation |
| Phase 3 (app builder) | Visual admin UX and metadata-driven rendering | Primary delivery phase for Form Designer, View Builder, and App Designer v2 |
| Phase 4 (automation) | Workflows and runtime actions | Business rules and hook integration |
| Phase 5 (security and operations) | RBAC hardening, audit, self-host ops | Publish governance, diff/rollback controls, operational readiness |

### 7.2 Current Status

- Foundation capabilities are available for entity/app creation, draft/publish lifecycle, and worker runtime rendering.
- Wave A metadata-runtime baseline is now largely implemented, and current focus is hardening plus parity gaps across maker/worker surfaces.
- Metadata runtime now supports calculated fields (`calculation_expression`) with server-side evaluation (`add`, `concat`) and write-protection for computed fields.
- Relationship baseline is in place: maker relation-field creation uses typed target selection, and worker create/detail forms render relation fields as lookup selects.
- Form Designer now supports section-level related-record sub-grids, and worker record detail pages render those sub-grids from published metadata.
- Worker runtime now includes metadata-driven record detail routes with form resolution order (`?form` override -> sitemap `default_form` -> first available `main` form).
- App Studio now supports deterministic sitemap page reordering through inline move actions, preserving per-binding form/view defaults during reorder saves.
- Sitemap editor now provides richer WYSIWYG behavior with visual selection highlights, clickable preview-to-properties focus, direct add/delete actions, and cross-parent drag/drop insertion.
- Form Designer and sitemap surfaces now include keyboard-first controls (`Alt + Arrow`, `Ctrl/Cmd + Z`, `Ctrl/Cmd + Y`) plus `?` shortcut overlays for faster editing.
- Publish Center now supports dependency-aware selective publish checks, unknown-selection issue reporting, dependency edge metadata (`dependency_path`), and server-backed run history (`/api/publish/history`).
- Publish Center diff preview now includes field-level and surface-level deltas (form/view/binding) with persisted publish-time snapshots as baseline.
- Publish diff behavior is protected by handler-level and end-to-end coverage, including multi-publish snapshot history and unknown selection reporting.
- Business Rules v1 baseline is now implemented: entity/form-scoped rule CRUD, runtime retrieval endpoint, visual condition/action authoring, and worker-side rule execution.
- Business-rule validation is enforced in both layers: worker forms block save on `show_error`, and API create/update now enforces active entity-scoped `show_error` rules server-side.
- Rust-first contract updates for publish and business-rule flows are generated and exported through `@qryvanta/api-types`.
- Platform docs now include dedicated implementation pages for App Designer v2, Form Designer, View Builder, Relationship Support, Business Rules, and Publish Center.

### 7.3 Delivery Waves (Maker Portal Stream)

#### Wave A (now): Metadata Runtime Completion
**Goal:** Ensure metadata contracts are complete and reliable before deep UI builder investment.
**Status:** Largely implemented; remaining work is hardening, parity gaps, and deeper relationship semantics.

1. Expand metadata definitions for forms/views/relationships/sitemap to production-ready fidelity.
2. Validate runtime endpoints for form layout, view query config, and navigation resolution.
3. Add dependency validation in publish flow (apps/forms/views/relationships).
4. Enforce tenant-safe and layer-correct implementations across domain/application/infrastructure.

#### Wave B: Visual Builders Core
**Goal:** Makers can visually compose forms and views without editing JSON.
**Status:** Baseline implemented; continue UX polish and advanced authoring workflows.

5. Form Designer canvas + properties panel (tabs, sections, column rules, field settings).
6. View Builder designer (columns, widths, default sorting, visual filters with AND/OR groups).
7. Relationship authoring and sub-grid configuration integrated into modeler + forms.
8. Multiple forms/views per entity with explicit default selection behavior.

#### Wave C: App Composition and Preview
**Goal:** Full model-driven app composition with maker confidence before publish.
**Status:** Core flows implemented; selective publish and preview cohesion are still being refined.

9. App Designer v2 sitemap drag-and-drop and page ordering/grouping.
10. Per-page form/view overrides and fallback rules.
11. Live preview loop for app, view, and form edits on draft metadata.
12. Publish Center UX for diff inspection and selective publish.

#### Wave D: Intelligence and Extensibility
**Goal:** No-code logic + pro-code escape hatches.
**Status:** Started via Business Rules v1 baseline; dashboards, CLI, and SDK/hook expansion remain.

13. Business Rules v1 (visibility, requiredness, default/set value, validation message).
14. Dashboard and chart composition with metadata-defined widgets.
15. CLI `qry` export/import/diff and metadata-as-code workflows.
16. SDK/hook surface for custom controls and workflow integration.

### 7.4 Completion Criteria per Wave

- Domain/application/infrastructure boundaries are respected per `AGENTS.md` architecture rules.
- Rust-first API contracts are reflected in generated TypeScript types (`@qryvanta/api-types`) when DTOs change.
- Tests cover domain invariants and application use-cases for every shipped capability.
- Lint/check/test pipelines pass before merge.
- User-facing behavior changes are documented in `apps/docs/content/docs` in the same change set.

### 7.5 Immediate Next Priorities (Post-Current Change Set)

1. Harden server-side Business Rules execution beyond validation-only flow: runtime create/update now applies `set_default_value` and `set_field_value`, honors `set_required` / `set_optional`, and enforces update-time lock preservation for `lock_field` targets.
2. Publish Center UX hardening is now delivered at baseline: workspace issues are grouped by severity/category, selective publish uses a unified validate-then-publish selection flow (`dry_run` + execute), and history supports run-level drill-down with selection replay.
3. Relationship tooling expansion now includes baseline N:N bridge authoring (`junction_pair` template in entity workbench) plus stronger selective-publish dependency diagnostics (`entity.field -> entity`) and in-flight relation dependency handling for co-selected entities.
4. Continue relationship tooling with deeper visual modeler/ERD interactions and explicit advanced semantics beyond lookup/bridge templates.
5. Regression coverage expansion is now advanced for interaction mechanics: app/sitemap ordering determinism, binding default-selection fallback rules, sitemap-structure integrity checks (duplicate/negative plus contiguous position guards), saved-sitemap ordering precedence over binding-derived order, and canonical position-order normalization on sitemap save/get are covered in application tests; form/view save paths now enforce position integrity invariants (non-negative, non-duplicate, contiguous) and canonical ordering for tabs/sections/placements/sub-grids and columns; multi-save reorder -> undo -> redo style transitions are now covered for sitemap/form/view persistence paths, and remaining work focuses on broader interaction edge cases.
6. Wave D dashboard/chart delivery baseline is started with domain models, worker dashboard API contracts, and a sitemap-driven worker dashboard surface; remaining work extends persisted dashboard/chart authoring and live aggregated data rendering.
7. Begin `qry` CLI scaffolding for metadata export/import/diff on top of the existing metadata and publish APIs.

---

## 8. Design Principles for Implementation

### 8.1 Maker-First Thinking
Every decision is made from the maker's perspective: "Can a business analyst without a dev background use this feature?" If the answer is no, the UX must be simplified.

### 8.2 Progressive Disclosure
Simple things are simple, complex things are possible. A maker should have a working app in 5 minutes. Advanced features (business rules, custom components) are opt-in.

### 8.3 Metadata-Driven Everything
The entire runtime is generated from metadata. There are no "hardcoded" screens in the worker app. Everything goes through the metadata layer: navigation, forms, views, validation, permissions.

### 8.4 Convention over Configuration
When creating an entity, default forms, default views, and sensible defaults are automatically generated. The maker only needs to customize what deviates.

### 8.5 Real-Time Preview
Every editor (form, view, app) has a live preview. The maker sees immediately how changes will look – without having to publish first.

### 8.6 Composable Architecture
Backend metadata APIs are designed so that both the visual Maker Portal and the CLI/Metadata-as-Code can build on them. A unified API contract for all consumers.

---

## 9. Differentiation from Power Platform

Qryvanta should deliberately differ from Power Platform in the following ways:

| Aspect | Power Platform | Qryvanta |
|---|---|---|
| Hosting | Cloud-only (Microsoft) | Self-hosted first, cloud optional |
| Licensing | Per-user, expensive | Open source (Apache 2.0) |
| Lock-in | Strong binding to Microsoft ecosystem | Open APIs, metadata-as-code, portable |
| Technology | .NET/C# backend | Rust backend (performance) |
| Extensibility | Plugins in C#, PCF controls | Hooks + TypeScript SDK + WASM plugins |
| Solution transport | Managed/unmanaged solutions | Git-based, YAML, CI/CD-native |
| Canvas apps | Yes (separate app type) | No – focus on model-driven |

---

## 10. Technical Implications for the Backend

### Metadata Entities (Rust Domain) – Implemented vs Planned

Current status in `crates/domain`:

- `FormDefinition` (+ tabs/sections/placements/sub-grids) – implemented
- `ViewDefinition` (+ columns/sort/filter groups) – implemented
- `AppDefinition` + sitemap types (`AppSitemap`, areas/groups/sub-areas) – implemented
- `BusinessRuleDefinition` (+ conditions/actions/scope) – implemented baseline
- `DashboardDefinition` – planned
- `ChartDefinition` – planned

Implemented metadata entities follow the existing draft/publish and versioned runtime pattern.

### Metadata and Publish Endpoints (Rust/Axum)

Implemented baseline route groups:

- `/api/entities/:entity_logical_name/forms` – form CRUD and validation
- `/api/entities/:entity_logical_name/views` – view CRUD and validation
- `/api/entities/:entity_logical_name/business-rules` – business rule CRUD
- `/api/apps/:app_logical_name/sitemap` and `/api/apps/:app_logical_name/publish-checks` – app composition and publish readiness
- `/api/publish/checks`, `/api/publish/diff`, `/api/publish/history` – workspace publish checks, diff preview, and run history

Rust-first DTOs for these flows are generated into `packages/api-types/src/generated` and consumed through `@qryvanta/api-types`.

### Runtime Extensions (Current Baseline)

- `GET /api/workspace/apps/:app_logical_name/navigation` returns resolved sitemap navigation for worker apps.
- `GET /api/workspace/apps/:app_logical_name/entities/:entity_logical_name/forms` and `.../forms/:form_logical_name` return form layout metadata.
- `GET /api/workspace/apps/:app_logical_name/entities/:entity_logical_name/views` and `.../views/:view_logical_name` return view metadata used for list rendering.
- `GET /api/runtime/:entity_logical_name/business-rules` returns active runtime business rules for worker clients.
- Runtime list/detail APIs resolve behavior from published schema + form/view metadata + app-level defaults.

---

## Summary

The core of the strategy: **Metadata is the source of truth.** Everything – every form, every view, every app, every rule – is a metadata record managed through a draft/publish lifecycle. The Maker Portal is the visual surface for creating this metadata. The Worker App is the runtime that generates a working business app from this metadata.
