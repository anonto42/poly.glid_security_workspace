# 01 · Mindset & Lifecycle

> **Phase:** all · **Type:** foundation
> How a senior engineer *thinks* before they type. The mental models here are the lens
> for every later chapter.

← [Index](00-INDEX.md) · Next: [02 Planning](02-planning.md)

---

## The one-sentence definition of seniority

> A senior engineer optimizes for the **total cost of the system over its lifetime**,
> under uncertainty, by making **tradeoffs explicit** and **decisions reversible where
> possible**.

Everything below is a corollary of that sentence. Juniors optimize for "make it work
now." Seniors optimize for "make it work, keep working, and keep changeable — for the
least total effort."

---

## Core mental models

Each model below is a thinking tool. Learn to reach for them by name.

### 1. There are no solutions, only tradeoffs
Every choice buys one quality attribute by spending another (e.g. flexibility costs
simplicity; caching buys latency and spends consistency). **The skill is not picking the
"best" — it's naming what you're spending and confirming the requirement justifies it.**
When someone proposes an absolute ("always use microservices"), the senior question is
*"in exchange for what?"*

### 2. One-way vs two-way doors (reversibility)
- **Two-way door** (reversible): decide fast, with ~70% of the information. Most code,
  most libraries, most internal APIs.
- **One-way door** (hard/expensive to reverse): slow down, write an ADR, get review.
  Examples: your database engine, your public API contract, your data model, your auth
  model, your language.

> Rule: spend your decision *budget* on one-way doors. Don't agonize over reversible ones.

### 3. Boring technology / "innovation tokens"
You get a small number of innovation tokens per project. Spend them on the part that is
actually your differentiator; use proven, boring, well-understood technology for
everything else. New tech multiplies unknowns (bugs, hiring, ops, community).

### 4. YAGNI · KISS · DRY (in that priority order)
- **YAGNI** ("You Aren't Gonna Need It") — don't build for imagined futures; build for
  the known requirement plus *cheap* seams for change.
- **KISS** ("Keep It Simple") — the simplest thing that meets the requirement *and its
  NFRs* wins.
- **DRY** ("Don't Repeat Yourself") — remove duplication of **knowledge**, not of
  *characters*. Two pieces of code that look alike but change for different reasons are
  **not** duplication (see [04: principles](04-principles.md)).

> Tension: DRY taken too far creates coupling. Prefer a little duplication over the wrong
> abstraction. "Duplication is far cheaper than the wrong abstraction." — Sandi Metz.

### 5. Optimize for change, not for the first write
Code is read ~10× more than written and changed many times after that. The valuable
property is **changeability**: clear boundaries, low coupling, high cohesion, good names,
fast tests. This is why architecture ([03](03-architecture.md)) and principles
([04](04-principles.md)) matter more than cleverness.

### 6. The cost of complexity compounds
Complexity is the real enemy (Ousterhout, *A Philosophy of Software Design*). It shows up
as **cognitive load**, **change amplification** (one change touches many places), and
**unknown unknowns**. Every feature has a carrying cost forever. Removing code is a
legitimate, high-value activity.

### 7. Make it work → make it right → make it fast (in order)
Correctness first, clean structure second, performance third — and **only optimize with a
measurement in hand**. Premature optimization couples your code to assumptions that are
usually wrong. (See performance method in [09](09-scaling.md).)

### 8. Risk-first sequencing
Do the scariest, least-understood thing *first* (a "spike" / proof of concept), while the
project still has slack to change direction. Cheap to pivot early, ruinous to pivot late.

### 9. Build vs buy vs borrow
Write only what is your core value. Buy/borrow (SaaS, OSS library, managed service) for
everything undifferentiated — *unless* it's a one-way door or a critical dependency you
can't afford to have controlled by others. Evaluate the dependency like a hire
([05](05-tech-selection.md)).

### 10. Total Cost of Ownership (TCO)
The purchase price of a decision is tiny next to its running cost: ops, on-call, patching,
onboarding, the next migration. When comparing options, compare **TCO over the expected
lifetime**, not lines of code today.

### 11. Conway's Law
Systems mirror the communication structure of the org that builds them. If you want a
modular system, you need modular teams (and vice-versa). Architecture and team topology
are the same decision viewed twice.

### 12. Make the implicit explicit
Hidden assumptions are where bugs and disagreements live. Senior engineers surface them:
write the NFR number down, name the invariant, document the *why*, encode the constraint
in a type or a test so the computer enforces it.

---

## Decision-making, operationally

