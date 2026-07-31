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

## Development Environment

- Nix owns reproducible host-side developer tools and native development
  libraries through `flake.nix` and the committed `flake.lock`.
- Moon remains the repository task runner, Cargo remains the Rust build system,
  and GitHub Actions remains the CI and delivery platform.
- The development flake does not package the application. Native installers and
  release packages are a separate delivery concern.

### Nix implementation status (2026-08-01)

- The initial Nix development environment is complete and committed in
  `flake.nix` and `flake.lock`.
- It pins Rust 1.96.0 with `rustfmt` and the WebAssembly target, Moon 2.4.6,
  Dioxus CLI 0.7.9, Go, Git, jq, pkg-config, and the Linux GTK/WebKit
  libraries needed by Dioxus Desktop.
- Supported flake systems are x86_64 Linux, aarch64 Linux, and aarch64-darwin.
  Intel macOS is intentionally not declared because the current nixpkgs
  release no longer supports that target.
- Validation completed with tool-version checks, native-library checks, flake
  evaluation, and `cargo check --locked -p polyglid-desktop`.
- Nix is a development dependency layer only; it does not replace Moon, Cargo,
  Docker, or the existing GitHub Actions workflow.

### Next development steps

1. Add a lightweight CI job that runs `nix flake check` when the flake changes.
2. Design the installer/runtime-data layer separately for Linux, Windows, and
   macOS: config, data, cache, logs, plugins, workspaces, permissions, and
   PATH registration.
3. Implement idempotent first-run setup and versioned upgrade migrations.
4. Add uninstall behavior, signing, and update metadata after the installer
   layout stabilizes.

### Runtime directory foundation (2026-08-01)

- `polyglid-client::RuntimePaths` is the single source of truth for runtime
  directories and first-run initialization.
- `LocalClient::open_default` resolves `POLYGLID_DATA_DIR` and
  `POLYGLID_WORKSPACE_ROOT`, then creates config, cache, logs, plugins,
  reports, and workspace directories idempotently.
- Unix runtime directories are restricted to owner-only permissions. Windows
  uses the OS access-control model and resolves data under `%LOCALAPPDATA%`.
- macOS resolves data under `~/Library/Application Support/PolyGlid`; other
  Unix systems retain the existing `~/.polyglid` default.
- Installer-specific registration, signing, migration, and uninstall behavior
  remains separate from this runtime bootstrap foundation.
