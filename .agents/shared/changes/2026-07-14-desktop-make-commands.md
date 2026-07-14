# Desktop Make Commands

Date: 2026-07-14

The default development command is now `make dev`, which runs the standalone
PolyGlid Dioxus desktop application. Clear explicit commands are also available:

- `make desktop` — run the desktop app
- `make server` — run the optional backend API
- `make desktop-build` — build the desktop app
- `make desktop-test` — test the desktop app

The former `wpm-*` targets remain compatibility aliases. The backend is not
required for the current local desktop preview.

The shared help target was also corrected to display command names instead of
their source Makefile names.
