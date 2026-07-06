# Shared Knowledge Base

Claude and Codex load from here.

- `agent-startup.md` - compact startup loader.
- `caveman.md` - shared compressed style/memory rules.
- `memory/` - stable facts and coding guide.
  - `development-commands.md` - setup, dev, database, and verification commands.
- `rules/` - task-specific rules; load selectively.
- `coders/` - shared role cards for Rust, review, and release work.
- `history/` - daily plans, updates, handoffs, decisions.
- `plans/` - cross-day plans only.
- `changes/` - repo-wide convention/change notes.
- `legacy/` - archived summaries from merged non-PolyGlid agent roots.
- `reference-map.md` - canonical paths.

Rules: keep files under 200 lines, avoid duplicate assistant-specific copies,
and never store secrets or raw chat logs. Legacy summaries are not active rules.
