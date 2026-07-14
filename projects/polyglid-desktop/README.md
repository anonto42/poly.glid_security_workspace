# PolyGlid Desktop

Local-first Dioxus desktop control plane for PolyGlid workspace projects,
automation, and AI capabilities.

## Current developer-space shell

- Rust/Dioxus developer-space shell and the single active desktop UI.
- Explorer and targets, scanner configuration, result dashboard, source viewer,
  plugin registry, settings, command palette, and problems/output/terminal panels.
- Unified WPM work tracks, automation control, and AI-agent workspace views.
- Tested work-track model, state-transition policy, filters, progress, and metrics.

The current non-track dashboard data is an interactive UI preview. SQLite, real
plugin execution, Git synchronization, automation handlers, and AI execution are
later integration phases; the existing Rust engine remains canonical.

## UI module ownership

- `src/main.rs` launches the Dioxus desktop window only.
- `src/ui/app.rs` composes the full developer-space shell.
- `src/ui/state.rs` owns shared interactive UI state.
- `src/ui/models.rs` owns view, tab, filter, plugin, and report models.
- `src/ui/shell.rs`, `sidebar.rs`, `editor.rs`, `bottom_panel.rs`, and
  `overlays.rs` own the persistent workspace regions.
- `src/ui/features/` contains independently controlled scanner, plugin, track,
  automation, and agent screens.
- `src/ui/components.rs` contains reusable visual controls.
- `src/ui/preview.rs` isolates temporary seeded data from UI components.

## Verify

```bash
cargo fmt --all -- --check
cargo test -p polyglid-desktop
cargo check -p polyglid-desktop
```

## Run on CachyOS/Arch Linux

Dioxus Desktop requires Linux WebView/GTK libraries. This machine already has the
WebKit/GTK dependencies but still needs `libxdo` for linking. On CachyOS/Arch,
the `xdotool` package provides that library:

```bash
sudo pacman -S --needed xdotool
cargo run -p polyglid-desktop
```

## Manual UI checklist

- Window opens with no terminal error.
- Rail navigation opens Explorer, Plugins, Work tracks, Automation, and AI agents.
- Scanner execution opens the interactive preview result and findings.
- Work-track filters and expandable track cards update correctly.
- Plugin enable/disable, settings tabs, command palette, and bottom tabs respond.
- The developer-space layout remains readable at the minimum window size.
