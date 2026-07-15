# PolyGlid Production Desktop Plan

Accepted: 2026-07-15

## Outcome

Replace every desktop preview with real, persisted behavior while preserving:

- Dioxus desktop as the only UI.
- `polyglid-core` ownership of domain rules and SQLite repositories.
- `polyglid-runtime` ownership of sandboxed WASM execution.
- `polyglid-events` ownership of typed cross-boundary events.
- Denied-by-default capabilities and sequential signature, trust, permission checks.

No feature is complete until empty, loading, populated, error, disabled, and
success states are verified where applicable.

## Current baseline

Already present in core/runtime: SQLite migrations and stores for settings,
targets, plugins, executions, reports, permissions, signatures, publishers,
audit, marketplace, and collaboration; plugin lifecycle services; execution
manager; report exporters; Wasmtime runtime; typed plugin events.

Missing or incomplete: workspace/project schema and discovery, Dioxus service
adapter, most desktop event wiring, real UI data, pipelines, work-track storage,
complete command registry, toasts, file pickers, and production accessibility.

`ARCHITECTURE.md` must describe Dioxus rather than the removed Tauri/React client.

## Gate 0 — Contracts and foundations

- Correct architecture documentation and crate names.
- Define application-service interfaces consumed by desktop and CLI.
- Define typed workspace, project, plugin, execution, settings, audit, pipeline,
  track, notification, and command events in `polyglid-events`.
- Establish platform paths for database, plugins, reports, and default workspace.
- Establish shared error codes and UI-safe messages.
- Tests prove desktop and CLI do not import `rusqlite`.

Exit: workspace builds; boundaries are documented and compiler-verifiable.

## Gate 1 — Workspaces and projects

Status: implemented on 2026-07-15; target-size visual acceptance remains.

- Migration adds `workspaces` and `projects`, active-workspace uniqueness, paths,
  timestamps, discovery state, and indexes.
- Core store and service implement list, discover, activate, create, rename,
  archive/remove, and refresh operations.
- Filesystem discovery does not follow symlink directories and reports missing or
  denied roots without destroying previous metadata.
- Active workspace persists and reloads after restart.
- Desktop adds My Projects with loading skeleton, empty CTA, populated cards/list,
  selection, create/rename/remove dialogs, and recoverable error state.

Exit: restart test and core integration tests pass; no project preview data remains.

## Gate 2 — Plugin management

- Desktop consumes `PluginService` and persisted plugin/signature/trust records.
- File picker feeds validation, signature verification, publisher trust, capability
  consent, installation, registry refresh, enable/disable, and uninstall flows.
- Statuses: active, disabled, invalid/error, unsigned, unknown publisher, revoked.
- Detail screen includes capabilities, publisher, signature, path, and last run.

Exit: install/toggle/restart/uninstall tests pass with audited decisions.

## Gate 3 — Scanner and execution

- File/directory target picker and validated saved targets.
- Submission creates a job before execution and emits progress/log/state events.
- Enforce signature -> trust -> capability checks sequentially and audit each gate.
- Render typed report plus raw/log/timeline tabs; export JSON, Markdown, HTML, SARIF.
- Execution history supports sorting and success/failure/timeout/cancelled states.

Exit: real signed test component executes; failures populate Problems and history.

## Gate 4 — Settings, security, and audit

- Persist Strict, Balanced, and Development profiles and validated fuel limits.
- Trust-store CRUD, permission grant/expiry/revocation, and audit-log filtering.
- Reset-to-defaults is explicit, audited, and restores profile-derived settings.
- Settings updates propagate through typed events without restarting the app.

Exit: profile behavior and restart persistence tests pass.

## Gate 5 — Work tracks

- Migration and core service for tracks, project/plugin links, status transitions,
  ordering, timestamps, audit, and optimistic versions.
- UI create/edit/archive, project/status filters, details, and empty/error states.
- Allowed states: backlog, in-progress, review, done.

Exit: transition, filter, persistence, and restart tests pass; seed tracks removed.

## Gate 6 — Automation pipelines

- Migration and service for pipelines, ordered steps, dependencies, runs, and logs.
- UI create/edit/delete, accessible reordering, status, execution, history, and logs.
- Pipeline stops or continues according to explicit failure policy.

Exit: multi-plugin success/failure/restart tests pass.

## Gate 7 — Command, notifications, and shell polish

- Typed command registry with fuzzy search and keyboard activation.
- Workspace switcher reads real workspace records.
- Event-driven toast queue: maximum four, dismissible, five-second default.
- Resizable/collapsible sidebar and bottom panel with persisted dimensions.
- Terminal remains clearly marked as reserved until a safe PTY design is approved.

Exit: keyboard-only manual test and command/toast tests pass.

## Gate 8 — Production UI and accessibility

- Apply the Obsidian Emerald tokens consistently; no isolated magic colors.
- Complete hover, focus-visible, active, disabled, loading, empty, error, and
  success treatment for every interactive component.
- Modal Escape handling and focus management; accessible tabs, lists, and menus.
- Reduced-motion support, contrast audit, tooltips, responsive collapse, skeletons.
- Virtualize lists over 100 rows and debounce searches by 150ms.

Exit: 900x600, 1280x820, keyboard, reduced-motion, and performance checks pass.

## Final removal gate

- `rg` finds no `seed_`, `sample_`, preview behavior, or mock records in desktop.
- Full workspace format, check, tests, Clippy, and security tests pass.
- Database upgrade and backup/restore are tested from every supported schema.
- Manual acceptance checklist passes for every screen and failure mode.
- Documentation and `.agents` current-state memory match shipped behavior.

## Implementation rules

- Deliver one vertical gate at a time; do not wire every screen simultaneously.
- A Dioxus component never opens SQLite, manipulates plugin files, or invokes
  Wasmtime directly.
- Core services validate all input and return typed records/errors.
- Destructive filesystem deletion requires explicit confirmation and a separate
  operation from removing catalog metadata.
- Plugins provide typed WIT reports/panel descriptors, never arbitrary desktop UI.
- Backend/Axum remains optional for local desktop operation.
