# Unified WPM, AI, and Automation Platform

Status: canonical architecture plan.
Updated: 2026-07-12.

Implementation detail:
`.agents/shared/plans/wpm-mvp-work-plan.md`

Roadmap: `.agents/shared/plans/unified-platform-implementation-roadmap.md`

Source proposal review:
`.agents/shared/plans/wpm-source-proposal-comments.md`

## Objective

Build one Rust platform with several capabilities instead of separate WPM, AI,
and automation applications. WPM is the human control plane. A central execution
engine owns task orchestration. AI and deterministic automation are replaceable
capability adapters behind stable contracts.

## Architecture

```text
Dioxus WPM UI / CLI
        |
        v
Command + Query Facade
        |
        v
Central Execution Engine
  | task graph | state machine | policy | approvals | audit |
        |
        +--------+-----------+-----------+-----------+
        v        v           v           v           v
   Workspace  Automation   AI/RAG     Git Sync    Reports
   capability capability capability  capability  capability
        |        |           |           |           |
        +--------+-----------+-----------+-----------+
                         |
                         v
                SQLite + Event Outbox
                         |
                         v
                 wpm-data Git branch
```

## Ownership

### WPM control plane

- Projects, milestones, task trees, dependencies, criteria, comments, and status.
- User actions, plans, approvals, conflicts, and activity views.
- Local SQLite projections and Dioxus reactive state.
- Shows execution evidence; it does not implement build or AI logic.

### Central execution engine

- Accepts typed commands and creates versioned task runs.
- Resolves dependency graphs and schedules ready steps.
- Enforces state transitions, capabilities, path scopes, and approval policy.
- Invokes capability adapters and collects structured results/artifacts.
- Handles cancellation, timeout, retry, idempotency, and recovery.
- Writes audit events and transactional outbox records.
- Is UI-, model-, Git-, and tool-provider independent.

### Automation capability

- Workspace validation, project discovery, change detection, build, test, lint,
  formatting, packaging, diagrams, and release preparation.
- Uses typed Rust implementations and structured output.
- GNU Make remains a thin human shortcut; Bash remains only where OS integration
  makes it necessary.
- External writes, dependency changes, migrations, pushes, and deployment require
  explicit policy and approval.

### AI capability

- Workspace context, semantic/symbol search, analysis, suggestions, generation,
  security review, task decomposition, and summaries.
- Ollama is the default local provider; provider/model routing is configurable.
- AI output is a proposal or artifact, never automatic canonical state.
- Read-only AI arrives first. Writing agents require isolated worktrees, scoped
  capabilities, verification, and human approval.

### Git synchronization capability

- Publishes immutable WPM events through the isolated `wpm-data` branch.
- Imports unseen events idempotently into each developer's SQLite database.
- Never commits SQLite files or changes the active source branch/worktree.
- Provides eventual team collaboration; Axum/PostgreSQL real-time operation is
  deferred without changing domain or event contracts.

## Proposed Rust workspace shape

```text
projects/wpm/
  crates/
    wpm-domain/       # entities, value objects, state rules
    wpm-application/  # commands, queries, use cases, ports
    wpm-executor/     # task graph, scheduler, policy, run lifecycle
    wpm-storage/      # SQLx/SQLite, migrations, outbox, projections
    wpm-sync-git/     # event export/import and isolated Git worktree
    wpm-workspace/    # discovery, symbols, affected-project analysis
    wpm-automation/   # deterministic build/test/tool capabilities
    wpm-ai/           # providers, RAG, routing, feedback
    wpm-contracts/    # versioned command/event/result schemas
    wpm-app/          # Dioxus desktop composition root and UI
```

Keep crates only when boundaries need independent testing or dependency control.
Begin with fewer crates/modules if compilation overhead outweighs separation.

## Shared contracts

Every executable task contains:

- Task/run UUID, schema version, goal, kind, and priority.
- Dependencies, acceptance criteria, allowed paths/tools/capabilities.
- Risk level, approval requirements, timeout, and retry policy.
- Input artifact references and expected outputs.

Every result contains:

- Status, summary, evidence, artifacts, and affected entities/files.
- Commands/actions performed and duration.
- Verification results, warnings, errors, risks, and follow-up.
- Provider/tool identity and correlation/causation IDs.

Use enums and validated state transitions. Do not parse terminal prose to determine
success. Large outputs live as artifacts; events store references and hashes.

## Unified task lifecycle

```text
draft -> ready -> queued -> running -> review -> verified -> done
                    |          |          |
                    v          v          v
                 blocked     failed    rejected
```

Only the engine changes run state. Capabilities return results; WPM renders state.
Retry creates a new attempt under the same run, preserving prior evidence.

## Execution examples

### Workspace change

```text
Git change detected -> executor creates analysis run
-> deterministic diff/build checks -> optional AI summary
-> WPM notification/comment -> publish WPM event
```

### Agent implementation later

```text
Approved task -> isolated worktree -> coding capability writes patch
-> automation tests -> independent review capability
-> human approval -> controlled merge
```

## Persistence

- SQLite stores current local projections, runs, attempts, artifacts, approvals,
  audit events, outbox, import receipts, and conflicts.
- Immutable domain/integration events synchronize through Git in the MVP.
- Source code, tests, Git commits, and signed approvals remain canonical evidence.
- `.agents` stores curated architecture knowledge and plans, not operational task
  state or raw AI conversations.

## Migration from current code

1. Freeze new feature growth in the standalone `.workspace/ai/rust` CLI.
2. Define `wpm-contracts` commands, events, task results, and errors.
3. Build domain, storage, outbox, and execution lifecycle tests.
4. Move reusable workspace discovery and deterministic tools behind ports.
5. Move AI providers/RAG/features behind the AI capability interface.
6. Replace important Make/Bash logic with tested Rust automation handlers.
7. Compose capabilities in Dioxus WPM and keep compatibility CLI/Make adapters.
8. Remove duplicated legacy paths only after behavior and migration verification.

## Delivery phases

1. Foundation: contracts, domain, SQLite, audit/outbox, executor state machine.
2. WPM core: Dioxus projects/tasks/comments/criteria and incremental rendering.
3. Git sync: event schemas, isolated branch/worktree, import, conflict handling.
4. Automation: Rust validation/change/build/test handlers and evidence artifacts.
5. AI: read-only context/search/analysis with model routing and structured results.
6. Safe agents: scoped writing, worktrees, approvals, testing, independent review.
7. Collaboration: Axum, PostgreSQL, RBAC, WebSockets, hosted/mobile synchronization.

## Architecture gates

- Domain and executor tests run without Dioxus, Git, Ollama, or network access.
- All mutations are authorized, transactional, audited, and recoverable.
- Duplicate/reordered events and restarted runs are idempotent.
- Automation exit status and evidence determine success, never AI confidence alone.
- Active source worktrees remain untouched by WPM data synchronization.
- No writing agent is enabled before capability and approval enforcement passes.
