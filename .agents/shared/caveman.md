STATE: PolyGlid has a compiling Rust workspace, working runtime proof, and the
first Phase 2 permission model slice. CLI componentizes `recon_probe.wasm`, runs
`recon_probe.component.wasm` through Wasmtime, and prints structured plugin
reports.

TODAY: Added workspace crates, target validation, typed report/event models,
WIT host/guest bindings, WASI adapter componentization, Wasmtime execution, CLI
`doctor/config/plugin` commands, deterministic `recon_probe` logic, plugin
manifest parsing, capability display/parsing, denied-by-default permission
decisions, scoped capability requests, and allowed/denied/failed capability
audit events. Added `POLYGLID_CONFIG` persistent approval loading. Checks green.
Added WIT `dns.resolve` as the first host capability adapter, scoped to the run
target and gated by `dns-resolve`.
Added WIT `reports.write` as the second host adapter, scoped to `reports_dir`
and gated by `report-write`.
Added `scripts/run-mvp.sh` and `docs/development/MVP_RUNBOOK.md` so the current
CLI MVP can be run locally with one command.

NEXT: Continue Phase 2 by adding more host adapters only behind manifest
requests, approvals, and scoped runtime checks.

HANDOFFS: Use `docs/development/STEP_BY_STEP_DEVELOPMENT_FLOW.md` as the build
sequence. Use `.agents/shared/rules/polyglid-architecture.md` for Rust/WIT/
runtime/plugin work. Use `.agents/shared/rules/ai-context-management.md` for
agent memory and Graphify-style recall.

RISKS: WASI preview2 linker is available with default context. Custom
network/filesystem/process/env host functions are not implemented yet and must
stay blocked unless manifest requests and host approvals pass.

LOAD: `Cargo.toml`, `wit/polyglid.wit`, `crates/polyglid-core/src/lib.rs`,
`crates/polyglid-runtime/src/lib.rs`, `crates/polyglid-cli/src/main.rs`,
`plugins/recon_probe/src/lib.rs`, docs flow.
