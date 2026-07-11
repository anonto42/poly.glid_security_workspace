# .workspace/ — Data Map

## Live directories (verified 2026-07-11)

| Path | Size | Stores |
|------|------|--------|
| `ai/` | ~3.4 GB including ignored artifacts | Rust AI engine, configs, predictions, training samples, embeddings, and `rust/target/` |
| `automation/` | 60 KB | Makefile build system: includes, scripts, templates |

Most disk usage is ignored Rust output under `.workspace/ai/rust/target/`; it is
not project knowledge. Build artifacts should eventually use the root `target/`
or another disposable cache location.

## Generated or partially active directories

| Path | Subdirs | Stores |
|------|---------|--------|
| `configs/` | `env/`, `git/`, `ide/` | Environment `.env` files, git hooks, IDE settings |
| `data/` | `analytics/`, `backups/`, `migrations/` | Build/test analytics, DB backups, SQL schemas |
| `docs/` | `diagrams/`, `examples/`, `snippets/` | Mermaid/PlantUML diagrams, code examples, language snippets |
| `integrations/` | `ci/`, `communication/`, `monitoring/` | CI pipeline configs, Slack/Discord webhooks, Grafana dashboards |
| `plugins/` | `generators/`, `validators/` | Code generators, lint/schema validators |
| `quality/` | `benchmarks/`, `gates/`, `policies/`, `reports/` | Benchmark suites, quality gates, project policies, periodic reports |
| `releases/` | `manifests/`, `registry/` | Version manifests, service/dependency registry |
| `security/` | `audits/`, `certificates/`, `policies/`, `secrets/` | Audit reports, TLS certs, security policies, encrypted secrets |
| `state/` | `cache/`, `locks/`, `logs/`, `temp/` | Build caches, file locks, log archives, temporary files |
| `templates/` | `infrastructure/`, `projects/` | Docker/K8s/Terraform templates, project starter templates |

`configs/`, `data/`, `docs/`, `quality/`, `releases/`, and `security/` are no
longer purely skeletons. The AI CLI can generate or append content in these
locations. See `workspace-ai-current-state.md` before changing their contracts.
