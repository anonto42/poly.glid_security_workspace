# 02 · Planning & Prioritization

> **Phase:** Idea → Plan
> Turn a fuzzy idea into a scoped, prioritized, risk-aware plan — and capture the
> decisions so they survive.

← [01 Mindset](01-mindset.md) · [Index](00-INDEX.md) · Next: [03 Architecture](03-architecture.md)

---

## Mental model

> **Planning is risk reduction, not prediction.** You are not forecasting the future;
> you are ordering work so that the **most uncertain, most valuable, most irreversible**
> things are learned/decided first, while change is still cheap.

A plan's job is to answer four questions, in order:
1. **Why / for whom?** (problem, users, success criteria)
2. **What?** (functional + non-functional requirements)
3. **In what order?** (priority + sequencing by risk and value)
4. **How do we keep the decisions?** (ADRs, definition of done)

---

## Step 1 — Frame the problem (Idea)

Before any requirement, write a **one-paragraph brief** a non-expert could understand:

- **Problem**: what hurts today, concretely.
- **Users / actors**: who, and what they're trying to accomplish.
- **Constraints**: the top 3–5 hard limits (budget, deadline, team skills, platform,
  compliance, offline/online, must-run-on-customer-LAN, etc.).
- **Success criteria**: how you'll know it worked — ideally measurable.
- **Non-goals**: what you are explicitly *not* doing (this is as important as the goals).

> *fbsy example:* "Sync biometric attendance from on-prem ZKTeco devices to an HRMS via
> webhook, from a single native binary running on an office LAN machine, with no data loss
> and no manual steps. Non-goals: cloud multi-tenancy, a GUI."

---

## Step 2 — Requirements

### Functional requirements
What the system *does* — behaviors, as user-centric statements. Capture them as **user
stories** ("As a payroll admin, I can …, so that …") or **use cases**. Each should have
**acceptance criteria** (Given/When/Then) so "done" is testable, not opinion.

