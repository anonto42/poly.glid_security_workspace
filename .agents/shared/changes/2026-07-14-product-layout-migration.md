# Product-Oriented Project Layout Migration

Date: 2026-07-14

## Changed

- Removed language grouping from `slices/`.
- Moved every Rust crate directly under `slices/<product-name>`.
- Renamed `wpm` to `polyglid-desktop` and its Cargo package/binary accordingly.
- Moved the WIT contract to `slices/polyglid-contracts`.
- Moved the plugin to `slices/recon-probe`.
- Initially preserved the React and Tauri clients; both were removed after Dioxus
  was selected as the canonical desktop.
- Moved the Rust config example into `slices/polyglid-config`.
- Updated Cargo paths, Make targets, automation, documentation, diagrams, project
  registry, and agent placement rules.

## Verification

- Root and Dioxus Cargo metadata resolve every new path.
- Full workspace check passes after removing the legacy Tauri member.
- Workspace tests pass with 46 tests after skipping one pre-existing hanging
  `test_plugin_manager_lifecycle` test.
- Four PolyGlid Desktop domain tests pass.
- PolyGlid Desktop builds and launches from its new path.
- Workspace structure validation and diff checks pass.

## Known baseline issues

- Root formatting check exposes pre-existing unformatted Rust files.
- The plugin-manager lifecycle test hangs beyond 90 seconds when run unfiltered.
