# Full Project Data Map — What Lives Where

---

## Root Level

```
poly.glid_security_workspace/
│
├── 📄 ARCHITECTURE.md          # System design & architecture docs
├── 📄 Cargo.toml               # Rust workspace configuration
├── 📄 Cargo.lock               # Rust dependency lock file
├── 📄 Makefile                 # Main automation entry point
├── 📄 package.json             # Node.js/NPM scripts wrapper
├── 📄 workspace.toml           # Workspace configuration
├── 📄 README.md                # Project overview
├── 📄 polyglid.db              # SQLite database (local dev)
│
├── 📁 configs/                 # Global configurations
├── 📁 docs/                    # Documentation
├── 📁 extensions/              # IDE extensions & plugins
├── 📁 infrastructure/          # Docker, K8s, Terraform
├── 📁 slices/                  # All source code
├── 📁 releases/                # Release artifacts
├── 📁 sdk/                     # SDK implementations
├── 📁 shared/                  # Shared resources
├── 📁 target/                  # Build outputs
├── 📁 tests/                   # Global tests
├── 📁 tools/                   # Workspace tools
└── 📁 .workspace/              # Automation & configuration
```

---

## Root Files

### `ARCHITECTURE.md`
System architecture diagrams, component relationships, design decisions (ADRs), technology stack, data flow, security architecture, deployment strategies.

**Current:** 51 lines — crate architecture, execution pipeline, cryptographic signature stages.

### `Cargo.toml`
Defines all Rust workspace crates, manages shared dependencies, configures build profiles.

**Current:** 8 crate members: `polyglid-cli`, `polyglid-config`, `polyglid-core`, `polyglid-events`, `polyglid-plugin-api`, `polyglid-runtime`, `polyglid-server`, `.workspace/ai/rust`.

### `Makefile`
Single entry point for all operations: build, test, dev, clean, init, help. Includes OS detection layer.

**Current:** Modular phases: `init` → `_init-check-tools` → `_init-install-deps` → `_init-build` → `_init-validate`.

### `package.json`
NPM scripts wrapper that delegates to make (dev, build, test, clean, init).

### `workspace.toml`
Workspace configuration: languages (node, rust), 9 project definitions with paths, language, type.

### `README.md`
Project overview, product shape, core principles, build order, MVP instructions.

### `polyglid.db`
SQLite database for local development — plugin registry, execution history, permissions, audit logs.

---

## Folder by Folder

---

### 1. `configs/` — Global Configurations

| Subdirectory | Contents | Status |
|-------------|----------|--------|
| `env/` | `.env.dev`, `.env.staging`, `.env.production` | ✅ Files exist |
| `linting/` | `.eslintrc.js`, `.prettierrc`, `.stylelintrc` | ✅ Files exist |
| `build/` | `webpack.config.js`, `rollup.config.js`, `vite.config.js` | ✅ Files exist |
| `git/hooks/` | pre-commit, pre-push, commit-msg | ⬜ Skeleton |
| `git/templates/` | commit message templates | ⬜ Skeleton |
| `security/` | `.snyk`, `.trivyignore`, `.secrets.baseline` | ⬜ Not created |

**Purpose:** All global configuration files for the entire workspace.
**Status:** 3 subdirs live (env, linting, build), rest are empty stubs.

---

### 2. `docs/` — Documentation Hub

| Subdirectory | Contents | Status |
|-------------|----------|--------|
| `architecture/` | CODEBASE.md, CLI_TECH_DECISION.md, DESKTOP_UI.md, REPOSITORY_LAYOUT.md, SYSTEM.md | ✅ 6 files |
| `guides/` | getting-started, adding-new-language, workspace-conventions, deployment-guide | ⬜ Not created |
| `decisions/` | ADR records (polyglot approach, build system, language isolation, security framework) | ⬜ Not created |
| `tutorials/` | building-plugins, security-analysis, custom-analyzers | ⬜ Skeleton |
| `diagrams/` | PlantUML, Mermaid, images | ⬜ Not created |
| `api/` | REST API, gRPC API, WebSocket docs | ⬜ Not created |
| `development/` | MVP_RUNBOOK.md, PACKAGING.md, STEP_BY_STEP_FLOW.md, WORKFLOW.md | ✅ 4 files |
| `planning/` | DEVELOPMENT_TARGETS.md, MVP.md, ROADMAP.md | ✅ 3 files |
| `security/` | SECURITY_MODEL.md | ✅ 1 file |
| `framework/` | 10-part framework series (index → stack-maps) | ✅ 11 files |
| `plugin_foundation/` | Reference doc | ✅ 1 file |
| `production_stack/` | Reference doc | ✅ 1 file |
| `technical_feature/` | Reference doc | ✅ 1 file |
| `branding/` | Brand guide | ✅ 1 file |

