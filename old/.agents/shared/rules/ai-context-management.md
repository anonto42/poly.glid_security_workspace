---
paths:
  - '.agents/**'
  - 'docs/**'
---

# AI Context Management

Use this when changing `.agents`, docs, skills, memory, plans, or Graphify-style
knowledge recall.

## Context Layout

- Keep `.agents/shared/` as the only durable assistant source of truth.
- Keep always-loaded files small and stable.
- Put task-specific instructions in focused rule, scope, coder, or skill files.
- Keep durable project facts in `memory/`.
- Keep cross-day plans in `plans/` and daily work in `history/YYYY-MM-DD/`.
- Keep repo-wide changes in `changes/`.

## Injection Rules

- Load core memory first.
- Load only rules that match the touched files or task.
- Load scopes and coder roles only when they add useful constraints.
- Prefer precise file reads over dumping the whole context tree.
- Keep files under 200 lines when possible.

## Knowledge Graph Recall

Graphify-style indexing can help find relationships across code, docs, WIT,
tests, decisions, and agent rules.

- Treat graph hits as navigation hints.
- Verify every claim against source files before editing.
- Do not let generated summaries override code, tests, docs, or security rules.
- Do not index secrets, raw chats, private keys, or target credentials.

## Maintenance

- Update context files when architecture or workflow changes.
- Remove stale instructions that describe old stacks.
- Avoid duplicate assistant-specific copies.
- Record why durable context changed in `.agents/shared/changes/`.
