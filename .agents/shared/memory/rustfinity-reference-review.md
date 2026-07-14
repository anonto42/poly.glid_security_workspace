# Rustfinity Reference Review

Verified: 2026-07-14.
Repository: `https://github.com/rustfinity/rustfinity.git`
Inspected commit: `1c563ec`.

## Scope correction

This repository is not the Rustfinity website or a desktop/Dioxus application. It
contains a Rust challenge catalog, a local/cloud CLI, a Docker-oriented code
runner, and a small AST testing crate. It cannot provide frontend component or
desktop-state architecture for WPM.

The root license permits internal evaluation but restricts modification and
derivative works. Some crate manifests declare MIT, creating ambiguity. Reuse
concepts only; do not copy code until licensing scope is clarified.

## Repository architecture

```text
challenges/                 strict exercise packages and metadata
crates/cli/                 download, submit, auth, init, link, deploy
crates/rustfinity-runner/   compile/test/run inside a Docker boundary
crates/syntest/             syntax/AST-oriented test helpers
cloud-examples/axum-api/    minimal generated deployment template
```

There are 146 challenge directories. Each follows a predictable contract:
`description.md`, `Cargo.toml`, solution/starter Rust, and integration tests.
Workspace tests validate required files, matching package names, unique metadata,
and timestamps.

## Execution flow

```text
Challenge package
 -> CLI/download or submitted code
 -> base64 transport
 -> runner writes temporary Cargo files
 -> cargo check/test/run
 -> merged compiler/test output
 -> optional timing and heap measurement
```

The documented Docker invocation disables networking and applies CPU/memory limits.
Rustlings execution uses a fresh temporary Cargo project. The runner itself does
not establish all isolation; the caller must launch the container safely.

## Patterns worth adopting

- Versioned, strict work-package layout with validation tests.
- Starter/input, instructions, expected checks, and evidence as separate artifacts.
- One typed CLI entry point with explicit subcommands.
- Embedded, versioned templates instead of AI-generated scaffolding.
- Temporary workspace per execution and explicit working directory.
- Separate `check`, `test`, and `run` operations with real exit status.
- Sandboxed code execution with no network and CPU/memory/time/output limits.
- AST/symbol checks to complement behavior tests.
- Structured metadata, unique IDs/slugs, and schema validation.
- Cross-platform build planning and explicit target compatibility.
- Authentication/device flow only when hosted collaboration is introduced.

## Patterns to improve or avoid

- Do not send code through URLs or command-line base64; use files/stdin or typed IPC.
- Do not infer correctness from merged terminal prose or ignore exit codes.
- Do not let arbitrary submitted `Cargo.toml` download dependencies during a run.
- Do not reuse a mutable shared playground directory between concurrent tasks.
- Do not parse benchmark paths/output with fragile regex/string indexing.
- Do not auto-install/update tools or initialize/commit Git without explicit approval.
- Do not store API keys as unprotected JSON; use OS credential storage later.
- Do not run user code directly on the host; Docker alone is also insufficient for
  high-risk hostile code without stronger sandbox policy.
- Do not couple catalog metadata, runner internals, UI state, and remote transport.

## Mapping to the unified platform

```text
Rustfinity concept       PolyGlid/WPM capability
challenge package     -> versioned task/work package
challenge metadata    -> task, criteria, inputs, permissions
CLI                    -> shared WPM CLI facade
runner                 -> sandbox execution capability
syntest                -> Rust AST/symbol analysis capability
structure tests        -> workspace/policy validation
submit/deploy          -> approval-gated artifact/release workflow
compiler output        -> structured evidence and diagnostics
```

## Platform features to build

### Dioxus control plane

- Workspace/project selector and health/status dashboard.
- Task tree, dependencies, acceptance criteria, comments, approvals, and activity.
- Source explorer and read-only file viewer first; editable workspace later.
- Task input/instruction panel and versioned artifact viewer.
- Run/check/test controls with permission and resource summaries.
- Live structured diagnostics, stdout/stderr, tests, timing, and artifact results.
- AI suggestions, code-analysis/security findings, patch preview, accept/dismiss.
- Git sync/offline/pending/conflict/quarantine status.

### Central execution engine

- Typed command/result contracts, task graph, attempts, cancellation, timeout,
  retry, recovery, approvals, capabilities, evidence, and audit/outbox.
- Execution profiles: workspace-safe, isolated build/test, and later hostile-code.
- Unique temporary directory/container per run with cleanup and artifact retention.
- Structured compiler/test parsers; terminal text remains display-only.

### Automation capabilities

- Workspace/package schema validation.
- Format, check, lint, build, unit/integration test, benchmark, and clean.
- Change/impact detection, templates, diagrams, reports, and release preparation.
- Resource limits, dependency allowlists/cache, toolchain pinning, and provenance.

### AI and analysis capabilities

- Exact/symbol/semantic search and workspace context.
- Code generation as patch artifacts.
- Code quality, dependency, security, and performance analysis.
- Test/document/task generation, failure explanation, and improvement suggestions.
- Rust AST rules using `syn`-style analysis where deterministic checks are possible.

## Suitable implementation order

1. Do not add a code editor or runner before Phase 0 baseline and shared contracts.
2. Build WPM domain/storage/executor and read-only source viewer.
3. Add safe local `cargo check/test` automation with structured evidence.
4. Add strict work-package metadata and validation.
5. Add isolated per-run execution and resource policies.
6. Add AST analysis, then read-only AI explanations and findings.
7. Add patch generation/editing only after worktree isolation and approvals.

## Current project phase

Architecture and roadmap planning are complete. No WPM implementation directory is
present yet. The next work is Phase 0 baseline, followed by Phase 1 contracts and
domain—not Dioxus components or code execution yet.
