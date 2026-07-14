# Dioxus UI Modularization

Date: 2026-07-14

## Decision

PolyGlid Desktop UI code is organized by control boundary instead of keeping
the full application in `src/main.rs`.

## Structure

- `ui/app.rs`: root composition
- `ui/state.rs`: centralized Dioxus signals
- `ui/models.rs`: shared UI models
- `ui/shell.rs`: title bar, activity rail, and status bar
- `ui/sidebar.rs`: view-specific navigation sidebars
- `ui/editor.rs`: active feature routing and editor tabs
- `ui/bottom_panel.rs`: problems, output, and terminal panels
- `ui/overlays.rs`: settings and command palette
- `ui/components.rs`: reusable controls and cards
- `ui/features/`: scanner, plugins, tracks, automation, and agents
- `ui/preview.rs`: temporary seeded preview data

`src/main.rs` now launches the desktop window only. New feature behavior should
be added to its named feature module and coordinated through `AppState` when
multiple regions need the same state.

## Verification

- `cargo fmt -p polyglid-desktop -- --check`
- `cargo check -p polyglid-desktop --offline`
- `cargo test -p polyglid-desktop --offline` (4 passed)
- `cargo clippy -p polyglid-desktop --offline -- -D warnings`
- Brief desktop launch completed without a startup error.

The UI behavior and visual shell remain preview implementations; workspace
discovery, SQLite, and real automation execution remain follow-up phases.
