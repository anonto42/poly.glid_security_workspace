# 04 · Design Principles Catalog

> **Phase:** all (cross-cutting) · **Type:** reference
> The complete, organized catalog of the principles a senior engineer reasons with —
> CIA, SOLID, GRASP, component/structural principles, 12-factor, distributed-systems laws
> — each with *what it is*, *why*, *the tradeoff*, and *how to apply it*.

← [03 Architecture](03-architecture.md) · [Index](00-INDEX.md) · Next: [05 Tech Selection](05-tech-selection.md)

---

## How to use principles (read this first)

> **Principles are heuristics, not laws.** They are compressed experience. They will
> **conflict** with each other, and the skill is arbitration, not obedience.

- Every principle exists to serve a [quality attribute](01-mindset.md#quality-attributes)
  (mostly *maintainability* via low coupling / high cohesion).
- Applying a principle has a **cost** (usually indirection/complexity). Apply it when the
  benefit (changeability, safety) exceeds that cost — i.e. for code that will *actually*
  change or matter.
- When two principles disagree, fall back to the [mindset chapter](01-mindset.md):
  *what am I trading, and does the requirement justify it?*

The grand unifying pair underneath nearly all of them:

> **Low coupling, high cohesion.** *Coupling* = degree of interdependence between modules
> (minimize across boundaries). *Cohesion* = degree to which a module's parts belong
> together (maximize within). If you remember only one thing, remember this.

---

## Foundational principles (the everyday set)

| Principle | Statement | Tradeoff / failure mode if overdone |
|-----------|-----------|--------------------------------------|
| **DRY** — Don't Repeat Yourself | Every piece of *knowledge* has one authoritative representation. | Over-applied → premature abstraction & coupling. DRY is about *knowledge*, not identical-looking code. |
| **KISS** — Keep It Simple | Prefer the simplest design that meets the requirement + NFRs. | "Simple" ≠ "easy" or "few lines"; sometimes simple needs more code. |
| **YAGNI** — You Aren't Gonna Need It | Don't build for speculative futures. | Don't use it to skip *cheap seams* for known-likely change. |
| **SoC** — Separation of Concerns | Each part addresses one concern (UI ≠ business ≠ persistence). | Too-fine separation → fragmentation & ceremony. |
| **Single Source of Truth (SSOT)** | One authoritative place for each fact/config/schema. | Requires discipline to avoid convenient copies. |
| **POLA** — Principle of Least Astonishment | Behave the way a reasonable user/dev expects. | Sometimes the "expected" thing is wrong; document deviations. |
| **Fail fast** | Detect and surface errors at the earliest point (validate at the boundary). | Must be paired with graceful handling at the *right* layer ([07](07-operations.md)). |
| **Composition over inheritance** | Build behavior by combining small parts, not deep class trees. | Can scatter logic if taken to extremes; prefer for flexibility. |
| **Law of Demeter** ("don't talk to strangers") | An object should only call its immediate collaborators (`a.b()` not `a.b().c().d()`). | Overzealous → wrapper bloat. Targets coupling. |
| **Encapsulation / information hiding** | Hide internals behind a small, stable interface (Parnas). | The basis of "deep modules": simple interface, hidden complexity. |
| **Make illegal states unrepresentable** | Use the type system so bad states can't be constructed. | Strongest in typed languages; invest where correctness matters. |
| **Idempotency** | Doing an operation twice == doing it once. | Essential for retries & at-least-once delivery ([07](07-operations.md), [09](09-scaling.md)). |

---

## SOLID

Five object/module-design principles (Robert C. Martin). They are really *five ways to
manage coupling and cohesion at the class/module level.*

| | Name | Statement | In practice | Tradeoff |
|---|------|-----------|-------------|----------|
| **S** | Single Responsibility | A module should have **one reason to change** (one actor/stakeholder it answers to). | Split the class that changes for both "billing rules" and "report format." | Over-split → too many tiny classes; judge by *reasons to change*, not verbs. |
| **O** | Open/Closed | Open for extension, **closed for modification** — add behavior without editing tested code. | Add a new `Adapter`/strategy rather than `if/else`-ing an existing one. | Speculative extension points = YAGNI violation; add the seam when a 2nd case appears. |
| **L** | Liskov Substitution | A subtype must be usable anywhere its base type is, without surprises. | A `Square` that breaks `Rectangle`'s contract violates LSP. Honor pre/post-conditions. | Forces honest hierarchies; favors composition when "is-a" is a lie. |
| **I** | Interface Segregation | Many small, client-specific interfaces beat one fat one. | Split `Repository` into `Reader`/`Writer` if clients need only one. | Too granular → interface sprawl. |
| **D** | Dependency Inversion | Depend on **abstractions**, not concretions; high-level policy must not depend on low-level detail. | The architectural **ports & adapters** rule ([03](03-architecture.md#the-dependency-rule-the-one-rule-to-remember)). | Indirection cost; worth it at real boundaries, noise inside a leaf module. |

> SOLID is the *class/module-scale* statement of the same idea Clean/Hexagonal architecture
> states at the *system scale*: stable abstractions in the center, volatile details at the
> edges, dependencies pointing inward.

---

## GRASP — assigning responsibilities

GRASP (General Responsibility Assignment Software Patterns, Craig Larman) answers the
question SOLID doesn't: *which object should do this?* Nine patterns:

1. **Information Expert** — assign a responsibility to the class that has the data to do it.
2. **Creator** — the class that aggregates/contains/closely-uses B should create B.
3. **Controller** — a non-UI object that receives and coordinates a system operation (≈ a
   use case in [03](03-architecture.md)).
4. **Low Coupling** — assign responsibilities to keep dependencies low. (Meta-goal.)
5. **High Cohesion** — keep each element's responsibilities focused. (Meta-goal.)
6. **Polymorphism** — handle type-based variation with polymorphism, not conditionals.
7. **Pure Fabrication** — invent a service class (not from the domain) to keep cohesion
   high / coupling low (e.g. a `Mapper`, a `Repository`).
8. **Indirection** — introduce a mediator to decouple two things (adapters, events).
9. **Protected Variations** — wrap predicted points of variation behind a stable interface
   (the "why" behind Open/Closed and Dependency Inversion).

> SOLID tells you *how* a module should be shaped; GRASP tells you *where to put the
> responsibility* in the first place.

---

## Component principles (package/module scale — Uncle Bob)

Above the class, below the system: how to group classes into **components** (packages,
crates, modules, libraries). Two triads.

### Component **cohesion** — what goes *together* in a component
- **REP — Reuse/Release Equivalence:** the unit of reuse is the unit of release; a
  component must be releasable & versioned as a whole.
- **CCP — Common Closure:** classes that change *for the same reason at the same time*
  belong in the same component. (SRP for components.)
- **CRP — Common Reuse:** classes used together belong together; don't force consumers to
  depend on things they don't use. (ISP for components.)

> These three pull against each other (the "tension triangle"). Early projects favor CCP
> (group by change); mature, reusable libraries shift toward REP/CRP.

### Component **coupling** — how components depend on each other
- **ADP — Acyclic Dependencies:** the dependency graph must have **no cycles**. Cycles
  make components un-buildable/un-testable in isolation. Break them with Dependency
  Inversion or a new component.
- **SDP — Stable Dependencies:** depend in the direction of **stability** — volatile
  components depend on stable ones, never the reverse.
- **SAP — Stable Abstractions:** a component should be as **abstract** as it is **stable**
  (stable components should be abstract — interfaces/policy — so they can stay stable while
  being extended).

> SDP + SAP together are why your `domain`/`ports` (stable, abstract) sit at the center and
> your `adapters` (volatile, concrete) point at them.

---

## CIA

*The security triad.* The foundation of [08 Security](08-security.md). Every security control serves one of
three properties; every threat attacks one.

| | Property | Guarantees | Attacked by | Defended with |
|---|----------|-----------|-------------|---------------|
| **C** | **Confidentiality** | Only authorized parties can read the data | Eavesdropping, leaks, over-broad access | Encryption (in transit/at rest), access control, **least privilege**, secret redaction |
| **I** | **Integrity** | Data/code isn't altered without detection | Tampering, MITM, supply-chain injection | Hashing/signatures, input validation, **immutability**, code signing, transactions |
| **A** | **Availability** | Authorized users can access it when needed | DoS, outages, data loss | Redundancy, rate limiting, backups, graceful degradation, capacity planning |

> Some add **Authenticity** and **Non-repudiation** (the "AAA"/Parkerian extensions).
> Security principles that operationalize CIA — *least privilege, defense in depth, secure
> by default, fail-safe defaults, zero trust, complete mediation* — live in
> [08 Security](08-security.md).

---

## Twelve-Factor

*The Twelve-Factor App (and beyond).* The canonical checklist for apps that are portable, scalable, and operable across
environments. Still the backbone of cloud-native design; treat each as a default.

| # | Factor | One-line rule |
|---|--------|---------------|
| 1 | **Codebase** | One codebase in version control, many deploys. |
| 2 | **Dependencies** | Declare & isolate explicitly (lockfile); no system-wide assumptions. |
| 3 | **Config** | Store config in the **environment**, not in code. |
| 4 | **Backing services** | Treat DBs/queues/caches as attached, swappable resources (via URLs/creds). |
| 5 | **Build, release, run** | Strictly separate the three stages; releases are immutable & versioned. |
| 6 | **Processes** | Run as **stateless** processes; persist state in backing services. |
| 7 | **Port binding** | Export services by binding to a port; be self-contained. |
| 8 | **Concurrency** | Scale out via the process model (more processes), not just bigger threads. |
| 9 | **Disposability** | Fast startup, **graceful shutdown**; robust to sudden death. |
| 10 | **Dev/prod parity** | Keep dev, staging, prod as similar as possible. |
| 11 | **Logs** | Treat logs as **event streams** to stdout; let the platform route them. |
| 12 | **Admin processes** | Run admin/one-off tasks as identical, one-off processes. |

> **Beyond 12 (the 15-factor extensions, Kevin Hoffman):** *API first*, *Telemetry*
> (observability is a first-class factor — [07](07-operations.md)), and *Authentication &
> authorization* as a built-in concern ([08](08-security.md)). Newer "16-factor for AI"
> work adds factors for non-determinism, conversational memory, and AI-specific security.

> *Note for non-server software:* even a native agent like `fbsy` benefits from the spirit
> of these — env/file config separate from code, structured logs to stdout, graceful
> shutdown, stateless-between-runs design.

---

## Distributed-systems laws (when you have a network)

Only relevant once state is shared across processes/nodes ([09 Scaling](09-scaling.md)),
but the laws are non-negotiable once you're there.

### CAP & PACELC
- **CAP:** during a **network Partition**, you can keep at most one of **Consistency** or
  **Availability** — pick **CP** (refuse to serve possibly-stale data) or **AP** (serve,
  reconcile later). Partition tolerance isn't optional on a real network, so the real
  choice is C-vs-A *during a partition*.
- **PACELC** (the more useful refinement): **if P**artition → choose **A** or **C**;
  **E**lse (normal operation) → choose **L**atency or **C**onsistency. *Even with no
  partition, stronger consistency costs latency.* This is the everyday tradeoff (e.g.
  reading from a replica is faster but possibly stale).

### Companion principles
- **Eventual consistency** — replicas converge *over time*; design the UX for it (it's a
  product decision, not just a DB setting).
- **Idempotency + at-least-once** — networks retry, so design operations to tolerate
  duplicates (idempotency keys).
- **Fallacies of distributed computing** — the network is **not** reliable, zero-latency,
  infinite-bandwidth, secure, or free. Every cross-process call can fail or hang → it
  needs a timeout, a retry policy, and a fallback ([07](07-operations.md)).
- **CALM / commutativity, sagas, the outbox pattern** — patterns for consistency without
  distributed transactions; reach for them only at the microservices stage.

---

## When principles conflict — arbitration

Real situations where they fight, and the senior call:

| Conflict | Resolution heuristic |
|----------|----------------------|
| **DRY vs decoupling** | Prefer a little duplication over coupling two things that change for *different* reasons. "The wrong abstraction" is costlier than duplication. |
| **YAGNI vs Open/Closed** | Don't add extension points speculatively. Add the seam when the **second** case actually appears ("rule of three"). |
| **KISS vs DRY** | If removing duplication makes the code harder to understand, keep it simple and duplicated. |
| **Performance vs clean structure** | Structure first; optimize the *measured* hot path, and isolate the ugly fast code behind a clean interface. |
| **Consistency (PACELC-C) vs latency** | Driven by the NFR: money/inventory → consistency; feeds/search/recommendations → latency. |
| **Security vs usability** | Make the secure path the *easy* path (secure by default), so you don't have to trade them. |

---

## Anti-patterns (smells the principles exist to prevent)

- **Big Ball of Mud** — no discernible structure; everything coupled to everything.
- **God object / God module** — one thing knows/does too much (violates SRP, cohesion).
- **Spaghetti / shotgun surgery** — one change forces edits in many places (high coupling).
- **Anemic domain model** — data with no behavior; all logic leaks into "services."
- **Circular dependencies** — violates ADP; un-testable in isolation.
- **Premature abstraction / optimization** — paying for flexibility/speed with no evidence.
- **Leaky abstraction** — the interface forces you to know the implementation.
- **Magic / hidden control flow** — astonishing behavior (violates POLA).
- **Distributed monolith** — microservices that must deploy together (worst of both worlds).

---

## Checklist (principles)

- [ ] Modules are **low-coupling, high-cohesion**; each has one reason to change.
- [ ] Dependencies point toward stable abstractions; **no dependency cycles** (ADP).
- [ ] Interfaces are small and client-specific (ISP); I depend on abstractions (DIP).
- [ ] Duplication removed is *knowledge*, not just similar-looking code.
- [ ] I applied principles where change/risk is real — not as ceremony everywhere.
- [ ] CIA considered for anything touching data or trust boundaries.
- [ ] 12-factor defaults honored (config in env, stateless, logs to stdout, graceful shutdown).
- [ ] If distributed: every remote call has timeout + retry; consistency model is a conscious choice.

---

## References

- R. C. Martin — *Clean Code*, *Clean Architecture*, *Agile Principles, Patterns, and
  Practices* (SOLID + component principles)
- C. Larman — *Applying UML and Patterns* (GRASP)
- D. Parnas — *On the Criteria To Be Used in Decomposing Systems into Modules* (information hiding)
- The Twelve-Factor App — <https://12factor.net/> · K. Hoffman, *Beyond the Twelve-Factor App*
- CIA triad — <https://en.wikipedia.org/wiki/Information_security#Key_concepts>
- CAP — <https://en.wikipedia.org/wiki/CAP_theorem> · PACELC — D. Abadi
- Fallacies of Distributed Computing — P. Deutsch
- Sandi Metz, *The Wrong Abstraction* — <https://sandimetz.com/blog/2016/1/20/the-wrong-abstraction>
