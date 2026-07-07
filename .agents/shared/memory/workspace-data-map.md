# .workspace/ — Data Map

## Live directories (have real content)

| Path | Size | Stores |
|------|------|--------|
| `ai/` | 972 MB | Rust AI engine: source, configs, compiled deps (target/) |
| `automation/` | 60 KB | Makefile build system: includes, scripts, templates |

## Skeleton directories (for future use)

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
