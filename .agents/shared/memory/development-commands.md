# Development Commands

Run commands from the repository root.

## Development Environment

Enter the pinned development shell before running project commands:

```sh
nix develop
```

Verify its primary tools with:

```sh
rustc --version
rustc --print target-libdir --target wasm32-unknown-unknown
moon --version
dx --version
```

The flake currently supports x86_64 Linux, aarch64 Linux, and Apple Silicon
macOS. Intel macOS is not a supported Nix target in the pinned nixpkgs release.
If `nix develop` is unavailable because of a host sandbox restriction, use a
normal Nix installation on the host; the project configuration itself has been
validated independently.

The desktop runtime creates these directories on first run:

```text
data/              SQLite database and application-owned state
data/config/       persistent configuration
data/cache/        disposable caches
data/logs/         application logs
data/plugins/      installed plugin artifacts
data/reports/      generated reports
workspace/         default project workspace
```

Override the data and workspace roots with `POLYGLID_DATA_DIR` and
`POLYGLID_WORKSPACE_ROOT` when testing portable or isolated installations.

## Desktop

```sh
cargo run -p polyglid-desktop
```

The desktop build requires the platform libraries used by Dioxus desktop and
WebView.

## Website

Build the production GitHub Pages bundle with:

```sh
moon run website:bundle
```

The generated site is in
`target/dx/polyglid-website/release/web/public`.

## Verification

Run targeted package checks while iterating:

```sh
cargo check -p polyglid-desktop
cargo test -p polyglid-core
```

Before completing shared or cross-layer changes:

```sh
cargo fmt --all --check
cargo check --locked --workspace
cargo test --locked --workspace
```

Run the full CI sequence with:

```sh
scripts/ci/verify-rust.sh
```

The equivalent Moon tasks are:

```sh
moon run :format
moon run :check
moon run :test
moon run :build
```

Moon orchestrates these tasks; Cargo remains the Rust build system. CI uses
`moon ci --downstream deep` to run only affected tasks while also validating
every transitive consumer of a changed shared crate.

Do not use `cargo clippy` unless a project clippy script is added and its usage
is documented here.
