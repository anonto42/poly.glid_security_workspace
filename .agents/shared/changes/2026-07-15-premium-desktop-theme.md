# Premium Desktop Theme

Date: 2026-07-15

## Direction

PolyGlid Desktop now uses the Obsidian Emerald theme: neutral graphite surfaces,
high-clarity typography, and emerald reserved for active/success states. Blue,
amber, and red communicate information, warnings, and errors.

## Implemented

- Centralized color, typography, radius, shadow, focus, motion, selection, and
  scrollbar tokens in `assets/theme.css`.
- Restyled the top bar, activity rail, sidebar, editor tabs, bottom panel, and
  status bar with stronger visual hierarchy and less border noise.
- Restyled scanner forms, metrics, result charts, plugin cards, work tracks,
  automation pipelines, agent workspace, settings, command palette, and menus.
- Added consistent elevation, rounded surfaces, hover motion, focus treatment,
  spacing, and text contrast.

## Verification

- Desktop format and check pass offline.
- Four desktop tests pass.
- Strict desktop Clippy passes.
- The rebuilt desktop launched without a startup error.

Manual visual approval remains with the user; adjust tokens rather than adding
isolated colors when feedback requires another theme iteration.
