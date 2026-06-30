# Agent Startup

Use this as the compact shared loader for Claude and Codex.

## Load

1. `.agents/shared/memory/MEMORY.md`
2. Today's `.agents/shared/history/YYYY-MM-DD/caveman.md`; create a stub if
   missing and work will continue in this repo
3. Matching scope file from `.agents/shared/scopes/`, if applicable
4. Matching coder role from `.agents/shared/coders/`, if useful
5. `.agents/shared/rules/README.md`, then relevant rule files only

Use Asia/Dhaka for the date unless the user says otherwise.
Daily history is local working memory and stays gitignored.

## Hard Rules

- PolyGlid is a Rust/Tauri/WASM security workspace; ignore stale app-stack
  assumptions from older agent memory.
- CLI-first: plugin behavior must be testable without the GUI.
- `polyglid-core` owns product behavior; adapters own external systems.
- Plugins are untrusted by default and capabilities are denied by default.
- Keep WIT contracts stable and structured; do not parse plugin terminal text.
- Security features are for authorized testing and defensive diagnostics.
- Do not store secrets, raw chat logs, tokens, private keys, or target
  credentials in `.agents`.

## Work Loop

- Check dirty worktree and preserve unrelated changes.
- Identify area: docs, core, runtime, plugin API, config, events, CLI, desktop,
  plugin, tests, infra, or review.
- Load matching coder role when it helps.
- Load only matching rules, not the whole rules folder.
- For review/today-work requests, load `.agents/shared/caveman.md` and use
  compressed review output unless user asks normal.
- For split work, read today's `plans.md` and `handoffs.md`; read full
  `updates.md` only when details are needed.
- After meaningful work, append durable progress to today's `updates.md`.
- After meaningful work, rewrite today's `caveman.md` to reflect current
  `STATE` / `TODAY` / `NEXT` / `HANDOFFS` / `RISKS` / `LOAD`.
- Record task splits in `handoffs.md` and decisions in `decisions.md`.
- Keep files under 200 lines; create `updates-02.md` when needed.
- Do not store secrets or raw chat logs.
