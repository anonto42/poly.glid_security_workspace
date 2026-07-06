# The Engineering Playbook — Master Index

> A reusable, technology-agnostic framework for taking **any** project from idea to
> production-grade, maintainable, performant, secure, and scalable software.
>
> It is written as a **decision playbook**: for every topic you get a *mental model*,
> *the decision and when it applies*, *tradeoffs (pros/cons)*, an *actionable checklist*,
> and *references*. It is grounded in a real worked example — this repository
> (`fingerbridge` / `fbsy`, a native Rust agent) — but every principle is stated so it
> transfers to other stacks (Go, Node.js, Java, Python, web, mobile, desktop, infra).

---

## How to read this

This framework has a **hybrid backbone**: a *lifecycle spine* you walk in order, where
each phase links into *concern deep-dives* you return to as references.

```
LIFECYCLE SPINE (read top-to-bottom for a new project)

  IDEA ─▶ PLAN ─▶ ARCHITECT ─▶ BUILD ─▶ SHIP ─▶ OPERATE ─▶ SCALE
   │       │          │          │       │         │          │
   └───────┴──────────┴──────────┴───────┴─────────┴──────────┘
                          │
                          ▼
            CONCERN DEEP-DIVES (jump in anytime)
   Principles · Tech Selection · Tooling · Operations · Security · Scaling
```

- **Starting a new project?** Read `01` → `02` → `03`, then build, dipping into the
  concern chapters as each phase demands.