---

### 3. `extensions/` — IDE & Browser Extensions

| Subdirectory | Contents | Status |
|-------------|----------|--------|
| `vscode/` | PolyGlid extension, language support | ⬜ Skeleton |
| `intellij/` | PolyGlid plugin, language support | ⬜ Skeleton |
| `browser/` | Chrome extension, Firefox extension | ⬜ Skeleton |

---

### 4. `infrastructure/` — Infrastructure as Code

| Subdirectory | Contents | Status |
|-------------|----------|--------|
| `docker/images/` | Custom Docker images (rust-builder, node-builder, python-builder) | ⬜ Not created |
| `docker/compose/` | Docker Compose files (dev, staging, production) | ⬜ Skeleton |
| `docker/k8s/` | Kubernetes manifests (base deployments, services, ingress, configmaps + staging/production overlays + Helm charts) | ⬜ Skeleton |
| `terraform/` | Terraform configs (AWS, GCP, Azure with modules) | ⬜ Skeleton |
| `monitoring/` | Prometheus config + alerts, Grafana dashboards + datasources, Loki + Fluentbit logging | ⬜ Skeleton |

---

### 5. `slices/` — All Source Code

| Path | Language | Contents | Status |
|------|----------|----------|--------|
| `slices/apps/desktop/` | Rust | Dioxus developer-space application and WPM UI | ✅ Live |
| `slices/engine/core/` | Rust | Core engine: execution, plugins, security, and stores | ✅ Live |
| `slices/apps/cli/` | Rust | CLI and Ratatui development dashboard | ✅ Live |
| `slices/apps/server/` | Rust | HTTP backend: auth routes, server setup, tests | ✅ Live |
| `slices/engine/runtime/` | Rust | Wasmtime component loading and sandboxed execution | ✅ Live |
| `slices/contracts/plugin-api/` | Rust | Plugin-facing types and capabilities | ✅ Live |
| `slices/configs/config/` | Rust | Configuration and plugin registry | ✅ Live |
| `slices/contracts/events/` | Rust | Typed event system | ✅ Live |
| `slices/contracts/wit/polyglid.wit` | WIT | Canonical host/plugin contract | ✅ Live |
| `slices/plugins/recon-probe/` | Rust | First-party WASM diagnostic plugin | ✅ Live |

---

### 6. `releases/` — Release Management

| Subdirectory | Contents | Status |
|-------------|----------|--------|
| `notes/` | Release notes (v1.0.0, v1.1.0, v1.2.0) | ⬜ Skeleton |
| `binaries/` | Compiled binaries (linux, macos, windows) | ⬜ Skeleton |
| `packages/` | Package archives (tar.gz, zip) | ⬜ Skeleton |
| `manifests/` | Helm chart artifacts | ⬜ Not created |

---

### 7. `sdk/` — SDK Implementations

| Path | Contents | Status |
|------|----------|--------|
| `sdk/plugin-template/` | Starter template: Cargo.toml, src/lib.rs, wit/polyglid.wit | ✅ Live |
| `sdk/examples/hello_world/` | Minimal example plugin | ✅ Live |
| `sdk/examples/recon_probe/` | Feature example plugin | ✅ Live |
| `sdk/README.md` | SDK usage guide | ✅ Live |
| `sdk/CHANGELOG.md` | Version history | ✅ Live |
| `sdk/VERSION` | Current version number | ✅ Live |
| `sdk/rust/` | Future Rust SDK | ⬜ Skeleton |
| `sdk/node/` | Future Node.js SDK | ⬜ Skeleton |
| `sdk/python/` | Future Python SDK | ⬜ Skeleton |
| `sdk/go/` | Future Go SDK | ⬜ Skeleton |

---

### 8. `shared/` — Shared Resources

| Subdirectory | Contents | Status |
|-------------|----------|--------|
| `configs/` | Shared eslint, prettier, typescript, docker configs | ⬜ Skeleton |
| `protocols/` | Protobuf definitions, Thrift, OpenAPI specs | ⬜ Skeleton |
| `schemas/` | Database schemas, JSON validation, GraphQL | ⬜ Skeleton |
| `assets/` | Images, fonts, translations | ⬜ Skeleton |

---

### 9. `target/` — Build Outputs

