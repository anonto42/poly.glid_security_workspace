# Real Workspace and Projects Slice

Date: 2026-07-15

## Decision

The attached production specification is the desktop acceptance target. Delivery is
split into ordered gates in `shared/plans/polyglid-production-desktop-plan.md`.
Workspaces/projects are the first real end-to-end slice; preview features are removed
gate by gate instead of being replaced simultaneously.

## Implemented

- Added SQLite schema version 5 for workspaces and projects.
- Added core stores and `WorkspaceCatalogService` for discovery, activation,
  creation, rename, catalog exclusion, and confirmed file deletion.
- Direct-child discovery ignores directory symlinks and identifies common project
  kinds from manifests.
- Added typed workspace/project events in `polyglid-events`.
- Connected Dioxus My Projects and its workspace picker to the core service through
  a desktop adapter. Dioxus does not import `rusqlite`.
- Added real loading, empty, ready, error, create, rename, refresh, remove, and
  delete-confirmation UI states.
- Corrected `ARCHITECTURE.md` to describe the Dioxus desktop boundary.

## Verification

- Focused core catalog tests: 3 passed.
- Desktop tests: 4 passed.
- Targeted core/events/desktop check passed offline.
- Manual desktop launch used `/tmp/polyglid-gate1-data` and discovered 10 real
  repository projects with a persisted `ready` workspace.

One unrelated existing test warning remains in `plugin_manager.rs` for an unused
`JsonRegistryStorage` import.