- **Looking something up?** Go straight to the concern chapter (e.g. "what's the SOLID
  letter for this?" → [04-principles](04-principles.md)).
- **Porting to another stack?** Read the matching section of [10-stack-maps](10-stack-maps.md).

---

## The chapters

| # | Chapter | Lifecycle phase | What it answers |
|---|---------|-----------------|-----------------|
| 01 | [Mindset & Lifecycle](01-mindset.md) | All | How does a senior engineer *think*? What are the mental models and quality attributes? |
| 02 | [Planning & Prioritization](02-planning.md) | Idea → Plan | What are we building, in what order, and how do we decide? |
| 03 | [Architecture & Codebase](03-architecture.md) | Architect | How is the system shaped? Where do the boundaries go? |
| 04 | [Design Principles Catalog](04-principles.md) | All (cross-cutting) | SOLID, CIA, GRASP, 12-factor, coupling/cohesion — every principle, with tradeoffs. |
| 05 | [Technology & Package Selection](05-tech-selection.md) | Architect → Build | Which language, framework, and dependencies — and how to decide? |
| 06 | [Dev Tooling & CI/CD](06-tooling.md) | Build → Ship | How do we get fast, safe, repeatable feedback and releases? |
| 07 | [Operations & Lifecycle Behaviors](07-operations.md) | Operate | Config, observability, **auto-update**, resilience, incident response. |
| 08 | [Security](08-security.md) | All (cross-cutting) | CIA, threat modeling, OWASP, secrets, supply chain. |
| 09 | [Scaling: VPS → Microservices](09-scaling.md) | Scale | How do we grow from one box to a distributed system — only when it pays? |
| 10 | [Cross-Stack Maps](10-stack-maps.md) | All | Apply this framework to Go / Node / Java / Python / web / mobile / infra. |

---

## Lifecycle ↔ concern matrix

What you actively do in each phase, and which concern chapters back it up.

| Phase | Primary activity | Concern chapters to pull from |
|-------|------------------|-------------------------------|
| **Idea** | Frame the problem, constraints, success criteria | [01](01-mindset.md) |
| **Plan** | Requirements (functional + NFRs), scope, priority, risks | [02](02-planning.md) |
| **Architect** | Boundaries, style, data, ADRs | [03](03-architecture.md), [04](04-principles.md), [05](05-tech-selection.md) |
| **Build** | Implement vertical slices, tests, quality gates | [04](04-principles.md), [05](05-tech-selection.md), [06](06-tooling.md), [08](08-security.md) |
| **Ship** | CI/CD, release, signing, rollout | [06](06-tooling.md), [08](08-security.md) |
| **Operate** | Config, logs/metrics/traces, updates, on-call | [07](07-operations.md), [08](08-security.md) |
| **Scale** | Remove bottlenecks, distribute, partition | [09](09-scaling.md), [03](03-architecture.md) |

---

## The 7 quality attributes everything is judged against

Every decision in this playbook ultimately serves one or more of these "-ilities". When
two decisions conflict, you are really trading one attribute against another — name the
trade explicitly.

1. **Maintainability** — can a new engineer change it safely in 18 months?
2. **Reliability** — does it do the right thing under failure?
3. **Performance** — is it fast and resource-efficient *enough* for the requirement?
4. **Security** — confidentiality, integrity, availability (CIA).
5. **Scalability** — can it grow with load and team size?
6. **Observability** — can you tell what it's doing from the outside?
7. **Operability** — is it cheap and safe to run, deploy, and recover?

> Full definitions and how to reason about each: [01-mindset](01-mindset.md#quality-attributes).

---

## "New project in one hour" quickstart

A condensed senior-engineer bootstrap. Each step links to the depth.

1. **Frame it.** One paragraph: problem, users, top 3 constraints, what "done" means. → [02](02-planning.md)
2. **List NFRs.** Pick target numbers for the quality attributes that matter (e.g. p99 latency, RPO/RTO, max binary size). → [02](02-planning.md#non-functional-requirements-nfrs)
3. **Choose boring tech.** Default to what your team knows and what is proven. → [05](05-tech-selection.md)
4. **Pick a style.** Almost always: **modular monolith** with clean boundaries. → [03](03-architecture.md)
5. **Lay out boundaries.** `domain` (no I/O) ← `ports` (traits) ← `adapters` (I/O) ← `application` (use cases). → [03](03-architecture.md#codebase-organization)
6. **Wire the quality gates first.** Formatter, linter, type/borrow checker, test runner, pre-commit hook, CI. → [06](06-tooling.md)
7. **Build a walking skeleton.** One end-to-end vertical slice that ships. → [02](02-planning.md#walking-skeleton)
8. **Make it observable & configurable from day one.** Structured logs + 12-factor config. → [07](07-operations.md)
9. **Threat-model the boundaries.** Bind locally, validate input, manage secrets, redact logs. → [08](08-security.md)
10. **Write the first ADR.** Record *why*, not just *what*. → [02](02-planning.md#architecture-decision-records-adrs)

---

## How this maps onto *this* repository (worked example)

`fbsy` is a native biometric-attendance bridge. It already embodies most of this playbook:

- **Architecture:** hexagonal / clean — see [`docs/CODEBASE_ARCHITECTURE_DECISION.md`](../CODEBASE_ARCHITECTURE_DECISION.md)
  and the `src/{domain,ports,adapters,application,runtime,support}` layout.
- **Tech selection:** Rust + a deliberately small crate set — see [05](05-tech-selection.md#worked-example-the-fbsy-stack).
- **Operations:** structured logging, a self-update path (`self-replace`), service install
  flows — see [07](07-operations.md).
- **Security:** local-only bind, secret redaction (`src/support/redaction.rs`), `rustls`
  TLS — see [08](08-security.md).

Use those as the "filled-in" instance; use the chapters as the reusable template.

---

## How to reuse this framework for a new project / new stack

1. Copy `docs/framework/` into the new repo.
2. In `00-INDEX.md`, replace the "worked example" section with *your* project.
3. Walk the lifecycle spine; record decisions as ADRs ([02](02-planning.md#architecture-decision-records-adrs)).
4. For a different language/platform, read [10-stack-maps](10-stack-maps.md) — the
   principles are identical; only the *tools* change.

---

## Source spine (the canonical references behind this framework)

These recur across chapters; each chapter also lists its own.

- The Twelve-Factor App — <https://12factor.net/>
- C4 model (Simon Brown) — <https://c4model.com/>
- Clean / Hexagonal architecture — <https://en.wikipedia.org/wiki/Hexagonal_architecture_(software)>
- OWASP Top 10 (2025) — <https://owasp.org/Top10/2025/en/>
- OpenTelemetry — <https://opentelemetry.io/docs/concepts/observability-primer/>
- DORA metrics — <https://dora.dev/guides/dora-metrics/>
- SLSA (supply chain) — <https://slsa.dev/> · Sigstore — <https://www.sigstore.dev/>
- The Update Framework (TUF) — <https://theupdateframework.io/>
