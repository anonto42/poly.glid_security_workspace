# Workspace AI Current State

Verified: 2026-07-11

## Repository state

- Branch `main` was clean and six commits ahead of `origin/main` when inspected.
- The six local commits primarily add AI-generated workspace content, Makefile
  integration, WPM planning, and audit fixes.
- `.workspace` uses about 3.4 GB; almost all of this is ignored Rust build output
  in `.workspace/ai/rust/target/`.
- `.workspace/automation/scripts/validate-workspace.sh` passes.
- `cargo test --locked` for the AI engine builds successfully but runs zero tests
  and emits 33 warnings.

## Current runtime flow

1. Developer invokes a root Makefile target or `polyglid-ai` CLI command.
2. `main.rs` parses the command and creates `AIEngine` with the current directory
   as workspace root.
3. The engine loads `.workspace/ai/configs/ai-config.toml` and optional domain
   model files.
4. It builds `WorkspaceContext`, provider, cache, feature modules, tool executor,
   ingest service, and feedback tracker.
5. The selected feature reads workspace files or generated indexes.
6. AI operations call an OpenAI-compatible endpoint. Current configuration is
   Ollama at `http://localhost:11434/v1` using `phi3:3.8b`.
7. Results are printed and may be written to `.workspace` reports, predictions,
   diagrams, releases, configs, snippets, or embeddings.
8. Most commands append timing and status to `data/analytics/usage.jsonl`.

## Implemented capabilities

- Workspace structure, dependency, code-quality, and security analysis.
- Code/test/document generation and code review.
- Build suggestions, workspace suggestions, and status reporting.
- Code ingestion, embeddings, cosine-similarity search, and Rust snippet extraction.
- Controlled workspace tools, prediction feedback, and analytics.
- Diagram, release-manifest, IDE/config, Makefile, and change-list generation.
- Background watcher, scheduler, and auto-suggestion pipeline scaffolding.
- WPM design plan and root Makefile targets.

## Known gaps and risks

- File-specific `analyze` and `security` handlers only print messages.
- There are no AI-engine tests, so a successful build is not behavioral proof.
- Compiler warnings expose unfinished or disconnected code paths.
- Domain model configuration is loaded but not consistently routed per feature.
- `Anthropic`, `Hybrid`, and other non-local provider types currently fall through
  to the OpenAI-compatible provider rather than distinct implementations.
- Ollama commands depend on local models, including `nomic-embed-text` for ingest.
- Prediction feedback APIs exist, but generated outputs are not consistently saved
  as feedback-trackable predictions.
- The automation Makefile has duplicate Node/Rust recipes and no `validate` target;
  root validation invokes the validation script directly.
- AI-generated writes lack a unified permission, approval, rollback, and isolated
  worktree policy.

## Intended role

Treat this engine as a project-aware automation and intelligence layer. It is
useful for private local search, repeatable reports, repository conventions, and
CI-compatible operations. It is not yet a full replacement for a capable coding
agent on complex debugging, architecture, or multi-file autonomous changes.

## Immediate technical priorities

1. Add unit and integration tests for CLI, provider parsing, ingest, tools, and
   generated paths.
2. Complete placeholder handlers and remove warnings.
3. Add structured JSON task/result contracts.
4. Add permission boundaries and approval gates before agent-generated writes.
5. Implement real domain model routing and provider fallbacks.
6. Move disposable AI build output out of `.workspace` knowledge storage.
7. Build the coordinator and project-management layer described in the active plan.
