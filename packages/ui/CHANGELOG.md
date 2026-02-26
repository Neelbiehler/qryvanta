# @qryvanta/ui

## 0.3.0

### Minor Changes

- fcfa69f: Add a new style primitive set to `@qryvanta/ui`, including tabs, command bar, dialog, accordion, tooltip, empty state, split pane, property panel, tree view, data grid, and search filter bar components. Replace the toast stub with a context-based toast system and add dedicated client entry points for dialog, toast, split pane, tree view, and data grid.

## 0.2.2

### Patch Changes

- c893fab: Fix Next.js App Router server/client boundaries by keeping the `@qryvanta/ui` root export server-safe and moving dropdown menu primitives to the client-only subpath `@qryvanta/ui/dropdown-menu`.

## 0.2.1

### Patch Changes

- f40c757: Add shadcn attribution in package metadata and README to improve package clarity on npm.

## 0.2.0

### Minor Changes

- c4103e3: Prepare `@qryvanta/ui` for external npm consumption with built `dist` exports,
  changesets-based release tooling, and release workflow automation.
