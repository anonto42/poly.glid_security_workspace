# WPM Current State

Verified: 2026-07-15.

## Current implementation

- `projects/polyglid-desktop` is a Rust package using Dioxus Desktop 0.7.9.
- PolyGlid Desktop is a normal root Cargo workspace member. The nested workspace
  and lockfile were removed after retiring the React and Tauri clients.
- The Dioxus application now replaces all legacy dashboard surfaces at the UI layer:
  Explorer/targets, scanner, results, source, plugins, settings, command palette,
  problems/output/terminal, and runtime status.
- The same shell adds WPM work tracks, automation control, and AI-agent views.
- My Projects is now a real vertical slice. Core schema version 5 persists
  workspaces/projects, active selection, discovery status, exclusions, and timestamps.
- Dioxus loads the project catalog asynchronously through `WorkspaceCatalogService`,
  shows loading/empty/error/ready states, and supports create, rename, catalog removal,
  explicitly confirmed file deletion, refresh, and persisted workspace selection.
- Discovery scans direct child directories without following directory symlinks and
  classifies Rust, Node, Python, and general projects from their manifest files.
- Interactive preview behavior remains for targets/scanner results, plugin toggles,
  track filters/details, settings, automation, and agents.
- Domain features: serializable `WorkTrack`, `TaskStatus`, progress calculation,
  overview aggregation, and validated state transitions.
- SQLite is real for workspace/project catalog data. Real executor/plugin desktop
  adapters, Git sync, automation handlers, tracks persistence, and AI integration
  are still pending. Core/runtime remain canonical until their UI gates pass.

## Plugin integration boundary

- The desktop must not implement a second plugin engine. It should call application
  services backed by `polyglid-core::PluginManager` and `polyglid-runtime`.
- `PluginManager` already validates, installs, discovers, enables, and persists
  plugins through the core SQLite `WorkspaceStore`.
- Plugins implement the `security-tool` WIT world: metadata, required capabilities,
  typed execution reports, and host-rendered CLI/desktop panel layouts.
- The host grants scoped capabilities such as DNS resolution and report writing;
  plugins never receive unrestricted host access.
- Current `ui/preview.rs` plugin cards and scanner reports must eventually be
  replaced by typed core commands/results and events.

## Verification

- `cargo test -p polyglid-desktop --offline`: 4 tests pass.
- `cargo test -p polyglid-core workspace_catalog --offline`: 3 tests pass.
- `cargo check -p polyglid-core -p polyglid-events -p polyglid-desktop --offline`:
  passes.
- `git diff --check`: passes.
- The desktop launched on 2026-07-15 with isolated data and discovered 10 real
  project folders from the repository `projects/` directory.
- CSS is compiled into the binary with `include_str!`; plain `cargo run` does not
  depend on the Dioxus CLI asset pipeline.

## Manual UI dependency

The host is CachyOS (Arch-based). Dioxus Desktop linking requires `libxdo.so`,
which CachyOS provides through the `xdotool` package. Do not use Debian/Ubuntu
`apt-get` instructions on this host. A temporary extracted package was sufficient
for verification, but normal development should install the system package once.

Run manually:

```bash
sudo pacman -S --needed xdotool
cargo run -p polyglid-desktop
```

Use `projects/polyglid-desktop/README.md` for the visual checklist.

## Phase status

Gate 1 workspaces/projects is implemented as the first real vertical slice. It still
needs broader UI acceptance at all target window sizes before being marked closed.
Next, finish Gate 0 shared errors/platform paths and then integrate persisted plugin
management as Gate 2. Follow `polyglid-production-desktop-plan.md` in gate order.
