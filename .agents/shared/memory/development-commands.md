# Development Commands

Run commands from the repository root.

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
