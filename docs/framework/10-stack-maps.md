# 10 · Cross-Stack Maps

> **Phase:** all · **Type:** reference
> The whole framework is **technology-agnostic**: the mental models, principles,
> architecture, security, and scaling chapters are identical no matter what you build in.
> **Only the tools change.** This chapter is the translation table.

← [09 Scaling](09-scaling.md) · [Index](00-INDEX.md)

---

## The core claim

> Chapters [01](01-mindset.md)–[04](04-principles.md), [08](08-security.md), and
> [09](09-scaling.md) do not change between stacks. SOLID is SOLID in Go, TypeScript, Java,
> Python, Dart. The dependency rule, CIA, the scaling ladder, 12-factor — universal. What
> changes is **which formatter, which test runner, which web framework, which packaging
> tool** instantiates each idea.

So porting this playbook to a new project = keep chapters 01–04/08/09 verbatim, and swap
the *tool rows* below.

---

## Universal tooling map (every backend/CLI stack needs all of these)

| Concern ([chapter]) | Rust *(this repo)* | Go | Node / TypeScript | Java / Kotlin (JVM) | Python |
|---------------------|--------------------|----|--------------------|---------------------|--------|
| Build / deps | Cargo + `Cargo.lock` | go mod + `go.sum` | npm/pnpm + lockfile | Gradle/Maven | uv/Poetry + lock |
| Format ([06](06-tooling.md)) | `cargo fmt` | `gofmt` | Prettier | Spotless/ktlint | Black/ruff-fmt |
| Lint ([06](06-tooling.md)) | Clippy | golangci-lint | ESLint | Detekt/SpotBugs | Ruff/Pylint |
| Type/compile check | rustc + borrow ck | `go build/vet` | `tsc` | javac/kotlinc | mypy/pyright |
| Test ([06](06-tooling.md)) | `cargo test` | `go test` | Vitest/Jest | JUnit | pytest |
| Property tests | proptest | gopter | fast-check | jqwik | Hypothesis |
| Errors ([05](05-tech-selection.md)) | `Result`, thiserror/anyhow | explicit `error` values | typed errors/Result libs | exceptions (checked sparingly) | exceptions + types |
| HTTP server (adapter) | axum/actix | net/http, chi, Gin | Express/Fastify/Nest | Spring Boot/Quarkus | FastAPI/Django |
| DB access (adapter) | sqlx/SeaORM | sqlc/GORM | Prisma/Drizzle | JPA/jOOQ | SQLAlchemy |
| Logging ([07](07-operations.md)) | `tracing` | slog/zap | pino | SLF4J/Logback | structlog |
| Telemetry ([07](07-operations.md)) | OTel-rust | OTel-go | OTel-js | OTel-java (agent) | OTel-python |
| Dep audit ([08](08-security.md)) | `cargo audit` | govulncheck | npm audit | OWASP dep-check | pip-audit |
| Package/ship ([06](06-tooling.md)) | static binary | static binary | container/bundle | jar/native-image | wheel/container |

> Concurrency model differs and shapes design: Rust async (Tokio) / Go goroutines+channels
> / Node single-thread event loop / JVM threads+virtual threads / Python asyncio + GIL
> (use processes for CPU-bound). The *patterns* ([07 resilience](07-operations.md)) are the
> same; the primitives differ.

---

## Backend-only stacks — when to reach for each

| Stack | Sweet spot | Watch out for |
|-------|-----------|---------------|
| **Go** | Network services, CLIs, infra tooling; fast builds, tiny static binaries, simple ops | Verbose error handling; generics still maturing in idioms |
| **Node/TS** | Rapid product dev, full-stack JS, I/O-bound APIs, huge ecosystem | CPU-bound work; dependency sprawl; pick TS (types) over JS |
| **Java/Kotlin** | Large, long-lived enterprise systems; strong tooling & talent | Heavier runtime/footprint; framework magic (Spring) can hide cost |
| **Python** | Data/ML, scripting, fast prototypes, glue | GIL limits CPU parallelism; packaging historically painful (use uv); slower runtime |
| **Rust** | Native perf + memory safety, single binary, untrusted-input parsing, low footprint | Steeper learning curve; longer compile times; smaller (but growing) talent pool |

**Architecture is identical across all of them:** `domain` (pure) ← `ports` (interfaces) ←
`adapters` (I/O) ← `application` (use cases), package-by-feature as it grows
([03](03-architecture.md)). The folder names carry over verbatim.

---

## Frontend stacks

The same principles apply; the vocabulary shifts (components, state management, rendering).

