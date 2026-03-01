# Documentation Policy

Documentation updates are mandatory for all externally meaningful changes.

## Changes That Require Docs Updates

- New features
- API contract changes
- Metadata model changes
- Configuration changes
- Migration and deployment changes
- Operational and observability changes

## Required Update Locations

1. End-user and self-hosting docs: `apps/docs/content/docs`
2. Repository-level maintainer context: `docs/`
3. Process rules when needed: `AGENTS.md`

## Structure Requirements

- Place docs in the correct section and keep navigation updated.
- Prefer extending existing pages over adding fragmented one-off pages.
- Keep commands, endpoints, and config names current.

## Pull Request Checklist

1. Behavior changes have matching documentation.
2. New pages are discoverable from docs navigation.
3. Snippets and examples reflect current implementation.

Docs updates may be skipped only for internal refactors with no behavior or operational impact.
