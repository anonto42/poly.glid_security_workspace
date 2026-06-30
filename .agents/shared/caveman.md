STATE: PolyGlid now has a compiling Rust workspace and a working runtime proof:
CLI componentizes `recon_probe.wasm`, runs `recon_probe.component.wasm` through
Wasmtime, and prints structured plugin reports.

TODAY: Added workspace crates, target validation, permission-store basics,
typed report/event models, WIT host/guest bindings, WASI adapter
componentization, Wasmtime execution, CLI `doctor/config/plugin` commands, and
deterministic `recon_probe` logic. Checks green.

NEXT: Start Phase 2 permission model: plugin manifest format, requested
capability parsing, and denied-by-default runtime enforcement before host
capabilities are exposed.

HANDOFFS: Use `docs/development/STEP_BY_STEP_DEVELOPMENT_FLOW.md` as the build
sequence. Use `.agents/shared/rules/polyglid-architecture.md` for Rust/WIT/
runtime/plugin work. Use `.agents/shared/rules/ai-context-management.md` for
agent memory and Graphify-style recall.

RISKS: WASI preview2 linker is available with default context, but PolyGlid
capability policy is not yet wired to per-plugin manifests. Do not expose custom
network/filesystem/process host functions until Phase 2 is complete.

LOAD: `Cargo.toml`, `wit/polyglid.wit`, `crates/polyglid-core/src/lib.rs`,
`crates/polyglid-runtime/src/lib.rs`, `crates/polyglid-cli/src/main.rs`,
`plugins/recon_probe/src/lib.rs`, docs flow.
