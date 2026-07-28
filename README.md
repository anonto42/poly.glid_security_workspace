# PolyGlid

PolyGlid is being built one working product phase at a time. The current phase
contains only the cross-platform Dioxus desktop shell and local project
workspace.

## Current functionality

- Open the local PolyGlid workspace.
- Discover persisted projects.
- Create and select a project.
- Rename a project.
- Remove a project from the catalog.
- Permanently delete a project folder after explicit confirmation.
- Switch between configured local workspaces.

## Repository

```text
apps/desktop/       Dioxus desktop application
crates/client/      UI-independent controllers and local adapter
crates/config/      persisted configuration models
crates/core/        local project and application services
crates/events/      shared event types required by the client
crates/plugin-api/  shared domain types required by the core
crates/runtime/     runtime dependency required by the local client
```

Future product areas are intentionally absent. They will be introduced only
when their development phase begins.

## Development

```bash
cargo run -p polyglid-desktop
cargo check --workspace
cargo test --workspace
```
