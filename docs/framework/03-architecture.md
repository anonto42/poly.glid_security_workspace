# 03 · Architecture & Codebase Organization

> **Phase:** Architect
> The shape of the system: where the boundaries go, which way dependencies point, and how
> the source tree reflects that. Architecture is the set of decisions that are **expensive
> to change** — so this is where you spend care.

← [02 Planning](02-planning.md) · [Index](00-INDEX.md) · Next: [04 Principles](04-principles.md)

---

## Mental model

> **Architecture = the boundaries and the dependency directions.** Not the framework, not
> the folders, not the diagram — those are consequences. The architecture *is* the answer
> to "what can change without forcing a change somewhere else?"

Two forces define every architecture:
- **Coupling** — how much one part must know about another. Minimize it *across*
  boundaries.
- **Cohesion** — how much the things inside one part belong together. Maximize it *within*
  a boundary.

Good architecture = **low coupling between modules, high cohesion within them**, and a
**dependency direction that points toward stability** (volatile details depend on stable
policy, never the reverse). Everything else in this chapter is a technique for achieving
that.

---

## Describe before you decide: the C4 model

Before choosing a *style*, learn to *draw* the system at the right altitude. The **C4
model** (Simon Brown) gives four zoom levels — most teams only ever need the first two.

| Level | Shows | Audience |
|-------|-------|----------|
| **1. Context** | Your system as one box + users + external systems it talks to | Everyone, incl. non-technical |
| **2. Container** | Deployable/runnable units (app, db, queue, SPA, CLI) and how they communicate | Architects, ops |
| **3. Component** | Major components inside one container and their responsibilities | Developers of that container |
| **4. Code** | Classes/types (usually generated, rarely hand-drawn) | Only when needed |

> Use it to *think* and to *communicate*. A Context + Container diagram in your README
> answers 80% of "how does this work?" questions and is the backbone of threat modeling
> ([08](08-security.md)). Notation- and tool-independent; "diagrams as code" via
> Structurizr/Mermaid keeps them in version control.

---

## Choosing an architectural style

Styles are points on a spectrum from "one process, simple" to "many processes,
independently scalable & deployable." **Default to the simplest that meets your NFRs**;
move right only when a quality attribute forces you.

| Style | What it is | Choose when | Cost / cons |
|-------|-----------|-------------|-------------|
| **Script / single file** | No structure | Throwaway, glue, <500 LOC | No boundaries; doesn't survive growth |
| **Layered (n-tier)** | UI → service → data access layers | Simple CRUD apps, familiar teams | Layers leak; business logic drifts into UI/DB; hard to test in isolation |
| **Hexagonal (Ports & Adapters)** | Core logic + ports (interfaces) + adapters (I/O impls) | You want testable core decoupled from I/O; most non-trivial apps | More interfaces/indirection |
| **Clean / Onion** | Concentric layers; deps point inward to entities/use-cases | Complex business rules living 5+ years | Boilerplate; overkill for CRUD |
| **Modular monolith** | One deployable, strong internal module boundaries | **The 2026 default for new products** | Discipline required to keep boundaries; one deploy unit |
| **Event-driven** | Components communicate via events/messages | Decoupling producers/consumers, async workflows, spiky load | Eventual consistency, harder to trace/debug |
| **Microservices** | Independently deployable services per capability | Org > ~30–50 engineers, parts need independent scale/deploy/failure isolation | Distributed-systems tax: network, data consistency, ops, observability |
| **Serverless / FaaS** | Functions on managed runtime | Spiky/low traffic, event glue, minimal ops | Cold starts, vendor lock-in, local-dev friction, limits |

