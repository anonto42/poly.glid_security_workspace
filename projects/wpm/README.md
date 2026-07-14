# WPM

Local-first Dioxus desktop control plane for PolyGlid workspace projects,
automation, and AI capabilities.

## Current developer-space shell

- Rust/Dioxus replacement shell for the legacy React/Tauri dashboard views.
- Explorer and targets, scanner configuration, result dashboard, source viewer,
  plugin registry, settings, command palette, and problems/output/terminal panels.
- Unified WPM work tracks, automation control, and AI-agent workspace views.
- Tested work-track model, state-transition policy, filters, progress, and metrics.

The current non-track dashboard data is an interactive UI preview. SQLite, real
plugin execution, Git synchronization, automation handlers, and AI execution are
later integration phases; the legacy Rust engine remains canonical until parity.

## Verify

```bash
cargo fmt --manifest-path projects/wpm/Cargo.toml -- --check
cargo test --manifest-path projects/wpm/Cargo.toml
cargo check --manifest-path projects/wpm/Cargo.toml --bin wpm
```

## Run on CachyOS/Arch Linux

Dioxus Desktop requires Linux WebView/GTK libraries. This machine already has the
WebKit/GTK dependencies but still needs `libxdo` for linking. On CachyOS/Arch,
the `xdotool` package provides that library:

```bash
sudo pacman -S --needed xdotool
cargo run --manifest-path projects/wpm/Cargo.toml
```

WPM temporarily uses an isolated Cargo workspace because Dioxus Desktop 0.7 and
the legacy Tauri desktop select incompatible Linux WebView dependency versions.
Remove the isolation after the Tauri frontend is retired or the upstream stacks
become compatible.

## Manual UI checklist

- Window opens with no terminal error.
- Rail navigation opens Explorer, Plugins, Work tracks, Automation, and AI agents.
- Scanner execution opens the interactive preview result and findings.
- Work-track filters and expandable track cards update correctly.
- Plugin enable/disable, settings tabs, command palette, and bottom tabs respond.
- The developer-space layout remains readable at the minimum window size.
