# PolyGlid GPUI Workbench

This crate is the native GPU client migration track for PolyGlid. It uses the
Apache-2.0-licensed `gpui` crate and PolyGlid's existing client gateway and
feature controllers. The interface and component implementation are original
to PolyGlid; Zed's GPL workspace and UI crates are not copied.

The current visual milestone provides:

- a compact editor-first title bar, repository and branch chips, large editor
  canvas, right-side workspace explorer, bottom utility bar, and command
  palette;
- typed keyboard actions for navigation and shell controls;
- live bootstrap data from `LocalClient` through `DesktopControllers`;
- a safe workspace-root listing that rejects traversal and out-of-root paths;
- a refresh command that reloads the application snapshot and explorer;
- unit-tested tab and navigation state.

Run it during the migration with:

```bash
cargo run --manifest-path apps/workbench/Cargo.toml
```

To open this repository itself in an isolated development profile:

```bash
POLYGLID_DATA_DIR=/tmp/polyglid-workbench-dev \
POLYGLID_WORKSPACE_ROOT="$PWD" \
cargo run --manifest-path apps/workbench/Cargo.toml
```

The Dioxus `polyglid-desktop` binary remains the packaged production client
until the GPUI workbench reaches feature and platform parity.

The complete section, tab, code-editor, terminal, settings, overlay, state, and
controller plan is maintained in
[`docs/architecture/WORKBENCH_COMPONENT_MAP.md`](../../docs/architecture/WORKBENCH_COMPONENT_MAP.md).
