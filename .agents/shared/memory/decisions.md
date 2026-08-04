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

1. Done: `.github/workflows/ci.yml` runs a `nix` job (`nix flake check
   --no-build`) on every push and pull request.
2. Done (2026-08-01): Linux AppImage packaging implemented and locally
   verified end-to-end. `scripts/release/package-appimage.sh` builds a
   `PolyGlid.AppDir`, fetches `appimagetool` if missing, and produces the
   `.AppImage`; `.github/workflows/release.yml`'s Linux leg packages and
   uploads it as a separate `release-linux-x86_64-appimage` artifact
   alongside the existing tar.gz so `if-no-files-found: error` does not break
   the Windows/macOS legs. `scripts/release/publish.sh` needed no changes.
   Scope is Linux-only for this phase (Windows/macOS installer work deferred
   until real testing capacity exists); no `.deb`/`.rpm`/Flatpak; AppImage
   ships unsigned since AppImage does not require OS-level signing to run.
   AppImage now includes AppStream metadata, runtime-directory guidance, and a
   deterministic AppDir validation step. Still open: real app icon/branding
   (placeholder in place, marked with a `TODO` in the script).
3. Done (2026-08-04): idempotent first-run setup and versioned migration
   reporting are implemented in the client and surfaced by desktop startup.
4. Add uninstall behavior, signing, and update metadata after the installer
   layout stabilizes. Revisit Windows/macOS installer scope and signing when
   that work actually starts.

   The current Windows ZIP and macOS Unix archive now bundle runtime-directory
   guidance and run package-content validation. Windows MSI registration and
   macOS DMG packaging are now defined; signing and uninstall behavior remain
   intentionally deferred.

   Windows MSI packaging is now defined with WiX v4 and included as a separate
   release artifact. It installs the executable and runtime guidance, creates a
   Start Menu shortcut, and supports major upgrades. Signing and actual hosted
   Windows validation remain prerequisites before calling it production-ready.

   macOS DMG packaging is now defined for Apple Silicon. It creates a standard
   `.app` bundle with `Info.plist`, runtime guidance, and an Applications alias.
   Signing, notarization, and actual hosted macOS validation remain open.

   The temporary release `workflow_dispatch` bypass commits were reverted on
   2026-08-04. Production release builds are again gated by a successful
   Release Please result.

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

### Versioned first-run setup (2026-08-04)

- `LocalClient::open_with_setup` and `open_default_with_setup` expose a
  `SetupReport` without changing the existing `open` APIs.
- Setup reports `FirstRun`, `Migrated`, or `Ready`, records directories created,
  and reports the database version transition and applied migration versions.
- Runtime directory creation and SQLite migrations remain transactional and
  idempotent; opening an existing installation does not recreate directories
  or reapply migrations.
- Future installer work should call this setup boundary after installation and
  surface its errors to the user before opening the desktop shell.
- The Dioxus desktop startup now calls the setup-aware API and surfaces first-run
  or migration notices in the existing Projects status area. Setup failures
  remain blocking startup errors with recovery guidance.
