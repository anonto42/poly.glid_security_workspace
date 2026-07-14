# WPM Current State

Verified: 2026-07-14.

## Current implementation

- `projects/wpm` is now a real Rust workspace package using Dioxus Desktop 0.7.9.
- WPM temporarily uses its own nested Cargo workspace/lockfile because Dioxus 0.7.9
  and the legacy Tauri desktop require incompatible Linux `wry` versions. The root
  workspace excludes `projects/wpm` until Tauri retirement or upstream compatibility.
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
  Rust engine remains canonical until explicit parity and migration gates pass.

## Verification

- `cargo fmt --manifest-path projects/wpm/Cargo.toml -- --check`: passes.
- `cargo test --manifest-path projects/wpm/Cargo.toml --offline`: 4 tests pass.
- `cargo check --manifest-path projects/wpm/Cargo.toml --bin wpm --offline`: passes.
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
cargo run --manifest-path projects/wpm/Cargo.toml
```

Use `projects/wpm/README.md` for the visual checklist.

## Phase status

Implementation has started, but Phase 0 and Phase 1 are not fully complete. The
visual/domain slice is valid parallel work; next close the baseline gate and finish
versioned commands, events, results, permissions, and the complete domain model
before starting SQLite persistence.
