# Unified Platform Implementation Roadmap

Status: ready for implementation.
Updated: 2026-07-12.

Parent architecture:
`.agents/shared/plans/project-management-agent-platform.md`

## Delivery rule

Implement one vertical capability at a time. A phase finishes only after its code,
tests, migration/rollback path, documentation, and `.agents` knowledge are updated.
Keep current AI and Make behavior until its replacement passes parity checks.

## Phase 0 — Baseline and safety

- Record current commands, outputs, warnings, runtime files, and supported workflows.
- Add smoke fixtures for current workspace validation, change detection, AI status,
  analysis, security, generation, diagrams, releases, and Makefile targets.
- Define which current outputs are canonical, generated, cached, or disposable.
- Stop new features in `.workspace/ai/rust`; allow only fixes during migration.
- Establish formatting, linting, test, dependency, and security CI gates.

Gate: baseline report is reproducible, dirty work is preserved, and failures in the
legacy system are documented rather than silently accepted.

## Phase 1 — Workspace and contracts

- Create initial WPM project/modules with one binary and minimal crate boundaries.
- Add `wpm-contracts`: versioned IDs, commands, events, task requests/results,
  artifacts, evidence, errors, permissions, and schema compatibility rules.
- Add `wpm-domain`: projects, milestones, tasks, dependencies, criteria, comments,
  approvals, audit events, runs, attempts, conflicts, and validated state machines.
- Add serialization, validation, transition, cycle, and contract fixture tests.

Gate: domain/contracts compile without Dioxus, SQLite, Git, Ollama, or network.

## Phase 2 — SQLite persistence

- Add SQLx/SQLite configuration and versioned migrations.
- Implement repositories for all Phase 1 entities.
- Add append-only audit log, transactional outbox, import receipts, and conflicts.
- Add optimistic versions, transactions, backup/restore, and migration rollback.
- Use temporary databases for repository and migration tests.

Gate: create/update/restart recovery works; every mutation is atomic and audited.

## Phase 3 — Central execution engine

- Implement task graph, ready queue, run/attempt lifecycle, cancellation, timeout,
  retry, idempotency, dependency resolution, and crash recovery.
- Add capability registry and typed execution context.
- Enforce path/tool/capability scopes and approval requirements centrally.
- Store structured results, logs, evidence, and artifact references.
- Use fake capabilities for deterministic engine tests.

Gate: executor tests cover success, failure, retry, cancellation, restart, blocked
dependencies, invalid output, duplicate requests, and denied permissions.

## Phase 4 — Minimal Dioxus control plane

- Bootstrap config, SQLite, executor, capabilities, and graceful shutdown.
- Build project/task tree, task detail, criteria, comments, approvals, run history,
  conflicts, and activity views.
- Use Dioxus signals/resources and refresh only affected entities.
- Add loading, offline, pending, failed, and accessible keyboard states.
- Add CLI modes: default UI, `status`, `check`, `doctor`, `sync`, and `task run`.

Gate: a developer completes a task lifecycle after restarting the application,
without direct SQL and with visible audit evidence.

## Phase 5 — Git event synchronization

- Define immutable event envelope, JSON schema, hashes, versions, and quarantine.
- Export outbox events to an isolated `wpm-data` worktree and publish safely.
- Fetch/import unseen events idempotently without touching active source branches.
- Add optimistic conflict records, tombstones, retries, cursors, and sync status UI.
- Test two clients with separate SQLite databases and a temporary Git remote.

Gate: create -> publish -> fetch -> import -> targeted UI refresh succeeds under
duplicates, reordering, conflicts, offline recovery, and interrupted publication.

## Phase 6 — Deterministic workspace capability

- Implement project discovery, workspace configuration, source/symbol indexing,
  affected-project mapping, file ownership, and safe path resolution.
- Return structured results with source references and hashes.
- Exclude caches, dependencies, targets, secrets, and generated artifacts by policy.

Gate: fixtures prove stable discovery and no reads outside permitted workspace roots.

## Phase 7 — Automation migration

Migrate and verify in this order:

1. Workspace validation and doctor checks.
2. Git/project change detection.
3. Format, lint, compile/check, and test execution.
4. Build and clean operations.
5. Configuration and project-template generation.
6. Diagrams, reports, and release-manifest generation.
7. Docker/Kubernetes/release preparation; external application remains approval-only.

Each handler uses argument arrays, restricted working directories, timeouts, output
limits, cancellation, and structured exit evidence. Make targets become thin `wpm`
wrappers only after parity tests pass.

Gate: new handlers match or improve legacy behavior on success and failure paths.

## Phase 8 — AI foundation

- Move provider traits, Ollama/OpenAI-compatible client, configuration, cache, and
  model routing into the WPM AI capability.
- Add health checks, timeouts, redaction, context limits, typed parsing, and fallback.
- Replace fixed chunks with symbol-aware retrieval plus exact search and dependencies.
- Separate local deterministic scoring from model-generated conclusions.
- Record provider/model, retrieved sources, prompt version, and parse confidence.

Gate: offline/provider/invalid-JSON/timeout tests pass and no secret enters prompts.

## Phase 9 — Read-only AI features

Migrate in increasing-risk order:

1. Workspace status and semantic/symbol search.
2. Suggestions and summaries.
3. Code and dependency analysis.
4. Security review combined with deterministic evidence.
5. Build optimization recommendations.
6. Test, documentation, diagram, and release-plan generation as artifacts.

AI results create proposals/findings, never automatic task completion or source edits.

Gate: every result cites evidence, is dismissible, and has benchmark fixtures.

## Phase 10 — Code generation and safe writing

- Generate patch artifacts rather than overwriting source files.
- Add dedicated source worktree per run, path locks, capability scopes, and quotas.
- Require approval before applying patches, installing dependencies, or external use.
- Run formatting, compile, targeted tests, regression tests, security checks, and an
  independent review before merge approval.
- Preserve rollback, patch provenance, and all failed attempt evidence.

Gate: adversarial permission, prompt-injection, conflict, rollback, and interrupted
run tests pass; agents cannot modify active worktrees or unapproved paths.

## Phase 11 — Specialized agents

- Coordinator: plan, dependency graph, delegation, final evidence summary.
- Workspace: read-only discovery and affected-component analysis.
- Coding: scoped patch production only.
- Testing: targeted/regression execution and diagnosis.
- Review/security: independent acceptance and risk assessment.
- Documentation/release: approved artifact generation.

Gate: agents communicate only through versioned task/result contracts, and no agent
can approve its own high-risk output.

## Phase 12 — Cleanup and collaboration

- Compare all migrated commands and generated artifacts against the baseline.
- Remove legacy AI/Make implementations only when unused and fully recoverable.
- Move disposable build/cache data outside `.workspace` knowledge storage.
- Later add Axum, PostgreSQL, RBAC, WebSockets, hosted sync, and mobile clients while
  retaining domain, executor, capability, and event contracts.

Gate: clean install, upgrade, backup/restore, two-client sync, full tests, and user
acceptance pass; architecture and operator documentation are complete.

## Verification on every phase

```text
format -> compile/check -> unit tests -> integration tests
-> security/static checks -> smoke test -> migration/recovery test
-> evidence review -> docs/.agents update
```

Do not begin a dependent phase while its prerequisite gate is failing. Small UI
work may proceed alongside backend phases only against stable versioned contracts.
