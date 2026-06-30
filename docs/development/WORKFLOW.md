# Development Workflow

Build PolyGlid in thin vertical slices. Each slice should cross the real boundary from host to plugin and back.

For the full phased build plan, see
[`STEP_BY_STEP_DEVELOPMENT_FLOW.md`](STEP_BY_STEP_DEVELOPMENT_FLOW.md).

## Local Development Order

1. Write or update the WIT contract.
2. Generate or update host/plugin bindings.
3. Implement the plugin.
4. Build the plugin as a WASM component.
5. Run it through `polyglid-cli`.
6. Add core/runtime tests.
7. Only then connect it to Tauri UI.

## Early Commands

Current commands:

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings

rustup target add wasm32-wasip1
cargo build -p recon-probe --target wasm32-wasip1
cargo run -p polyglid-cli -- plugin componentize \
  target/wasm32-wasip1/debug/recon_probe.wasm \
  target/wasm32-wasip1/debug/recon_probe.component.wasm
cargo run -p polyglid-cli -- plugin run \
  target/wasm32-wasip1/debug/recon_probe.component.wasm \
  --target example.com
cargo run -p polyglid-cli -- plugin run \
  target/wasm32-wasip1/debug/recon_probe.component.wasm \
  --target localhost \
  --allow dns-resolve \
  --allow report-write
POLYGLID_CONFIG=config.example.toml cargo run -p polyglid-cli -- config validate
```

## Testing Strategy

Unit tests:

- config validation
- permission decisions
- target validation
- event conversion

Integration tests:

- runtime loads component
- denied permission blocks execution
- plugin output maps into report events

Manual tests:

- CLI plugin run
- Tauri command invocation
- UI event rendering

## Development Rule

The GUI should never be the only way to test a plugin. Every plugin must be runnable through the development harness.
