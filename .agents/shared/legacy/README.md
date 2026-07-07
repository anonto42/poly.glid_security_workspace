# Legacy Agent Context

This folder holds merged notes from older agent roots that used to live outside
the canonical `.agents` tree.

Use these files only when a user explicitly asks about the old context. They are
not active PolyGlid instructions, rules, skills, or implementation plans.

## Merged Sources

- `.agents-extra/` contained DNS monorepo guidance and memory notes.
- `.claude/` contained EcoMart Go/Next.js Claude guidance, role cards, commands,
  and settings.

## Current Rule

PolyGlid agents should load `.agents/shared/agent-startup.md`,
`.agents/shared/caveman.md`, `.agents/shared/memory/`, and the Rust/PolyGlid
rules before implementation.
