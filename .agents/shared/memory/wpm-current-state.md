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
- Interactive preview behavior includes target selection/addition, scanner results,
  plugin toggles, track filters/details, settings, and workspace navigation.
- Domain features: serializable `WorkTrack`, `TaskStatus`, progress calculation,
  overview aggregation, and validated state transitions.
- Non-track data remains seeded UI state. SQLite, real executor/plugin adapters, Git
  sync, automation handlers, and AI integration do not exist in WPM yet. The legacy
  Rust engine remains canonical until explicit integration gates pass.

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
- `cargo check -p polyglid-desktop --offline`: passes.
- `git diff --check`: passes.
- The unified developer-space shell and navigation were visually verified on
  2026-07-14.
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

Implementation has started, but Phase 0 and Phase 1 are not fully complete. The
visual/domain slice is valid parallel work; next close the baseline gate and finish
versioned commands, events, results, permissions, and the complete domain model
before starting SQLite persistence.
