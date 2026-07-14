# Extensible Desktop Top Bar

Date: 2026-07-15

## Implemented

- Split the top bar into brand/workspace, command center, extension actions,
  runtime status, notifications, and user controls.
- Added data-driven `TopBarAction` preview descriptors to shared UI state.
- Added trusted extension actions that navigate to Plugins and Automation.
- Added interactive workspace selection, command-palette entry, notification
  panel, runtime settings entry, and profile/settings entry.
- Added responsive hiding rules for compact desktop widths.

## Boundary

The extension slot renders host-owned components from typed descriptors. Future
plugins may contribute approved action metadata, but they must never inject raw
HTML or arbitrary Dioxus code into the shell.

## Verification

- Desktop format and check pass offline.
- Four desktop tests pass.
- Strict desktop Clippy passes.
- The desktop launched with no startup error after the redesign.
