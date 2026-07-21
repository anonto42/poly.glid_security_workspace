# MVP Runbook

> **Developer harness only.** This runbook still verifies the host/plugin
> boundary, but it no longer defines the product MVP. The desktop acceptance
> boundary is [UI-First MVP](../planning/MVP.md).

This is the current local MVP path. It runs the Rust CLI host, componentizes the
demo WASM plugin, grants only the required capabilities, and writes a report
under `reports/`.

## One Command

```bash
scripts/run-mvp.sh
```

Use a different authorized test target:

```bash
scripts/run-mvp.sh example.com
```

## Manual Commands

```bash
rustup target add wasm32-wasip1
cargo build -p recon-probe --target wasm32-wasip1
cargo run -p polyglid-cli -- plugin componentize \
  target/wasm32-wasip1/debug/recon_probe.wasm \
  target/wasm32-wasip1/debug/recon_probe.component.wasm
cargo run -p polyglid-cli -- plugin run \
  target/wasm32-wasip1/debug/recon_probe.component.wasm \
  --target localhost \
  --allow dns-resolve \
  --allow report-write
```

## What You Should See

- The CLI builds and componentizes `recon_probe`.
- The host grants `dns-resolve` and `report-write` only for this run.
- The plugin returns a structured report.
- A summary file appears under `reports/`.

## Harness Boundary

This path proves the trusted host, Wasmtime runtime, WIT contract, plugin
manifest, denied-by-default permission model, DNS host adapter, report-writing
host adapter, and CLI harness. It does not prove the desktop permission,
asynchronous execution, report history, or packaging journey.
