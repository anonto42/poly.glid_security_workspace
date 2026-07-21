# AI System Roadmap — Local Model + Workspace Data + Continuous Learning

## The Goal

A self-contained AI system where all `tools/ai/` folders work together to:
- Run a local LLM (Ollama) that understands **this specific codebase**
- Continuously ingest code changes so the AI is always up-to-date
- Learn from your interactions to improve suggestions over time
- Eventually fine-tune a model on your workspace's patterns

---

## Phase 1 — Knowledge Base (RAG over workspace code)

### Problem
Today `polyglid-ai analyze` sends raw code to the model with generic prompts. The model has no persistent memory of the workspace structure or history.

### Solution — Codebase Vector Database

```
                    ┌─────────────────────────────┐
                    │   File watcher (inotify)     │
                    │   detects: file saved,       │
                    │   file created, file deleted │
                    └──────────┬──────────────────┘
                               │
                               ▼
                    ┌─────────────────────────────┐
                    │   Chunker + Embedder         │
                    │   • Split files into chunks  │
                    │   • Embed with nomic-embed   │
                    │     or all-MiniLM via Ollama │
                    └──────────┬──────────────────┘
                               │
                    ┌──────────▼──────────────────┐
                    │   Vector Store               │
                    │   (models/embeddings/)       │
                    │   • Qdrant or SQLite + vec   │
                    │   • Stores: chunk text,      │
                    │     file path, line range,   │
                    │     language, embedding       │
                    └──────────┬──────────────────┘
                               │
                    ┌──────────▼──────────────────┐
                    │   RAG Query                  │
                    │   • User asks question       │
                    │   • Embed query              │
                    │   • Find top-k similar chunks │
                    │   • Inject into LLM prompt   │
                    └─────────────────────────────┘
```

### How folders work together

| Folder | Role |
|--------|------|
| `rust/src/features/` — New `workspace_ingest` module | Chunks code, calls Ollama embed endpoint, stores vectors |
| `models/embeddings/` | Vector store files (SQLite + vectors or Qdrant snapshot) |
| `cache/` | Embedding cache (avoid re-embedding unchanged files) |
| `configs/ai-config.toml` — New `[rag]` section | `chunk_size`, `overlap`, `embed_model`, `top_k` |
| `rust/src/providers/` — Modified `generate()` | Injects RAG context before sending prompt |

### Key decisions

- **Embedding model:** `nomic-embed-text` via Ollama (lightweight, 137M params, works on CPU)
- **Vector DB:** `sqlite-vec` extension or a simple file-based approach (no external server needed)
- **Update strategy:** Watch `.git/logs/HEAD` for changed files, re-embed only those
- **Chunk strategy:** 512 tokens, 64 token overlap, by function/class boundaries when possible

---

## Phase 2 — Intelligence (Workspace-Aware Agents)

### Problem
Today's features (analyze, security, suggest) return placeholder data. The stubs need real logic.

### Solution — Tool-Augmented LLM

Instead of the LLM just generating text, give it **tools** it can call:

```
polyglid-ai analyze
  │
  ├── 1. RAG: find relevant code chunks
  ├── 2. Build prompt with context
  ├── 3. LLM decides which tools to call:
  │       • read_file(path, lines)    → reads file content
  │       • search_code(pattern)      → grep across workspace
  │       • list_files(dir)           → directory listing
  │       • run_test(test_name)       → runs specific test
  │       • read_git_log(limit)       → recent commits
  │       • lint_file(path)           → run linter
  │       • read_dep_graph()          → dependency info
  ├── 4. LLM generates final analysis with evidence
  └── 5. Cache result, save to predictions/
```

### How folders work together

| Folder | New content |
|--------|-------------|
| `rust/src/core/` — New `tools.rs` | Tool definitions + execution |
| `rust/src/core/engine.rs` | Modified to use tool-loop instead of single generate |
| `predictions/` | Each analysis/suggestion saved as JSON with tool call trace |
| `training/datasets/` | High-quality Q&A pairs extracted from usage (human feedback loop) |

---

## Phase 3 — Continuous Learning (Feedback Loop)

### Problem
The AI doesn't learn from your usage. Every session is cold.

### Solution — Three feedback mechanisms