> **The senior default:** a **modular monolith with hexagonal/clean internals.** It gives
> you clean boundaries (so it's testable and changeable) *and* one simple deploy unit (so
> it's cheap to operate) — and it can be split into services later *exactly along the
> module seams* if scale demands. See [09 Scaling](09-scaling.md) for when to split, and
> the [decision in this repo](../CODEBASE_ARCHITECTURE_DECISION.md).

### Clean vs Hexagonal — practical reconciliation
They're cousins, and most production code uses **both at once**: Hexagonal's external
vocabulary (driving vs driven **ports**, with **adapters** implementing them) and Clean's
internal layering (entities/use-cases at the center). Rule from current practice: *no
domain entity crosses a port boundary* — pass primitives/DTOs. Accept the extra-interfaces
cost gladly for complex, long-lived business rules; **skip it for simple CRUD where the
database is the business logic.**

---

## The dependency rule (the one rule to remember)

> **Source-code dependencies always point inward, toward stable, high-level policy. Inner
> layers know nothing about outer layers.**

```
   driving adapters            driven adapters
   (CLI, HTTP, UI)             (DB, HTTP client, files, devices)
        │                              ▲
        ▼                              │  implements
   ┌─────────────────────────────────────────────┐
   │ APPLICATION  (use cases / orchestration)     │
   │   ┌───────────────────────────────────────┐  │
   │   │ DOMAIN  (entities, value objects,      │  │   ← depends on NOTHING
   │   │          rules, invariants)            │  │      external
   │   └───────────────────────────────────────┘  │
   │   depends only on ► PORTS (interfaces/traits) │
   └─────────────────────────────────────────────┘
```

- **Domain** depends on nothing (no framework, no I/O, no DB, no HTTP). Pure logic and
  types. This is what makes it trivially testable and durable.
- **Ports** are interfaces *owned by the inside*, expressing what the core needs ("give me
  attendance records", "send this event").
- **Adapters** (outside) *implement* ports for a specific technology. They are the only
  place that imports `reqwest`/`axum`/a DB driver/the ZKTeco protocol.
- The arrow from a low-level detail to a high-level policy is achieved with **Dependency
  Inversion** ([04: the "D" in SOLID](04-principles.md#solid)).

This is the architectural-scale version of the principles in [04](04-principles.md). Get
this right and most of the "-ilities" follow for free.

---

## Domain-Driven Design (when the domain is the hard part)

DDD is how you find *where the boundaries should go* and *what the inside should say*. Use
its strategic patterns always (cheap, high value); reach for the full tactical set only
when business complexity justifies it.

### Strategic DDD (the high-leverage half)
- **Ubiquitous language** — one shared vocabulary used identically in conversation, docs,
  and code. If the business says "attendance event," the type is `AttendanceEvent`, not
  `Record2`.
- **Domain & subdomains** — find your **core domain** (your differentiator — invest
  here), vs *supporting* and *generic* subdomains (buy/borrow these — [05](05-tech-selection.md)).
- **Bounded context** — an explicit boundary within which a model and its language are
  consistent. The *same word* ("customer") can mean different things in different
  contexts; don't force one shared model. **Bounded contexts are your future
  service/module seams.**
- **Context mapping** — how contexts relate (partnership, shared kernel, customer-supplier)
  and the **Anti-Corruption Layer (ACL)**: a translation boundary that stops a messy
  external model (legacy system, third-party API) from polluting your clean one. (fbsy's
  adapters act as an ACL around the ZKTeco protocol and the HRMS API.)

### Tactical DDD (inside one bounded context)
- **Entity** — has identity over time (a `Device`, a `User`).
- **Value object** — defined by its attributes, immutable, no identity (a `Money`, a
  `TimeRange`). Prefer these; they make illegal states unrepresentable.
- **Aggregate** + **aggregate root** — a cluster of objects treated as one **consistency
  boundary**; outsiders touch it only via the root, which guards its invariants. Keep
  aggregates *small*.
- **Domain event** — something meaningful that happened (`AttendanceRecorded`). The unit
  of an event-driven system.
- **Repository** — a port for persisting/loading aggregates (collection-like interface,
  implemented by an adapter).
- **Domain service** — domain logic that doesn't naturally belong to one entity.

---

## Codebase organization

How the *source tree* expresses the architecture. Two orthogonal choices:

### A) Package by **layer** vs by **feature**
- **By layer** (`controllers/`, `services/`, `models/`): easy to start, but a single
  feature is smeared across the tree and changes touch every folder (low cohesion).
- **By feature / by domain** (`attendance/`, `billing/`, `devices/`, each containing its
  own layers): high cohesion, changes are local, scales to many people and is the natural
  precursor to service extraction. **Prefer this as the system grows.**

> This repo uses a hybrid: top-level layers (`domain/ ports/ adapters/ application/`) with
> *feature files* inside each (`attendance.rs`, `device.rs`, …). For small, single-purpose
> agents this is clean; for a large product, package-by-feature wins.

### B) Monorepo vs polyrepo
| | Monorepo | Polyrepo |
|---|---------|----------|
| Atomic cross-cutting change | ✅ one commit | ❌ coordinated PRs |
| Shared tooling / refactor | ✅ easy | ❌ duplicated |
| Independent versioning / access control | ❌ harder | ✅ natural |
| Tooling scale | needs build system (Bazel/Nx/Turborepo) at size | simple per repo |

> Default: **monorepo** for one team/product; split to polyrepo when teams need
> independent release cadence and ownership.

### Worked example: this repo's layout
```
src/
├── main.rs         # binary entrypoint only — wires & launches
├── lib.rs          # library root (so everything is testable)
├── cli/            # driving adapter: arg parsing, terminal UX  ─┐
├── application/    # use cases: sync_once, serve, doctor, update │ orchestration
├── domain/         # entities, events, sync_result — NO I/O      │ ← stable core
├── ports/          # traits: device, hrms, config_store          │ interfaces
├── adapters/       # driven: zkteco_tcp, hrms_reqwest, config_file│ I/O details
├── runtime/        # scheduler, process lifecycle, shutdown       │
├── config/         # typed config model + validation             │
└── support/        # cross-cutting: logging, paths, redaction    ┘
```
Dependency direction (enforced by review and module visibility):
`cli/runtime → application → ports/domain` and `adapters → ports/domain`; **`domain →
nothing.`**

---

## Data architecture (decide early — it's a one-way door)

- **Where does state live?** Stateless compute + state in a datastore is the scalable
  default ([09](09-scaling.md), [12-factor](04-principles.md#twelve-factor)).
- **SQL vs NoSQL:** default to a **relational DB** (Postgres) — transactions, constraints,
  and flexibility cover most needs; reach for NoSQL/specialized stores for a *specific*
  access pattern (KV cache, document, time-series, search, graph).
- **Schema is a contract.** Use **versioned migrations** checked into the repo; never
  hand-edit production schemas. Plan migrations to be backward-compatible
  (expand → migrate → contract).
- **Own your data per boundary.** In a modular monolith, modules shouldn't reach into each
  other's tables; in microservices, a service owns its data exclusively (no shared DB).
- **Consistency model** is an architectural choice with user-visible consequences — see
  [CAP/PACELC in 04](04-principles.md#cap--pacelc) and [09](09-scaling.md).

---

## Checklist (architecture)

- [ ] I drew a C4 Context + Container diagram (even rough).
- [ ] I chose the **simplest style** meeting the NFRs (likely modular monolith).
- [ ] Dependencies point inward; the domain imports no framework/I/O.
- [ ] External systems are reached only through **ports**, implemented by **adapters**.
- [ ] Boundaries follow **bounded contexts** / cohesive features, not technical layers only.
- [ ] Aggregates/invariants are identified; illegal states are hard to represent.
- [ ] Data ownership, store choice, and migration strategy are decided and recorded (ADR).
- [ ] The module seams are where I'd split into services *if* I ever had to.

---

## References

- R. C. Martin, *Clean Architecture*
- A. Cockburn, *Hexagonal (Ports & Adapters)* — <https://alistair.cockburn.us/hexagonal-architecture/>
- C4 model — <https://c4model.com/>
- E. Evans, *Domain-Driven Design*; V. Vernon, *Implementing DDD*
- M. Fowler, *Patterns of Enterprise Application Architecture* & monolith-first — <https://martinfowler.com/bliki/MonolithFirst.html>
- Clean vs Hexagonal (2025) — <https://topictrick.com/blog/clean-vs-hexagonal-architecture>
- Modular monolith vs microservices (2025) — <https://www.ness.com/blog/modular-monolith-vs-microservices/>
- This repo's decision — [`docs/CODEBASE_ARCHITECTURE_DECISION.md`](../CODEBASE_ARCHITECTURE_DECISION.md)
