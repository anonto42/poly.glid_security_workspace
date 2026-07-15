# VS Code-style Workbench Shell

Date: 2026-07-15

## Implemented

- Added persistent top-level editor tabs for every rail destination.
- Added editor activation and close behavior, including last-tab protection.
- Added global shortcuts: command palette, quick open, editor destinations,
  sidebar, bottom panel, terminal panel, close editor, settings, and Escape.
- Replaced the static command list with fuzzy-filtered commands and keyboard
  selection.
- Added drag handles for sidebar width and bottom-panel height.
- Added collapse controls and persisted visibility/dimensions through
  `SettingsService` and the core settings store.
- Applied VS Code/GitHub-dark surface hierarchy and a 48px activity rail.
- Added reduced-motion handling, focus-visible states, accessible tab roles, and
  real workspace-derived status/notification content.
- Replaced the fake successful terminal transcript with an explicit reserved PTY
  state. Host command execution is not authorized by this UI phase.

## Verification

- `cargo check -p polyglid-desktop --offline` passed.
- `cargo test -p polyglid-desktop --offline` passed: 4 tests.
- `git diff --check` passed.
- Final Dioxus desktop binary launched successfully with isolated workspace data.

## Remaining

Typed event-driven toast history and an audited PTY/terminal core service remain
for later Gate 7 work. Feature screens still follow their individual gates.
