# Architecture Decisions

These decisions are current until the user explicitly changes them.

## Product Development

- Build one working phase at a time.
- Keep only files needed by the current phase and its real dependencies.
- Reintroduce future areas only when their development phase starts.

## UI

- Use Dioxus as the shipped UI technology across the desktop application and
  product website.
- The desktop uses Dioxus Desktop; the website uses Dioxus Web and produces a
  static GitHub Pages-compatible bundle.
- Treat Zed/GPUI as a design and methodology reference, not a second UI
  implementation.
- Keep business and security behavior outside the UI so a future approved UI
  decision does not require rewriting the domain layer.

## Layering

- Runnable application composition belongs in `apps/desktop`.
- Product website composition belongs in `apps/website`.
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
