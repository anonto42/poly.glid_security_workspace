# Implementation Guide

Use only when coding. For details, load the matching file in
`.agents/shared/rules/`.

## Workflow

1. Check dirty state.
2. Load task-specific rules.
3. Load touched crate/app scope from `.agents/shared/scopes/` when useful.
4. Read targeted code with `rg` and focused file reads.
5. Make scoped edits in existing style.
6. Verify narrowly, then broaden for shared contracts.
7. Update daily history for durable/split work; keep `summary.md` current.

## Rule Map

Use `.agents/shared/rules/README.md` to choose task-specific rules.

## PolyGlid Defaults

- Build CLI-first, then connect Tauri UI.
- Put product behavior in `polyglid-core`.
- Put Wasmtime details in `polyglid-runtime`.
- Put WIT contracts in `wit/` and shared bindings/types in
  `polyglid-plugin-api`.
- Keep plugins deterministic and structured before adding real network/file
  capabilities.
- Use vertical slices through contract, runtime, CLI, tests, then UI.

## Security

- Deny plugin capabilities by default.
- Validate targets and plugin manifests before execution.
- Do not expose filesystem, process, raw socket, environment, or secret access
  without explicit host capability design.
- Log/audit permission grants and denials.
- Never build exploit behavior until authorization and guardrails are explicit.

## Frontend Defaults

- Follow the PolyGlid brand guide and dense security-workspace layout.
- Keep pages thin; call Tauri commands that delegate to core use cases.
- Align UI contracts with `polyglid-events` and plugin report types.
- Handle loading, empty, error, and permission-denied states.

## Verification

- Rust changes: targeted tests, then `cargo check --workspace` and
  `cargo test --workspace` when available.
- WIT changes: regenerate/check host and plugin bindings, then run CLI plugin
  smoke tests.
- UI changes: Tauri command path plus frontend typecheck when available.
- Security/capability changes deserve negative tests.

## Reviews

Use `.agents/shared/skills/code-reviewer/SKILL.md` or
`.agents/shared/skills/review-pr/SKILL.md`. Findings first, severity ordered,
with exact files and lines.
