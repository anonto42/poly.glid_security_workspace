# Agent Knowledge Base Centralization

## Status

Completed.

## What Changed

- `.agents/shared/` is now the source of truth for memory, rules, skills, plans,
  and shared change notes.
- `.codex/` and `.claude/` are loader/reference folders only.
- They no longer contain expandable symlink folders for memory, rules, skills,
  plans, or changes.
- `.agents/shared/history/YYYY-MM-DD/` stores daily plans, updates, handoffs,
  and decisions.
- `AGENTS.md` and `CLAUDE.md` both point to `.agents/shared/`.

## Why

Codex and Claude previously had duplicated or duplicate-looking instruction
folders, which could drift or confuse the source of truth. Shared files keep
both assistants aligned.

## Follow-Up

- Put durable plans in `.agents/shared/plans/`.
- Put daily work history in `.agents/shared/history/YYYY-MM-DD/`.
- Put cross-assistant change notes in `.agents/shared/changes/`.
- Keep instruction and note files under 200 lines.
