# Development Commands

Run commands from the repository root.

## Desktop

```sh
cargo run -p polyglid-desktop
```

The desktop build requires the platform libraries used by Dioxus desktop and
WebView.

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

The equivalent Moon tasks are:

```sh
moon run polyglid:format
moon run polyglid:check
moon run polyglid:test
moon run polyglid:build
```

Moon orchestrates these tasks; Cargo remains the Rust build system.

Do not use `cargo clippy` unless a project clippy script is added and its usage
is documented here.
