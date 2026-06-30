# 2026-06-30 Legacy Agent Folders Merged

## Change

Merged the old `.agents-extra/` and `.claude/` roots into the canonical
`.agents` knowledge tree as legacy summaries.

## Decision

The source folders were not copied as active instructions because they described
older, unrelated projects:

- `.agents-extra/` described a DNS monorepo.
- `.claude/` described an EcoMart Go/Next.js application.

Their useful historical context now lives under `.agents/shared/legacy/`.

## Current Guidance

Future PolyGlid development should continue from:

- `.agents/shared/agent-startup.md`
- `.agents/shared/caveman.md`
- `.agents/shared/memory/`
- `.agents/shared/rules/polyglid-architecture.md`
- `.agents/shared/coders/rust-coder.md`
