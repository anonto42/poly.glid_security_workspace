# Step-By-Step Development Flow

This is the build path for PolyGlid. It turns the framework docs into an
ordered implementation loop so the project grows by tested vertical slices
instead of disconnected layers.

## Product North Star

PolyGlid is a cross-platform security workspace with a small trusted host and
sandboxed WebAssembly plugins. The trusted Rust host owns configuration,
permissions, scheduling, events, and UI integration. Plugins are untrusted by
default and can only use capabilities exposed by the host.

## Development Rules

- Build CLI-first. The GUI is not the first test harness.
- Grow through vertical slices: WIT contract -> plugin -> runtime -> CLI -> tests.
- Keep the core independent from Tauri, Wasmtime, filesystem, and UI details.
- Treat every plugin as untrusted, even first-party examples.
- Return structured data, never terminal text that the UI must parse.
- Record one-way decisions as ADRs before they become invisible assumptions.

## Phase 0: Planning Baseline

Goal: make the repo explain the system before implementation starts.

Steps:

1. Keep `README.md` aligned with the product shape.
2. Keep `docs/architecture/*` as the source for boundaries and crate ownership.
3. Keep `docs/security/SECURITY_MODEL.md` as the capability model source.
4. Keep `docs/planning/MVP.md` as the scope guard.
5. Add ADRs for hard-to-reverse choices.

Exit condition: a new contributor can explain the host/plugin boundary and MVP
without reading source code.

## Phase 1: Runtime Proof

Goal: run one harmless WASM component through a Rust CLI and print a structured
report.

Steps:

1. Create the Rust workspace and crates from `docs/architecture/CODEBASE.md`.
2. Add `contracts/polyglid.wit` with the first `plugin-report` contract.
3. Generate or mirror host/plugin API types in `polyglid-plugin-api`.
4. Implement `polyglid-runtime` as the Wasmtime adapter.
5. Implement `polyglid-cli` commands: `doctor`, `plugin inspect`, `plugin run`.
6. Build `plugins/recon_probe` as the first deterministic demo plugin.
7. Add tests for component loading, runtime errors, and report conversion.

Exit condition: `polyglid-cli plugin run ./path/to/recon_probe.component.wasm
--target example.com` executes without the GUI.

## Phase 2: Permission Model

Goal: make denied-by-default capability checks real before powerful plugins
exist.

Steps:

1. Define the `Capability` enum in core/API types.
2. Add plugin manifest metadata with requested capabilities.
3. Add permission decisions in `polyglid-core`.
4. Add runtime enforcement before any host capability is linked.
5. Add audit events for allowed, denied, and failed capability requests.
6. Add tests for denied network, file, process, and environment access.

Exit condition: a plugin cannot use a host capability unless the host grants it.

## Phase 3: Desktop Shell [COMPLETE]
 
 Goal: connect the proven engine to Tauri without moving product logic into UI
 handlers.
 
 Steps:
 
 1. [x] Create `apps/desktop`.
 2. [x] Add Tauri commands that call `polyglid-core` use cases.
 3. [x] Stream typed `polyglid-events` to the frontend.
 4. [x] Persist settings through `polyglid-config`.
 5. [x] Render the demo plugin report in the app.
 6. [x] Keep every command path runnable from CLI or tests.
 
 Exit condition: the desktop app can run the demo plugin and display its report.
 
 ## Phase 4: Workspace UI [COMPLETE]
 
 Goal: make the first operator workspace useful without hiding engine behavior.
 
 Steps:
 
 1. [x] Build the left utility navigation, central workspace panes, and status footer.
 2. [x] Add plugin list, run form, report viewer, and settings page.
 3. [x] Include loading, empty, denied, running, failed, and success states.
 4. [x] Keep dark-mode brand defaults from `docs/branding/README.md`.
 5. [x] Verify responsive behavior before adding richer interaction.
 
 Exit condition: the UI feels like a real security workspace, not a landing page.

## Phase 5: Plugin Families

Goal: add value only after sandboxing and permissions are proven.

Steps:

1. Add safe reconnaissance helpers.
2. Add banner/header analysis.
3. Add local defensive audit checks.
4. Add report generation and export.
5. Reject plugins that cannot express output through stable contracts.

Exit condition: richer plugins remain sandboxed, testable, and replaceable.

## Phase 6: Distribution

Goal: package the CLI, desktop app, and plugins in a repeatable release flow.

Steps:

1. Build optimized CLI binaries for supported targets.
2. Package Tauri app bundles/installers.
3. Define `.polyglid-plugin` packaging.
4. Add plugin signature/checksum validation.
5. Add update strategy and release notes.
6. Generate SBOMs and run supply-chain checks in CI.

Exit condition: another machine can install PolyGlid and run approved plugins.

## Agent And Knowledge Flow

Use `.agents/shared/` as the source of truth for assistant context. Keep core
instructions small, then load task-specific rules only when needed.

Graphify Labs describes Graphify as an open-source knowledge graph engine that
can turn code, docs, papers, meetings, and images into a traversable graph with
local or cloud deployment. For PolyGlid, use that idea as an optional
development accelerator:

- index `README.md`, `docs/`, `.agents/shared/`, `wit/`, `crates/`, `apps/`, and
  `plugins/`;
- use graph recall to find related contracts, rules, tests, and decisions;
- keep Markdown docs and agent files as the canonical human-editable source;
- never let a graph result override source code, tests, docs, or security rules.

## Definition Of Done

- Format, lint, type/borrow check, and tests pass for the touched layer.
- CLI path exists for plugin behavior before GUI wiring.
- Permissions are explicit and denied-by-default.
- Runtime/plugin failures do not crash the host.
- Docs or ADRs are updated when behavior or architecture changes.
- Agent memory is updated only with durable facts, never secrets or raw logs.
