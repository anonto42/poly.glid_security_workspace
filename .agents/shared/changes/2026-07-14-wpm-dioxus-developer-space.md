# WPM Dioxus Developer Space

Date: 2026-07-14

## Change

- Replaced the single work-track screen with a unified Dioxus desktop shell.
- Recreated the legacy dashboard surfaces: Explorer/targets, scanner, results,
  source viewer, plugin registry, settings, and problems/output/terminal panels.
- Added WPM work tracks, automation control, AI-agent workspace, command palette,
  and shared runtime/status navigation.
- Added interactive seeded state for navigation, targets, plugin toggles, scanner
  preview results, track filters/details, settings, and panel switching.
- Set the desktop title, useful initial/minimum sizes, and removed the default menu.

## Boundary

This completes UI scaffolding only. Seeded preview data must be replaced through
versioned contracts and adapters. The legacy Rust engine remains canonical until
plugin execution, persistence, automation, and AI parity are tested.

## Verification

- Formatting passes.
- Dioxus binary check passes offline.
- Four domain tests pass.
- Desktop rendering and navigation were manually verified on CachyOS.
