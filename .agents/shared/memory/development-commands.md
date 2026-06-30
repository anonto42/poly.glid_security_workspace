# Development Commands

Use from the repo root unless noted.

## Current State

The Rust workspace exists with core/runtime/CLI/plugin crates. The CLI can
componentize and run the demo WASM plugin through Wasmtime. Phase 2 permission
model work has started with manifest parsing, scoped capability requests,
explicit capability decisions, persistent approval config, and denied-by-default
checks. The first concrete host adapter is WIT `dns.resolve`, scoped to the run
target.

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
cargo run -p polyglid-cli -- plugin run ./path/to/plugin.component.wasm --target example.com --allow dns-resolve
cargo run -p polyglid-cli -- plugin run target/wasm32-wasip1/debug/recon_probe.component.wasm --target localhost --allow dns-resolve
POLYGLID_CONFIG=config.example.toml cargo run -p polyglid-cli -- config validate
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
