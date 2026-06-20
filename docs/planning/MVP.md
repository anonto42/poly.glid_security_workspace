# MVP Definition

The first useful version of PolyGlid should prove the architecture before it tries to become a full security suite.

## MVP Goal

Run one sandboxed WASM plugin from a Rust host, receive structured output, and show that output through a development CLI. The GUI can come after this boundary works.

## MVP Scope

Included:

- Rust workspace
- WIT contract
- plugin API types
- Wasmtime runtime adapter
- CLI development harness
- one safe diagnostic plugin
- config schema draft
- permission/capability model draft
- tests for core permission decisions

Not included yet:

- full Tauri UI
- plugin marketplace
- real exploit modules
- mobile builds
- complex multi-window behavior
- native installers

## First Plugin

The first plugin should be harmless and deterministic. Good candidates:

- target validator
- HTTP header fetcher for an explicitly provided URL
- local demo report generator with no network access

Recommended first plugin: `recon_probe`.

It should accept:

```text
target: string
```

It should return:

```text
plugin_name
target_tested
status
issues
recommendations
```

## First CLI Commands

```bash
cargo run -p polyglid-cli -- plugin inspect ./path/to/plugin.wasm
cargo run -p polyglid-cli -- plugin run ./path/to/plugin.wasm --target example.com
cargo run -p polyglid-cli -- config validate
```

## MVP Completion Checklist

- `cargo test` runs for core crates.
- CLI can load a compiled WASM component.
- CLI can reject a plugin when permissions are missing.
- CLI can run the demo plugin and print structured output.
- Runtime errors do not crash the host process.
- Docs explain how to build and run the MVP.

