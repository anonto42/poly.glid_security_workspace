# Legacy UI Removal

Date: 2026-07-14

## Decision

PolyGlid now has one canonical user interface:
`slices/polyglid-desktop`, implemented in Rust with Dioxus.

## Removed

- `slices/polyglid-web-legacy`
- `slices/polyglid-desktop-legacy`
- Node, React, and Tauri project automation entries
- Generated Node/Rust pseudo-project Make includes
- The nested Dioxus `Cargo.lock` and nested workspace boundary

## Integrated

- Added `slices/polyglid-desktop` to the root Cargo workspace.
- Added a `polyglid-desktop.mk` automation entry.
- Updated root Make targets to build, test, clean, and run Rust projects only.
- Updated workspace metadata, architecture notes, command references, and plans.
- Refreshed the root `Cargo.lock` with Dioxus dependencies.

## Verification

- `cargo check --workspace --offline`
- `cargo test --workspace --offline -- --skip test_plugin_manager_lifecycle`
- `cargo fmt -p polyglid-desktop -- --check`
- `env LIBRARY_PATH=/tmp/wpm-xdotool/usr/lib cargo build -p polyglid-desktop --offline`
- `.workspace/automation/scripts/validate-workspace.sh`
- Dry-ran the `wpm-build`, `wpm-run`, and `polyglid-desktop-dev` Make targets.

The lifecycle test remains skipped because it is a previously known hanging
test; this removal does not change that behavior.
