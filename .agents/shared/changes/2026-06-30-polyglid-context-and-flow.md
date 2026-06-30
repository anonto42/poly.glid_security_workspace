# PolyGlid Context And Development Flow

## Status

Completed.

## What Changed

- Added `docs/development/STEP_BY_STEP_DEVELOPMENT_FLOW.md`.
- Linked the step-by-step flow from `README.md` and
  `docs/development/WORKFLOW.md`.
- Replaced stale HRMS memory in `.agents/shared/memory/` with PolyGlid project
  facts, commands, architecture defaults, and security defaults.
- Updated `.agents/shared/agent-startup.md` hard rules for PolyGlid.
- Added `.agents/shared/rules/polyglid-architecture.md`.
- Added `.agents/shared/rules/ai-context-management.md`.
- Added `.agents/shared/coders/rust-coder.md`.
- Updated `.agents/shared/caveman.md` to the current PolyGlid state.

## Why

The docs define PolyGlid as a Rust/Tauri security workspace with a trusted host,
WIT contracts, Wasmtime runtime, CLI-first validation, and sandboxed WASM
plugins. The previous shared agent memory still described a Nest/Next HRMS app,
which would mislead future agents.

## Graphify Note

Graphify is useful as a design inspiration for local code/docs recall: index the
repo, traverse relationships, and use results to find relevant source files.
It is not a runtime dependency or replacement for source-code verification.

## Follow-Up

- Scaffold the Rust workspace in the phase order from the new development flow.
- Add ADRs when choosing exact WIT/component tooling versions.
- Keep `.agents/shared/` small, task-routed, and free of secrets/raw logs.

## Development Started

On 2026-06-30, the Rust workspace scaffold was added with initial crates,
manual CLI parsing, core permission/runtime traits, WIT host/guest bindings,
WASI componentization, Wasmtime execution, and `recon_probe` demo logic. `cargo
fmt --all -- --check`, `cargo check --workspace`, `cargo test --workspace`,
`cargo clippy --workspace -- -D warnings`, `polyglid doctor`, `polyglid config
validate`, and real component plugin runs passed.