| Subdirectory | Contents | Status |
|-------------|----------|--------|
| `debug/` | Debug builds (Rust, Node, Python) | ⬜ Auto-generated |
| `release/` | Release builds (binaries, dist) | ⬜ Auto-generated |
| `test/` | Test outputs (coverage, reports, logs) | ⬜ Auto-generated |

**Note:** `.gitignore` includes `/target/` — not committed.

---

### 10. `tests/` — Global Tests

| Subdirectory | Contents | Status |
|-------------|----------|--------|
| `e2e/` | End-to-end tests (Cypress, Playwright, API tests) | ⬜ Skeleton |
| `integration/` | Integration tests (service, database, message queue) | ⬜ Skeleton |
| `performance/` | Performance tests (load, stress, benchmark) | ⬜ Skeleton |
| `security/` | Security tests (vulnerability scan, penetration, compliance) | ⬜ Skeleton |

---

### 11. `tools/` — Workspace Tools

| Subdirectory | Contents | Status |
|-------------|----------|--------|
| `polyglid/` | Custom orchestration tool (Rust: engine, commands, adapters) | ⬜ Skeleton |
| `scripts/generators/` | new-project, new-language generators | ⬜ Skeleton |
| `scripts/migrations/` | Database migration scripts | ⬜ Skeleton |
| `scripts/ci-cd/` | Deploy and release scripts | ⬜ Skeleton |
| `templates/` | Project templates (node, python, rust, go) | ⬜ Skeleton |

---

### 12. `.workspace/` — Workspace Management System

| Path | Size | Contents | Status |
|------|------|----------|--------|
| `automation/` | 60 KB | Makefile build system: 10 includes (os, colors, config, utils, languages, docker, k8s, ci, help), 3 scripts, 2 templates | ✅ Live |
| `ai/` | 972 MB | Rust AI engine: source (26 files), configs (4 files), compiled deps (960 MB target/) | ✅ Live |
| `configs/` | 0 B | Future: environments, global git/husky, IDE configs, language configs | ⬜ Skeleton |
| `state/` | 0 B | Future: caches, logs, temp, metrics, locks | ⬜ Skeleton |
| `templates/` | 0 B | Future: project templates, microservice blueprints, infra templates | ⬜ Skeleton |
| `plugins/` | 0 B | Future: version-manager, dependency-graph, health-checker | ⬜ Skeleton |
| `security/` | 0 B | Future: secrets, audits, policies, certificates | ⬜ Skeleton |
| `quality/` | 0 B | Future: gates, reports, benchmarks, policies | ⬜ Skeleton |
| `releases/` | 0 B | Future: manifests, registry, strategies, rollbacks | ⬜ Skeleton |
| `data/` | 0 B | Future: migrations, backups, analytics | ⬜ Skeleton |
| `integrations/` | 0 B | Future: CI (GitHub Actions, GitLab, Jenkins), monitoring (Prometheus, Grafana, Datadog), communication (Slack, Discord, Teams) | ⬜ Skeleton |
| `docs/` | 0 B | Future: generated docs, snippets, diagrams | ⬜ Skeleton |

---

## Complete Folders at a Glance

| Folder | Purpose | Contents | Status |
|--------|---------|----------|--------|
| **`configs/`** | Global configs | env, linting, build, git | ⚡ 3 live |
| **`docs/`** | Documentation | architecture, planning, framework, security, dev | ✅ 28 files |
| **`extensions/`** | IDE extensions | VSCode, IntelliJ, browser | ⬜ Skeleton |
| **`infrastructure/`** | Infrastructure as Code | Docker, K8s, Terraform, monitoring | ⬜ Skeleton |
| **`slices/`** | Source code | Rust (7 crates + 1 plugin), Node (2 frontends) | ✅ Live |
| **`releases/`** | Release artifacts | notes, binaries, packages | ⬜ Skeleton |
| **`sdk/`** | SDKs | plugin-template, examples, per-language stubs | ✅ Live |
| **`shared/`** | Shared resources | configs, protocols, schemas, assets | ⬜ Skeleton |
| **`target/`** | Build outputs | Auto-generated (gitignored) | ⬜ Generated |
| **`tests/`** | Test suites | e2e, integration, performance, security | ⬜ Skeleton |
| **`tools/`** | Workspace tools | polyglid, scripts, templates | ⬜ Skeleton |
| **`.workspace/`** | Management system | automation, AI, configs, state, templates | ✅ 2 live + 10 skeletons |

---

## Legend

| Icon | Meaning |
|------|---------|
| ✅ **Live** | Has real files with content |
| ⚡ Partial | Some files exist, more planned |
| ⬜ Skeleton | Empty dirs (`.gitkeep` only) — ready for future use |
