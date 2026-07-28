# Desktop Scope

## Paths

- Application: `apps/desktop/src/main.rs`
- UI root: `apps/desktop/src/ui/app.rs`
- Shared UI state: `apps/desktop/src/ui/state.rs`
- Project feature: `apps/desktop/src/ui/features/projects.rs`
- Shell pieces: `shell.rs`, `sidebar.rs`, `top_bar.rs`, `editor.rs`
- Styles: `apps/desktop/assets/`

## Boundaries

- Dioxus components render and coordinate user interactions.
- Use `DesktopControllers`; do not access SQLite, filesystem services, or
  Wasmtime directly from a view.
- Move synchronous gateway work into `tokio::task::spawn_blocking`.
- Represent startup, loading, empty, success, operation, and error states.
- Prevent concurrent project mutations through the operation state.
- Keep permanent deletion visually distinct and explicitly confirmed.
- Persist shell changes through `SettingsController`.

## Current UI Scope

Projects is the only enabled product feature. The activity rail, top bar,
workspace sidebar, editor area, and status bar form the shell, but they must not
advertise future feature areas as working.
