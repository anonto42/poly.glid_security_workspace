# Architecture Decisions

These decisions are current until the user explicitly changes them.

## Product Development

- Build one working phase at a time.
- Keep only files needed by the current phase and its real dependencies.
- Reintroduce future areas only when their development phase starts.

## UI

- Use one shipped UI technology for the desktop application.
- The current implementation uses Dioxus for cross-platform desktop support.
- Treat Zed/GPUI as a design and methodology reference, not a second UI
  implementation.
- Keep business and security behavior outside the UI so a future approved UI
  decision does not require rewriting the domain layer.

## Layering

- Runnable application composition belongs in `apps/desktop`.
- UI-independent controllers and models belong in `crates/client`.
- Domain, persistence, security, and runtime behavior belongs in reusable
  crates.
- Dependencies flow from the app toward crates, never from crates toward the
  app.

## References

- `reference/legacy-main` and `reference/zed-base` are retained for information
  and ideas.
- Copy concepts selectively; do not merge either reference branch wholesale
  into `main`.