| Concern | Web (React/Vue/Svelte + TS) | Desktop | Mobile |
|---------|------------------------------|---------|--------|
| Framework | React/Next, Vue/Nuxt, SvelteKit | **Tauri** (Rust+web, tiny), Electron (web, heavy), native | **Flutter/Dart** (cross), Swift/SwiftUI (iOS), Kotlin/Compose (Android), React Native |
| Architecture | Components + unidirectional data flow; feature folders; container/presentational split | Same web stack + native bridge as an **adapter** | MVVM/MVI/BLoC; UI ← state ← domain |
| State mgmt | Redux/Zustand/Pinia/signals | same | Riverpod/BLoC (Flutter), state holders |
| The "domain" | **Still pure & framework-free** — business rules don't import React | same | same — keep logic out of widgets/views |
| Testing | Vitest + Testing Library; Playwright/Cypress E2E | same + native smoke tests | widget/UI tests + device/emulator E2E |
| Build/ship | Vite/bundler; CDN; versioned assets | platform installers; **code signing** + [auto-update](07-operations.md#auto-update) | App Store/Play pipelines; staged rollout |
| Cross-cutting | a11y, i18n, perf budgets (bundle size, Core Web Vitals), CSP/XSS ([08](08-security.md)) | OS integration, sandboxing | offline-first, battery/network, permissions |

> The dependency rule still holds: **UI is a driving adapter.** Your business logic should
> be testable with no DOM, no widget tree, no simulator. A React component calling
> `fetch()` inline is the frontend version of putting HTTP in your domain.
>
> *Desktop note:* **Tauri** (Rust core + web UI) is the spiritual sibling of an fbsy-style
> app — small, native, signed, auto-updating — vs Electron's larger footprint.

---

## Full-stack composition

- **Contract-first:** the API between frontend and backend is a [bounded
  context](03-architecture.md) boundary — define it explicitly (OpenAPI/GraphQL schema/
  gRPC/tRPC) and **contract-test** it ([06](06-tooling.md)).
- **Share types, not logic, across the boundary** (e.g. generate TS types from the schema).
  Don't couple frontend and backend internals.
- **Monorepo** often shines here (atomic API+client changes — [03](03-architecture.md)).
- Each side keeps its own clean architecture; they meet only at the contract.

---

## Infrastructure & system architecture

The framework extends to how you *run* it. Same principles (declarative, versioned,
least-privilege, observable, immutable):

| Concern | Practice / tool |
|---------|-----------------|
| Provisioning | **Infrastructure as Code** — Terraform/OpenTofu, Pulumi (declarative, in VCS, reviewed) |
| Config mgmt | Ansible; cloud-init; or immutable images (Packer) |
| Runtime | Single VPS → containers (Docker) → orchestration (Kubernetes/Nomad) → serverless — climb the [scaling ladder](09-scaling.md) |
| Networking | Reverse proxy/LB (nginx/Caddy/Traefik); private subnets; least-privilege security groups ([08](08-security.md)) |
| Secrets | Vault / cloud KMS / SOPS ([08](08-security.md#secrets)) |
| Observability | Prometheus + Grafana + Loki + Tempo, or a managed APM; **OTel** as the wire ([07](07-operations.md)) |
| Delivery | GitOps (Argo CD/Flux) or pipeline-driven CD; immutable artifacts ([06](06-tooling.md)) |
| Cost/reliability | Cloud **Well-Architected** reviews; autoscaling with limits; multi-AZ before multi-region |

> **IaC = your infra's source code:** it gets the same treatment as app code — version
> control, review, CI, no manual console clicks ("ClickOps" is the infra equivalent of an
> undocumented hotfix).

---

## How to bootstrap a new project (any stack) in order

A condensed, stack-neutral sequence. Do them top to bottom.

1. **Repo + license + README** stating the one-paragraph brief ([02](02-planning.md)).
2. **Pin the toolchain**, init deps + lockfile ([06](06-tooling.md)).
3. **Wire quality gates first:** format, lint, type-check, test runner, pre-commit hook, CI
   skeleton — *before* feature code ([06](06-tooling.md)).
4. **Lay out the architecture skeleton:** `domain / ports / adapters / application` (+
   `config`, `support`) ([03](03-architecture.md)).
5. **Build the [walking skeleton](02-planning.md#walking-skeleton):** one end-to-end
   vertical slice that runs and *ships*.
6. **Add observability + config from the first slice:** structured logs, env config
   ([07](07-operations.md)).
7. **Threat-model the boundaries; set secure defaults** ([08](08-security.md)).
8. **Write ADR-0001** recording the stack + architecture choice ([02](02-planning.md#architecture-decision-records-adrs)).
9. **Set up CI/CD** with signed, versioned artifacts ([06](06-tooling.md)).
10. **Iterate in vertical slices**, revisiting plan/architecture in miniature each time.

---

## Cloning this framework into another repo

1. Copy the `docs/framework/` folder over.
2. Keep [01](01-mindset.md)–[04](04-principles.md), [08](08-security.md),
   [09](09-scaling.md) as-is — they're universal.
3. In [05](05-tech-selection.md), [06](06-tooling.md), [07](07-operations.md), swap the
   *tool rows* using the tables above for your stack.
4. Rewrite the "worked example" sections and the [Index](00-INDEX.md) mapping for the new
   project.
5. Start the bootstrap sequence above; record decisions as ADRs.

---

## References

- The Twelve-Factor App (stack-neutral) — <https://12factor.net/>
- Terraform/OpenTofu — <https://opentofu.org/> · Pulumi — <https://www.pulumi.com/>
- Kubernetes — <https://kubernetes.io/docs/concepts/> · GitOps (Argo CD) — <https://argo-cd.readthedocs.io/>
- Tauri (lightweight desktop) — <https://tauri.app/> · Flutter — <https://flutter.dev/>
- OpenTelemetry (every language) — <https://opentelemetry.io/docs/languages/>
- AWS Well-Architected — <https://aws.amazon.com/architecture/well-architected/>
- Web performance (Core Web Vitals) — <https://web.dev/articles/vitals>
