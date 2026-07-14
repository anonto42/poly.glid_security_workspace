# WPM Dioxus Foundation

Date: 2026-07-14

## What changed

- Created the `projects/wpm` Rust/Dioxus package.
- Added initial domain rules and four passing tests.
- Added an interactive, responsive work-tracks control-plane screen.
- Documented Linux prerequisites and a manual visual test checklist.
- Isolated WPM's Cargo workspace/lockfile to prevent Dioxus WebView dependencies
  from downgrading and breaking the legacy Tauri desktop dependencies.
- Embedded the stylesheet in the binary so the UI works through plain `cargo run`
  without requiring Dioxus CLI asset collection.

## Verification

Formatting, WPM unit tests, binary type-checking, and diff checks pass. The host
was identified as CachyOS; desktop launch requires its `xdotool` package, which
provides `libxdo.so`. The earlier Debian `libxdo-dev` instruction was corrected.
The desktop app was launched afterward and the styled work-track screen was
visually verified.

## Next

Close Phase 0 baseline and complete Phase 1 versioned contracts/domain before
adding SQLite repositories.
