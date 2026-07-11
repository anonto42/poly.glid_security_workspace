# Project Memory

## Shape

- Product: PolyGlid Security Workspace.
- Shape: Rust workspace plus Tauri desktop frontend.
- Host: trusted Rust engine with hexagonal boundaries.
- Plugins: untrusted WebAssembly components behind WIT contracts.
- First client: `polyglid-cli` development harness.
- Later client: `apps/desktop` Tauri workspace UI.
- Runtime: Wasmtime Component Model.
- Contract source: `wit/polyglid.wit`.

## References

- Architecture: `docs/architecture/`
- Framework: `docs/framework/`
- Security model: `docs/security/SECURITY_MODEL.md`
- MVP: `docs/planning/MVP.md`
- Roadmap: `docs/planning/ROADMAP.md`
- Step flow: `docs/development/STEP_BY_STEP_DEVELOPMENT_FLOW.md`
- Workflow: `docs/development/WORKFLOW.md`
- Packaging: `docs/development/PACKAGING.md`
- Brand: `docs/branding/README.md`
- Rules: `.agents/shared/rules/`
- Scopes: `.agents/shared/scopes/`
- Coders: `.agents/shared/coders/`
- Commands: `.agents/shared/memory/development-commands.md`
- Current workspace AI status: `.agents/shared/memory/workspace-ai-current-state.md`
- Project-management agent plan: `.agents/shared/plans/project-management-agent-platform.md`
- Daily history: `.agents/shared/history/YYYY-MM-DD/`

Read the smallest matching doc set before coding. Do not load every rule file by
default.

## Core Architecture

- `polyglid-core` owns product behavior, policies, permissions, orchestration.
- `polyglid-runtime` owns Wasmtime loading/linking/execution.
- `polyglid-plugin-api` owns shared WIT-generated or mirrored types.
- `polyglid-config` owns config schema, validation, persistence.
- `polyglid-events` owns typed host/UI/runtime events.
- `polyglid-cli` owns development commands and harness behavior.
- `apps/desktop` owns Tauri commands and frontend workspace UI only.

## Security Defaults

- Plugins are untrusted by default.
- Sensitive capabilities are denied by default.
- Plugins cannot directly access filesystem, processes, environment, or raw
  network sockets.
- Host/plugin data crosses stable WIT contracts as structured values.
- Security features must support authorized testing and defensive diagnostics.

## Knowledge Graph Note

Graphify-style local knowledge graphs can help agents find related code, docs,
contracts, tests, and decisions. Treat graph results as recall hints only; source
files, tests, docs, and security rules remain canonical.