```
                    ┌─────────────────────┐
                    │  User Interaction    │
                    │  • Which suggestions │
                    │    do you accept?    │
                    │  • What do you       │
                    │    ignore?           │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                 ▼
    ┌─────────────────┐ ┌─────────────┐ ┌──────────────┐
    │ Implicit signals │ │ Explicit    │ │ Scheduled    │
    │ • Accepted vs   │ │ feedback    │ │ fine-tuning  │
    │   dismissed    │ │ • Thumbs    │ │ • Collect    │
    │ • File viewed  │ │   up/down  │ │   accepted   │
    │   after suggest │ │ • "Better  │ │   suggestions│
    │ • Copied code  │ │   would be"│ │   + workspace │
    └────────┬────────┘ └──────┬──────┘ │   code →     │
             │                 │        │   dataset     │
             ▼                 ▼        │ • Fine-tune   │
    ┌─────────────────────────────┐     │   LoRA adapter│
    │ Preference Store           │     │   on local    │
    │ (models/preferences/)      │     │   model       │
    │ • Per-user weights         │     └──────┬───────┘
    │ • Per-category weights     │            │
    └─────────────────────────────┘            ▼
                                      ┌──────────────┐
                                      │ LoRA Adapter  │
                                      │ (models/lora/)│
                                      └──────────────┘
```

### How folders work together

| Folder | New content |
|--------|-------------|
| `training/datasets/` | Curated Q&A pairs + code examples extracted from workspace |
| `training/notebooks/` | Jupyter notebooks for LoRA fine-tuning (using `unsloth` or `llama.cpp`) |
| `models/lora/` | Trained LoRA adapters (small files, <100MB each) |
| `models/code-completion/` | FIM (Fill-in-Middle) tokens for code completion |
| `models/test-prediction/` | Patterns learned from test files |
| `models/build-optimization/` | Build patterns learned from CI logs |
| `predictions/` | History of predictions + user feedback for retraining |
| `cache/` | Preference weights + per-user profiles |
| `configs/` — New `learning.toml` | `feedback_enabled`, `retrain_interval`, `min_feedback_count` |

---

## Phase 4 — Autonomy (Workspace Agent)

### Problem
You still run `make ai-analyze` manually. The AI never acts proactively.

### Solution — Background agent with scheduling

```
┌─────────────────────────────────────────────────┐
│                 polyglid-ai daemon               │
│  (runs in background, watches workspace)         │
├─────────────────────────────────────────────────┤
│  • Watches git for new commits                   │
│  • Watches files for changes (inotify)           │
│  • Re-embeds changed files (30s debounce)        │
│  • Runs scheduled analysis (nightly)             │
│  • Sends desktop notification on findings        │
│  • Auto-generates PR descriptions                │
│  • Suggests refactoring opportunities            │
│  • Flags security issues in real-time            │
└─────────────────────────────────────────────────┘
```

This is the **agent** version — it observes, decides, and acts on its own.

---

## Folder → Usage Matrix

| Folder | Now | Phase 1 | Phase 2 | Phase 3 | Phase 4 |
|--------|-----|---------|---------|---------|---------|
| **`rust/`** | CLI binary | + RAG module | + Tool system | + Learning loop | + Daemon mode |
| **`configs/`** | ai-config.toml | + `[rag]` section | + `[tools]` section | + `learning.toml` | + `daemon.toml` |
| **`models/`** | Empty | embeddings/ vector store | — | lora/ adapters | — |
| **`cache/`** | LLM responses | + embed cache | + tool results | + user preferences | + scheduled tasks |
| **`predictions/`** | Empty | — | analysis JSON | + feedback data | + automated findings |
| **`training/`** | Empty | — | — | datasets/ + notebooks/ | — |

---

## Implementation Priority

| Phase | What | Effort | Impact |
|-------|------|--------|--------|
| **P1** | RAG vector DB over workspace code | 2-3 days | High — AI finally understands the codebase |
| **P1** | Replace stubs with real LLM-powered analysis | 2-3 days | High — no more placeholder scores |
| **P2** | Tool-calling (read_file, search_code, etc.) | 3-5 days | Medium — better evidence in answers |
| **P3** | Feedback collection (accept/dismiss tracking) | 1-2 days | Medium — enables learning |
| **P3** | Scheduled retraining / LoRA fine-tuning | 5-7 days | Low — complex, benefit depends on data |
| **P4** | Background daemon + auto-suggest | 3-5 days | Low — nice-to-have, needs P1-P3 first |

---

## Quick Wins (can start today)

1. **Make `polyglid-ai analyze` actually use the LLM** — instead of hardcoded 80/100, ask the model for real scores
2. **Add `polyglid-ai ingest` command** — chunks workspace files, stores in `models/embeddings/`
3. **Inject RAG context into prompts** — query relevant chunks before generating
4. **Save every analysis to `predictions/`** — structured JSON for future training data
5. **Track which suggestions are useful** — add `--accept` / `--dismiss` flags
