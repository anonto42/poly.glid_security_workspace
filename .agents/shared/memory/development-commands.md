# Development Commands

Use from the repo root unless noted.

## Current State

The Rust workspace exists with initial core/runtime/CLI/plugin crates. Wasmtime
Component Model execution is still pending.

## Rust Quality

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## CLI Harness

```bash
cargo run -p polyglid-cli -- doctor
cargo run -p polyglid-cli -- plugin list
cargo run -p polyglid-cli -- plugin inspect ./path/to/plugin.wasm
cargo run -p polyglid-cli -- plugin componentize ./path/to/plugin.wasm ./path/to/plugin.component.wasm
cargo run -p polyglid-cli -- plugin run ./path/to/plugin.component.wasm --target example.com
cargo run -p polyglid-cli -- config validate
```

## Plugin Build

The plugin crate compiles for the host during normal workspace checks. Build the
WASM module and componentize it before CLI execution.

```bash
rustup target add wasm32-wasip1
cargo build -p recon-probe --target wasm32-wasip1
cargo run -p polyglid-cli -- plugin componentize \
  target/wasm32-wasip1/debug/recon_probe.wasm \
  target/wasm32-wasip1/debug/recon_probe.component.wasm
```

## Desktop UI

Add Tauri commands once `apps/desktop` exists. The CLI remains the required
engine test harness.

## Agent Notes

- Read `.agents/shared/agent-startup.md` before coding.
- Load `.agents/shared/rules/polyglid-architecture.md` for Rust/WIT/runtime work.
- Load `.agents/shared/rules/ai-context-management.md` for agent/doc memory work.
- Append durable progress to today's `updates.md` when work spans sessions.
