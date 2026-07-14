# Product-Oriented Project Layout Migration

Date: 2026-07-14

## Changed

- Removed language grouping from `projects/`.
- Moved every Rust crate directly under `projects/<product-name>`.
- Renamed `wpm` to `polyglid-desktop` and its Cargo package/binary accordingly.
- Moved the WIT contract to `projects/polyglid-contracts`.
- Moved the plugin to `projects/recon-probe`.
- Preserved React and Tauri clients as `polyglid-web-legacy` and
  `polyglid-desktop-legacy` until Dioxus reaches engine parity.
- Moved the Rust config example into `projects/polyglid-config`.
- Updated Cargo paths, Make targets, automation, documentation, diagrams, project
  registry, and agent placement rules.

## Verification

- Root and Dioxus Cargo metadata resolve every new path.
- Workspace check excluding the known legacy Tauri build-script defect passes.
- Workspace tests pass with 46 tests after skipping one pre-existing hanging
  `test_plugin_manager_lifecycle` test.
- Four PolyGlid Desktop domain tests pass.
- PolyGlid Desktop builds and launches from its new path.
- Workspace structure validation and diff checks pass.

## Known baseline issues

- Full workspace check still hits the existing Tauri generated-permissions path bug.
- Root formatting check exposes pre-existing unformatted Rust files.
- The plugin-manager lifecycle test hangs beyond 90 seconds when run unfiltered.
