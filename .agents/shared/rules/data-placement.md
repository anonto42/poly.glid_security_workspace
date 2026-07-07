# Data Placement Rules — Where To Store What

## Quick Decision Tree

```
What are you creating?
│
├─ Source code?
│   ├─ Rust crate → projects/rust/crates/<crate-name>/
│   ├─ Rust plugin → projects/rust/plugins/<plugin-name>/
│   ├─ Node.js app → projects/node/<app-name>/
│   ├─ Python project → projects/python/<project-name>/
│   └─ Go project → projects/go/<project-name>/
│
├─ Configuration?
│   ├─ Global (env, linting, build) → configs/
│   ├─ Language-specific (rustfmt, clippy, eslint) → .workspace/configs/languages/
│   ├─ IDE (VSCode, IntelliJ) → .workspace/configs/ide/
│   ├─ Environment (.env.*) → configs/env/
│   └─ AI model config → .workspace/ai/configs/
│
├─ Documentation?
│   ├─ Architecture docs → docs/architecture/
│   ├─ API docs → docs/api/
│   ├─ Security docs → docs/security/
│   ├─ Development guides → docs/development/
│   ├─ Tutorials → docs/tutorials/
│   ├─ Framework guides → docs/framework/
│   └─ Diagrams → docs/diagrams/
│
├─ Tests?
│   ├─ End-to-end → tests/e2e/
│   ├─ Integration → tests/integration/
│   ├─ Performance → tests/performance/
│   ├─ Security → tests/security/
│   └─ Unit → alongside source in projects/
│
├─ Infrastructure?
│   ├─ Docker → infrastructure/docker/compose/
│   ├─ Kubernetes → infrastructure/docker/k8s/
│   ├─ Terraform → infrastructure/terraform/
│   └─ Monitoring → infrastructure/monitoring/
│
├─ Automation?
│   ├─ Makefile target → .workspace/automation/includes/<module>.mk
│   ├─ Shell script → .workspace/automation/scripts/
│   ├─ CI/CD config → .workspace/integrations/ci/
│   └─ Automation template → .workspace/automation/templates/
│
├─ AI / Intelligence?
│   ├─ AI engine source → .workspace/ai/rust/src/
│   ├─ AI config → .workspace/ai/configs/
│   ├─ AI model → .workspace/ai/models/
│   ├─ AI cache → .workspace/ai/cache/
│   └─ Analytics/predictions → .workspace/intelligence/
│
├─ Release / Build?
│   ├─ Release notes → releases/notes/
│   ├─ Compiled binary → releases/binaries/
│   ├─ Package archive → releases/packages/
│   └─ Build output → target/ (gitignored)
│
├─ Shared / Cross-project?
│   ├─ Protocol definitions (protobuf, OpenAPI) → shared/protocols/
│   ├─ Data schemas → shared/schemas/
│   ├─ Shared assets (images, fonts) → shared/assets/
│   └─ Shared configs → shared/configs/
│
├─ SDK / Plugin Development?
│   ├─ Plugin template → sdk/plugin-template/
│   ├─ Plugin example → sdk/examples/<name>/
│   └─ Language SDK → sdk/<language>/
│
├─ Security / Secrets?
│   ├─ Security audit → .workspace/security/audits/
│   ├─ TLS certificates → .workspace/security/certificates/
│   ├─ Security policies → .workspace/security/policies/
│   ├─ Encrypted secrets → .workspace/security/secrets/
│   └─ Security tools → tools/scripts/security/
│
├─ State / Runtime data?
│   ├─ Build caches → .workspace/state/cache/
│   ├─ Logs → .workspace/state/logs/
│   ├─ Temp files → .workspace/state/temp/
│   ├─ Locks → .workspace/state/locks/
│   └─ Metrics → .workspace/state/metrics/
│
├─ IDE / Developer tooling?
│   ├─ VSCode extension → extensions/vscode/
│   ├─ IntelliJ plugin → extensions/intellij/
│   └─ Browser extension → extensions/browser/
│
├─ Quality / Governance?
│   ├─ Quality gates → .workspace/quality/gates/
│   ├─ Benchmarks → .workspace/quality/benchmarks/
│   ├─ Policies → .workspace/quality/policies/
│   └─ Reports → .workspace/quality/reports/
│
├─ Templates?
│   ├─ Project templates → .workspace/templates/projects/
│   ├─ Infrastructure templates → .workspace/templates/infrastructure/
│   └─ Microservice blueprints → .workspace/templates/microservices/
│
└─ Agent / AI internal?
    ├─ Architecture knowledge → .agents/ARCHITECTURE_FLOW.md
    ├─ Command reference → .agents/COMMAND_REFERENCE.md
    ├─ Data maps → .agents/shared/memory/
    ├─ Plans → .agents/shared/plans/
    └─ Rules → .agents/shared/rules/
```

## Lookup by File Extension

| Extension | Default Location |
|-----------|-----------------|
| `.rs` | `projects/rust/crates/` |
| `.ts`, `.tsx` | `projects/node/` |
| `.js`, `.jsx` | `projects/node/` |
| `.py` | `projects/python/` |
| `.go` | `projects/go/` |
| `.toml` | Alongside the project it configures |
| `.json` | Alongside the project or `configs/` |
| `.yaml`, `.yml` | `infrastructure/` or `.workspace/integrations/` |
| `.sh` | `.workspace/automation/scripts/` or `tools/scripts/` |
| `.md` | `docs/` or alongside the project |
| `.wit` | `projects/rust/wit/` or `sdk/` |
| `.proto` | `shared/protocols/proto/` |
| `.sql` | `shared/schemas/database/` |

## Principle

**Everything has one correct home.** If you're unsure, check:
1. Does it configure something? → nearest config directory
2. Does it run/build/test something? → `.workspace/automation/`
3. Does it document something? → `docs/`
4. Is it source code? → `projects/<language>/`
5. Is it shared between projects? → `shared/`
6. Is it runtime state? → `.workspace/state/`
