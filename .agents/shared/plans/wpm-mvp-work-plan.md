# WPM MVP Work Plan

Status: architecture direction accepted; implementation intentionally deferred.
Updated: 2026-07-11.

Parent architecture:
`.agents/shared/plans/project-management-agent-platform.md`

## Product goal

Build WPM as a local-first project-management application for this workspace. It
manages projects, milestones, task trees, dependencies, acceptance criteria,
comments, approvals, audit history, and later workspace agents.

The MVP has no separate backend server. Every developer runs the Dioxus desktop
application with a private local SQLite database. A dedicated Git branch carries
immutable WPM events between developers.

WPM uses the shared central execution engine and capability contracts defined by
the parent architecture; it does not create a second orchestration system.

## Selected MVP stack

- UI/application shell: Dioxus Desktop, written in Rust.
- Async runtime: Tokio.
- Persistence: SQLx with SQLite and versioned migrations.
- Models/serialization: Serde, UUID, UTC timestamps.
- Errors/diagnostics: thiserror and tracing.
- Architecture: modular monolith with ports and adapters.
- Synchronization transport: Git remote and dedicated `wpm-data` branch.
- Styling: HTML/CSS rendered by the system WebView; application logic stays Rust.

Not selected for the MVP: Axum, PostgreSQL, Redis, queues, microservices,
WebSockets, a separate React/TypeScript frontend, or a shared SQLite file.

## Internal boundaries

```text
Dioxus UI
   -> application services/use cases
   -> domain rules
   -> repository and sync ports
   -> SQLite repository + Git event adapters
```

Dioxus components never execute SQL or Git commands directly. Domain and
application crates remain UI-independent so Axum or another client can be added
later without rewriting project rules.

## Local write flow

```text
User action in Dioxus
   -> validate command and permissions
   -> one SQLite transaction updates local state
   -> append outbox record with immutable event
   -> UI receives local state notification
   -> publisher exports pending event and commits it to wpm-data
```

The outbox guarantees that a UI change and its publishable event are recorded
together. A failed Git push must not lose the local change; it remains pending.

## Shared synchronization flow

```text
New commit on wpm-data
   -> client helper fetches/pulls data branch
   -> validate event files and schema version
   -> import unseen events idempotently into SQLite
   -> update local projections in one transaction
   -> store sync cursor/result
   -> notify Dioxus state
   -> render only affected components
```

Each commit indicates that WPM data is available. Clients may synchronize
manually, periodically, at startup, or through a background helper. This is
eventual collaboration, not instant real-time editing.

## Git event storage

Do not commit SQLite databases. Store one immutable file per event:

```text
.wpm/
  schema.json
  events/YYYY/MM/<event-uuid>.json
  snapshots/<optional-versioned-snapshot>.json
```

Minimum event envelope:

- Event UUID and schema version.
- Workspace/project/entity IDs and entity type.
- Operation and payload.
- Author/device ID.
- UTC occurrence time plus entity/base version.
- Causation/correlation IDs.
- Content hash and optional signature.

Use UUIDs and versions for identity/order; timestamps are display and diagnostic
data because developer clocks can disagree. Events are append-only. Updates and
deletions create new events rather than rewriting history.

## Branch and repository policy

- Use only the dedicated `wpm-data` branch for shared WPM event files.
- Never mix source-code changes into WPM data commits.
- Use a separate worktree or internal Git directory so syncing does not switch or
  modify a developer's active source branch.
- The helper stages only known WPM event paths and uses structured commit metadata.
- Fetch, validate, merge/rebase policy, push, retry, and conflict results are logged.
- Source branches can reference WPM task/event IDs in commits without containing
  the operational database.

## Consistency and conflict rules

- Maintain an `imported_events` table keyed by event UUID for idempotency.
- Maintain an outbox with pending, publishing, published, and failed states.
- Use optimistic entity versions to detect concurrent edits.
- Comments are naturally append-only and normally merge without semantic conflict.
- Conflicting task edits become explicit conflict records; never silently use the
  latest timestamp.
- Tree moves must prevent cycles and validate parent/base versions.
- Deletions use tombstone/archive events; destructive purge is a separate approval.
- Invalid, unknown, or unauthorized events are quarantined and shown to the user.

## Dioxus rendering

- Load the application shell first, then projects, tree nodes, and detail panels.
- Use signals/resources for local reactive state.
- After import, publish affected entity IDs instead of reloading the entire database.
- Paginate activity/comments and virtualize large trees.
- Show local, pending-publish, synchronized, conflict, and offline states clearly.
- Never interpolate untrusted comments as raw HTML; escape or sanitize content.

## MVP data model

- Workspace and project.
- Milestone and task/tree node.
- Task dependency and acceptance criterion.
- Comment and approval.
- Audit event, synchronization event, imported-event receipt, and outbox record.
- Conflict record and optional artifact link.

Use validated enums and state transitions rather than free-form status strings.
Every successful mutation writes an audit event.

## Implementation sequence

Accepted UI-first staging on 2026-07-15: finish the complete visual contract with
isolated preview data and manually verify every screen before connecting rules,
permissions, persistence, plugins, automation, or agent execution. Preview buttons
must remain clearly non-production and must not bypass the later service boundary.

1. Finish the modular Dioxus shell, design system, screens, responsive states, and
   manual visual checklist using isolated preview data.
2. Freeze the UI models/events that application services must provide.
3. Add SQLite migrations, repositories, state transitions, audit log, and outbox.
4. Replace preview data/actions with typed services one feature at a time.
5. Define versioned event schemas, exporter/importer, validation, and idempotency.
6. Add isolated `wpm-data` worktree, publisher, fetch/import helper, and sync UI.
7. Add conflict/quarantine handling, backup/restore, and end-to-end verification.
8. Add roles and a read-only workspace agent only after data and permission
   contracts stabilize.

## MVP verification

- Domain transition, tree-cycle, repository, and migration tests.
- Transactional outbox crash/retry tests.
- Duplicate, out-of-order, malformed, unauthorized, and conflicting event tests.
- Two-client synchronization test using separate SQLite files and one Git remote.
- Test that the active source branch/worktree remains unchanged during sync.
- Stored-content XSS, path traversal, secret-redaction, and event-size tests.
- End-to-end: create -> publish -> fetch -> import -> targeted UI refresh.

## Deferred roadmap

- Real-time multi-user collaboration and centralized PostgreSQL.
- Axum APIs, authentication/RBAC, WebSockets, notifications, and hosted operation.
- Desktop/mobile synchronization beyond Git.
- Attachments, ratings, calendars, boards, workload, and advanced reporting.
- Coding/testing/review agents with capabilities, worktrees, approvals, and evidence.

The event envelope and domain services should remain stable when Git transport is
later replaced or supplemented by Axum and a centralized event/database service.