| Tool | Use it for | Output |
|------|-----------|--------|
| **ADR** (Architecture Decision Record) | One-way doors; anything future-you will ask "why?" about | A short, dated, immutable note: context → decision → consequences. See [02](02-planning.md#architecture-decision-records-adrs). |
| **Spike / PoC** | Reducing technical uncertainty before committing | Throwaway code + a written finding |
| **RFC / design doc** | Changes that affect others; needs buy-in | A reviewable proposal |
| **Reversibility test** | Deciding *how much process* a decision deserves | "One-way → write it down; two-way → just do it" |
| **Pre-mortem** | High-stakes plans | "Assume it failed; why?" — surfaces risks early |

> Heuristic: the amount of process should be proportional to the **cost of being wrong ×
> the difficulty of reversing**.

---

## The lifecycle spine

You will walk these phases for a new project. Each is a chapter or set of chapters.

```
IDEA ─▶ PLAN ─▶ ARCHITECT ─▶ BUILD ─▶ SHIP ─▶ OPERATE ─▶ SCALE
```

1. **Idea** — Frame the problem and constraints before solutioning. Output: a paragraph a
   stranger could understand, plus the top constraints and success criteria.
2. **Plan** — Turn the idea into requirements (functional **and** non-functional),
   scoped and prioritized, with risks named. → [02](02-planning.md)
3. **Architect** — Decide the shape: boundaries, style, data, the one-way doors. →
   [03](03-architecture.md)
4. **Build** — Implement in thin vertical slices, behind quality gates. →
   [06](06-tooling.md)
5. **Ship** — Release safely and repeatably (CI/CD, signing, staged rollout). →
   [06](06-tooling.md)
6. **Operate** — Run it: config, observability, updates, incidents. →
   [07](07-operations.md)
7. **Scale** — Remove the bottleneck that actually hurts, only when it hurts. →
   [09](09-scaling.md)

> The spine is a *default order*, not a waterfall. You loop: every build slice revisits
> plan and architecture in miniature. The point is that **each concern is consciously
> addressed, not skipped.**

---

## Quality attributes

The non-negotiable vocabulary. Every NFR you write ([02](02-planning.md)) and every
tradeoff you make targets one of these. Define each with a *number* whenever you can.

| Attribute | The question it answers | How you make it concrete (example metric) |
|-----------|------------------------|-------------------------------------------|
| **Maintainability** | Can someone change this safely later? | Cyclomatic complexity, module coupling, time-to-first-PR for a new hire |
| **Reliability** | Does it stay correct under failure? | Error budget, MTBF, success rate, data-integrity invariants |
| **Performance / Efficiency** | Fast and lean *enough*? | p50/p95/p99 latency, throughput, memory/CPU, binary size |
| **Security** | Confidential, intact, available (CIA)? | % inputs validated, secrets in vault, CVEs open, attack surface |
| **Scalability** | Grows with load *and team*? | Cost-per-request at 10×, max safe concurrency |
| **Observability** | Can you see inside from outside? | % requests traced, MTTR, "can I answer a novel question without a deploy?" |
| **Operability** | Cheap & safe to run/deploy/recover? | Deploy frequency, change-failure rate, time-to-rollback (DORA — [06](06-tooling.md)) |
| **Portability** | Moves across environments? | # of OS/arch targets, env-specific code |
| **Usability** (for the consumer — user *or* developer) | Easy to use correctly, hard to misuse? | API "pit of success", support tickets, docs coverage |

### How to reason about them
- **You cannot maximize all of them.** Pick the 2–3 that define this product and state
  target numbers; let the rest be "good enough." (A trading engine optimizes latency; a
  billing system optimizes integrity; a CLI tool optimizes usability + portability.)
- **Quality attributes are mostly architectural.** You can't bolt on performance,
  security, or observability at the end — they come from structure. That's why they're
  decided in *Architect*, not *Operate*.
- **Write them as testable NFRs.** "Fast" is a wish; "p99 < 200 ms at 500 rps" is a
  requirement you can verify and defend a tradeoff against.

---

## Anti-mindsets to catch in yourself

- **Resume-driven development** — choosing tech to learn it, not because it fits.
- **Gold-plating** — building beyond the requirement "while I'm here."
- **Cargo-culting** — copying a pattern (microservices, a design pattern, "what Google
  does") without its context or tradeoffs.
- **Hero coding** — undocumented cleverness only you understand; the opposite of
  maintainability.
- **Analysis paralysis** — applying one-way-door rigor to two-way-door decisions.
- **Premature abstraction / optimization** — paying for flexibility or speed you have no
  evidence you need.

---

## Checklist (mindset)

- [ ] I can state the problem and top 3 constraints in plain language.
- [ ] I named which decisions are one-way doors and gave those extra care.
- [ ] For each significant choice I can say *what I'm trading away*.
- [ ] I picked the 2–3 quality attributes that matter and gave them target numbers.
- [ ] I'm spending innovation tokens only where they create real value.
- [ ] I did the riskiest unknown first (spike) before committing the plan.
- [ ] I wrote down the *why* for anything future-me will question (ADR).

---

## References

- F. Brooks, *No Silver Bullet* (essential vs accidental complexity)
- J. Ousterhout, *A Philosophy of Software Design* (complexity, deep modules)
- D. Thomas & A. Hunt, *The Pragmatic Programmer* (DRY, orthogonality, tracer bullets)
- Dan McKinley, *Choose Boring Technology* — <https://boringtechnology.club/>
- Amazon shareholder letter 2015 (one-way / two-way doors)
- M. Conway, *How Do Committees Invent?* (Conway's Law)
- Sandi Metz, *The Wrong Abstraction* — <https://sandimetz.com/blog/2016/1/20/the-wrong-abstraction>
