# PolyGlid Workspace Summary

## Directory Layout
```
.
├── Makefile                          # Root — all commands
├── workspace.toml                    # Workspace project registry
├── apps/                             # Desktop, CLI, and server applications
├── crates/                           # Core, runtime, config, events, plugin API
├── contracts/                        # Canonical WIT contract
├── plugins/                          # First-party WASM plugins
├── site/                             # Static public website generator
├── tools/
│   ├── ai/                           # AI engine (Rust binary)
│   │   ├── configs/                  # ai-config.toml + per-domain model-configs/
│   │   ├── rust/src/                 # Engine source
│   │   │   ├── main.rs               # CLI entrypoint (16 commands)
│   │   │   ├── cli/commands.rs       # Command handlers
│   │   │   ├── core/engine.rs        # AIEngine, config, RAG
│   │   │   ├── features/             # Analysis, security, ingest, diagrams, etc.
│   │   │   ├── providers/            # Ollama/OpenAI/Local
│   │   │   ├── tools/                # 5 built-in tools
│   │   │   ├── feedback/             # Prediction tracking
│   │   │   └── pipelines/            # Daemon, watcher, scheduler
│   │   └── models/embeddings/        # Vector index (JSON)
│   ├── automation/
│   │   ├── includes/                 # Included .mk modules
│   │   │   ├── projects/*.mk         # Per-project targets (auto-generated)
│   │   │   ├── languages.mk          # Language-level build/test/clean
│   │   │   ├── docker.mk / k8s.mk / ci.mk
│   │   │   └── colors.mk, config.mk, utils.mk, help.mk
│   │   ├── scripts/                  # validate-workspace, generate-graph, detect-changes
│   │   └── templates/                # project.mk.template, language.mk.template
│   ├── data/analytics/               # Usage logs (JSONL)
│   └── quality/reports/              # AI analysis outputs
│   └── security/audits/              # AI security audit outputs
```

## Make Commands

### Workspace
| Command | Description |
|---------|-------------|
| `make init` | 6-phase setup: check tools, install deps, build, validate |
| `make status` | Show workspace health |
| `make info` | Show metadata (name, version, languages) |
| `make graph` | Generate dependency graph |
| `make new-project` | Scaffold a new project interactively |

### Build / Test / Dev
| Command | Description |
|---------|-------------|
| `make build` | Build all Rust + Node projects |
| `make test` | Run all tests |
| `make dev` | Start dev servers (parallel) |
| `make clean` | Remove build artifacts |
| `make build-rust` | Build Rust workspace (`cargo build --release`) |
| `make test-rust` | Run Rust tests (`cargo test --all`) |

### AI Engine
All AI commands auto-build the engine binary before running.

| Command | Description | Output |
|---------|-------------|--------|
| `make ai-analyze` | Full workspace AI analysis | `.workspace/quality/reports/` |
| `make ai-suggest` | Get improvement suggestions | stdout |
| `make ai-security` | Security audit scan | `.workspace/security/audits/` |
| `make ai-status` | Engine health check | stdout |
| `make ai-ingest` | Build code vector index | `tools/ai/models/embeddings/` |
| `make ai-search QUERY="..."` | Semantic code search | stdout |
| `make ai-diagram` | Generate architecture diagrams | `docs/diagrams/` |
| `make ai-release` | Generate K8s deploy manifests | `releases/manifests/` |
| `make ai-init-configs` | Generate .gitignore, .editorconfig, .vscode | root + `configs/` |
| `make ai-generate-mk` | Regenerate per-project .mk files | `automation/includes/projects/` |
| `make ai-detect-changes` | List projects changed since main | stdout |

### WPM (Workspace Project Manager)
| Command | Description |
|---------|-------------|
| `make init-wpm` | Scaffold WPM project from design plan |
| `make wpm-build` | Build WPM binary |
| `make wpm-run` | Start WPM server |
| `make wpm-db-setup` | Create + migrate database |
| `make wpm-test` | Run WPM tests |
| `make wpm-docker-up` | Start Docker Compose stack |
| `make wpm-docker-down` | Stop Docker Compose stack |
| `make wpm-plan` | Show WPM design document |

### Deploy
| Command | Description |
|---------|-------------|
| `make deploy` | Build → Docker → K8s (stub) |

## How to Manage

### Adding a new Make command
1. Open `Makefile` (root)
2. Add a `.PHONY:` line + recipe following the existing pattern
3. Run `make help` to verify it appears

### Per-project overrides
- Run `make ai-generate-mk` to regenerate project `.mk` files
- Edit `automation/includes/projects/<project>.mk` to override build/test/clean per project
- These are auto-included at the bottom of `Makefile` via `-include`

### Language-level targets
- Edit `automation/includes/languages.mk` to change how all projects of a language build/test/clean

### Infrastructure targets
- Docker: `automation/includes/docker.mk`
- Kubernetes: `automation/includes/k8s.mk`
- CI: `automation/includes/ci.mk`

### AI engine
- Source: `tools/ai/rust/src/`
- Config: `tools/ai/configs/ai-config.toml`
- Per-domain model overrides: `tools/ai/configs/model-configs/*.toml`
- Rebuild: `make build-ai-engine` (auto-runs before any `ai-*` command)

### Requirements
- **Ollama** must be running (`ollama serve`) for AI commands to work
- **nomic-embed-text** model needed for `ai-ingest` (`ollama pull nomic-embed-text`)
- Rust toolchain, Node.js, pnpm for builds