### Non-functional requirements (NFRs)
The qualities the system must have — these are the [quality attributes](01-mindset.md#quality-attributes)
made *concrete with numbers*. NFRs are where most projects fail silently, because they're
left implicit.

> Write each NFR as **attribute + metric + target + condition**.

| Bad (a wish) | Good (an NFR you can verify & defend) |
|--------------|----------------------------------------|
| "It should be fast." | "p99 sync cycle < 5 s for 5,000 records." |
| "It should be reliable." | "Zero attendance loss: never clear the device until the webhook 2xx is confirmed." |
| "It should be secure." | "Local API binds 127.0.0.1 only; all secrets redacted from logs." |
| "It should be small." | "Single static binary ≤ 15 MB, no runtime deps." |
| "It should be observable." | "Every sync emits a structured log line with a correlation id." |

Common NFR categories to *always consider* (skip explicitly, never by accident):
performance, reliability/availability, security, observability, operability,
maintainability, portability, compliance/legal, cost, accessibility, i18n.

---

## Step 3 — Scope it small

### MVP, but honest
The Minimum **Viable** Product is the smallest thing that delivers real value *and meets
its NFRs* — not the smallest thing that compiles. "Viable" includes "safe" and
"operable."

### Walking skeleton
Build a **walking skeleton** first: a tiny end-to-end implementation that exercises every
architectural layer and *actually ships/deploys*, even if each step is trivial.

> *fbsy:* read config → connect to one device → map one record → POST to a stub HRMS →
> log the result, wired through `cli → application → ports → adapters`. Once that runs and
> deploys, every later feature is an *increment*, not an integration gamble.

Why it beats horizontal layering: it de-risks integration (the riskiest part) on day one,
gives you a deployable artifact immediately, and validates the architecture against
reality.

### Vertical slices
Grow by **vertical slices** (a complete thin feature through all layers), not horizontal
layers (all of the DB, then all of the API…). Slices ship value and keep the system
always-releasable.

---

## Step 4 — Prioritize

You have more ideas than time. Use an explicit framework so priority is a *decision*, not
the loudest voice. Pick one and apply it consistently.

| Framework | How it works | Best for |
|-----------|-------------|----------|
| **MoSCoW** | Must / Should / Could / Won't (this release) | Fast scope-cutting, stakeholder alignment |
| **RICE** | score = (Reach × Impact × Confidence) ÷ Effort | Comparing many features objectively |
| **Kano** | Basic vs Performance vs Delight needs | Deciding *what kind* of value a feature adds |
| **Eisenhower** | Urgent×Important matrix | Personal/triage, "do/schedule/delegate/drop" |
| **WSJF** (SAFe) | Cost of Delay ÷ job size | Sequencing when delay has real cost |
| **Cost of Delay** | What does waiting cost per week? | Prioritizing by economics, not gut |

> Sequencing rule (overrides raw priority): **risk-first, then value-first.** Do the
> highest-uncertainty item early even if its value rank is lower — learning is the point.

### Tech debt is a backlog item, not a sin
Debt is sometimes the *right* leverage (ship now, pay later) — if it's *recorded* and
*deliberate*. Track it like any work; budget ~15–20% of capacity to repay it. Untracked,
accidental debt is the dangerous kind. Make the interest visible: "this hack costs us ~1
day per feature in module X."

---

## Step 5 — Estimate & sequence (lightly)

- Estimate **relatively** (S/M/L or points), not in false-precision hours; humans are bad
  at absolute estimates.
- Decompose until each item is ≤ a few days; big estimates mean "split further or spike."
- Budget for the **invisible work**: testing, docs, review, CI, deployment, error paths,
  observability. A feature isn't the happy path.
- Plan in **thin, shippable increments** with milestones; prefer a steady cadence over a
  big-bang date.

---

## Architecture Decision Records (ADRs)

The single highest-leverage planning habit. An ADR is a short, **immutable, dated** record
of one significant decision. It captures the thing code can't: *why*.

**When to write one:** any one-way door, any choice you debated, anything future-you will
ask "why on earth did we…?" about. (This repo already does this well — see
[`docs/CODEBASE_ARCHITECTURE_DECISION.md`](../CODEBASE_ARCHITECTURE_DECISION.md).)

**Template** (keep it to one page):

```markdown
# ADR-000N: <short title>
Date: YYYY-MM-DD
Status: Proposed | Accepted | Superseded by ADR-00MM

## Context
The forces at play: requirements, constraints, what we know and don't.

## Decision
What we are doing, stated plainly.

## Alternatives considered
Option A / B / C — and why we did NOT pick them.

## Consequences
Positive: …
Negative / costs: …
What this makes harder later: …
```

> Rule: ADRs are append-only. You don't edit a decision — you supersede it with a new ADR
> that references the old one. The history *is* the value.

---

## Definition of Done (DoD)

Agree, once, what "done" means for *every* item — so quality isn't negotiated per-ticket:

- [ ] Meets acceptance criteria
- [ ] Tests written and passing (incl. an error/edge path)
- [ ] Passes all quality gates (format, lint, type/borrow check) — [06](06-tooling.md)
- [ ] Observable (logs/metrics for the new behavior) — [07](07-operations.md)
- [ ] Security considered (input validation, secrets, least privilege) — [08](08-security.md)
- [ ] Docs / ADR updated
- [ ] Reviewed
- [ ] Deployable (no manual steps) and behind a flag if risky

---

## Checklist (planning)

- [ ] One-paragraph brief written: problem, users, constraints, success, **non-goals**.
- [ ] Functional requirements have testable acceptance criteria.
- [ ] NFRs written as attribute+metric+target — for every relevant quality attribute.
- [ ] Scope cut to an MVP that is viable *and* safe; a walking skeleton is the first build.
- [ ] Priority decided with an explicit framework; sequence is risk-first.
- [ ] Tech debt and "invisible work" are in the backlog, not assumed-free.
- [ ] One-way-door decisions captured as ADRs.
- [ ] A shared Definition of Done exists.

---

## References

- 12factor / NFR thinking — <https://12factor.net/>
- RICE — Intercom — <https://www.intercom.com/blog/rice-simple-prioritization-for-product-managers/>
- MoSCoW — DSDM — <https://www.agilebusiness.org/dsdm-project-framework/moscow-prioritisation.html>
- Kano model — <https://en.wikipedia.org/wiki/Kano_model>
- ADRs — Michael Nygard — <https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions>
  · collection: <https://adr.github.io/>
- Walking skeleton — Alistair Cockburn — <https://wiki.c2.com/?WalkingSkeleton>
- User Story Mapping — Jeff Patton
