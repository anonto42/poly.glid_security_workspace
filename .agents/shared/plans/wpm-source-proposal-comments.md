# WPM Source Proposal — Review Comments

Reviewed: 2026-07-11

Source: user-provided “Workspace Project Manager” proposal (dashboard, node tree,
comments, feedback, REST/WebSocket API, PostgreSQL, Redis, and Docker).

Status: useful product input, not implementation-ready specification.

## What to keep

- Dashboard, hierarchical work view, comments, replies, feedback, search, and
  real-time updates are coherent WPM product capabilities.
- Rust, Axum, Tokio, Serde, UUIDs, PostgreSQL, and WebSocket fit this workspace.
- The proposed project/config/infrastructure separation broadly matches current
  repository placement.
- Project tree and discussion features can become the human-facing layer over the
  coordinator and agent-task system.

## Required scope changes

- A node tree alone is not enough for development management. Add milestones,
  task dependencies, acceptance criteria, agent runs, artifacts, approvals, and
  immutable audit events from the active agent-platform plan.
- Separate filesystem/component nodes from work-item tasks. They have different
  lifecycles and permissions even if the UI can display them in one tree.
- Add task transition rules rather than accepting arbitrary status strings.
- Add workspace/repository identity so WPM can manage more than one project tree.
- Comments and ratings are collaboration features; they must not serve as formal
  approvals. Approvals need actor, scope, evidence, decision, and timestamp.

## Database comments

- Choose PostgreSQL for the collaborative service, or SQLite for a local-first
  MVP; do not maintain multiple unexplained `polyglid.db` copies as canonical data.
- `users` must exist before foreign keys reference it. Migration order and rollback
  behavior must be explicit.
- `projects.root_node_id` needs a deferred foreign key or should be derived; the
  proposal leaves it unconstrained.
- `ON DELETE CASCADE` on `nodes.parent_id` can delete an entire subtree. Require
  archive/soft-delete or explicit destructive confirmation.
- `parent_comment_id` should define deletion behavior and prevent cross-node reply
  relationships.
- Replace free-form type/status/category strings with validated enums or database
  constraints.
- Add optimistic concurrency (`version`) to prevent silent lost updates.
- Add tenant/workspace scope if multi-user operation is selected.
- Add `audit_events`, `approvals`, `agent_runs`, `artifacts`, task dependency, and
  acceptance-criterion tables before agent automation.
- For `ltree`, define safe labels and transactional subtree moves; moving a node
  must update all descendant paths atomically and reject cycles.

## API comments

- Prefer `PATCH` for partial updates and return consistent typed error envelopes.
- Add pagination, sorting, filtering, idempotency keys, and optimistic concurrency.
- Destructive endpoints need authorization, audit events, and confirmation policy.
- WebSocket connections require authenticated upgrade, project authorization,
  resume/reconnect semantics, event IDs, and backpressure handling.
- Broadcast only authorized project events; a global Tokio broadcast channel can
  leak cross-project data.
- Define OpenAPI and event schemas before frontend implementation.

## Security comments

- Never ship `admin/admin123`, plaintext repository credentials, or secrets in
  Compose files. Use environment/secret injection and first-run account setup.
- Passwords need Argon2id or an approved identity provider, secure sessions, CSRF
  protection where cookies are used, rate limiting, and account recovery policy.
- Attachments require size/type limits, malware handling, private object storage,
  authorization, and safe download headers.
- Render comments as escaped text or sanitized Markdown to prevent stored XSS.
- Agent actions require capability-based permissions and human gates for network,
  dependency, Git push, migration, release, and deployment operations.

## Frontend comments

- The HTML/JavaScript samples are conceptual; inline handlers and interpolated
  `innerHTML` are unsafe for untrusted node/comment content.
- Choose the UI architecture before implementation: existing Tauri/React client,
  a new React web application, or server-rendered HTMX. Avoid maintaining two
  frontends in the MVP.
- Tree drag-and-drop needs keyboard controls, screen-reader semantics, loading and
  failure states, cycle prevention, and server-confirmed rollback.
- Use a virtualized tree for large workspaces.

## Infrastructure and Makefile comments

- Redis, Nginx, a separate frontend container, and a message queue are premature
  until load or deployment requirements justify them.
- The current root `wpm-build` target hides build failure with `|| printf`; it must
  fail loudly in CI and normal verification.
- `init-wpm` currently depends on an AI generation flag that is not present in the
  inspected CLI contract. Scaffolding should use versioned templates.
- Docker should use pinned images, health checks, non-root users, internal networks,
  persistent-volume policy, and no default credentials.
- Build artifacts should not accumulate inside `.workspace/ai/rust/target`.

## Recommended MVP

1. Local-first single workspace and one administrator identity.
2. Projects, milestones, tasks, dependencies, criteria, comments, and audit events.
3. Rust/Axum API with one canonical database and migrations.
4. CLI plus one dashboard interface.
5. Read-only workspace agent connected through structured task/result contracts.
6. Human approval UI and capability policy before enabling writing agents.
7. Targeted tests for state transitions, permissions, tree moves, and audit history.

## Decisions required next

- PostgreSQL collaborative MVP or SQLite local-first MVP.
- Existing Tauri/React UI or separate web dashboard.
- Single-user first or immediate multi-user RBAC.
- Filesystem tree and task graph as separate models or one polymorphic model.
- Event-sourced workflow or relational state with append-only audit events.
- Exact approval levels and initial agent capabilities.
