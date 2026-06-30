# 09 · Scaling: Single VPS → Microservices

> **Phase:** Scale
> How to grow from one process on one box to a distributed system — **one rung at a time,
> and only when a real bottleneck forces it.** Most systems never need most of this ladder,
> and that's a success, not a failure.

← [08 Security](08-security.md) · [Index](00-INDEX.md) · Next: [10 Stack Maps](10-stack-maps.md)

---

## Mental model

> **Scale the bottleneck that actually hurts, only when it hurts, and measure before and
> after.** Premature scaling is the most expensive form of premature optimization — it
> buys complexity and operational cost against a problem you don't have.

Three laws to keep in mind:
- **Measure first.** You cannot scale what you haven't profiled. The bottleneck is almost
  never where you guessed.
- **Vertical before horizontal.** A bigger box (more CPU/RAM) is dramatically simpler than
  a fleet. Exhaust it first — modern single servers are *enormous*.
- **Statelessness is the key that unlocks horizontal scale.** Push state out of the compute
  tier ([12-factor #6](04-principles.md#twelve-factor)); then you can run N copies behind a
  load balancer trivially.

> **The two kinds of scale.** *Load scale* (more traffic/data) is what this chapter is
> mostly about. *Team/organizational scale* (more engineers) is the **other** reason to
> split a system — and often the *real* driver of microservices (Conway's Law,
> [01](01-mindset.md)). Don't confuse "we have lots of traffic" with "we have lots of
> teams"; they need different solutions.

---

## The scaling ladder

Climb only as far as your NFRs require. Each rung adds capability **and** operational cost.

```
0  One process on one VPS                         ← start here. Most apps live here happily.
1  Vertical scale (bigger VPS)
2  Split tiers on the box (app vs DB) → separate hosts
3  Managed backing services (DB, cache, object store, queue)
4  Horizontal app tier (load balancer + N stateless instances)
5  Caching everywhere (CDN, app cache, DB read replicas)
6  Async & queues (offload slow work; event-driven)
7  Data scaling (read replicas → partitioning/sharding → CQRS)
8  Modular monolith → extract a service (along a bounded context)
9  Microservices + platform (orchestration, mesh, central observability)
10 Multi-region / global (geo-distribution, edge, data locality)
```

### Rung-by-rung: trigger, move, and cost

| Rung | Symptom that triggers it | What you do | What it costs you |
|------|--------------------------|-------------|-------------------|
| **0–1** | Nothing / occasional CPU/RAM pressure | Run one good process; then resize the box | Almost nothing — *stay here as long as you can* |
| **2** | App and DB fighting for the same resources | Move DB to its own host | One more thing to operate + network hop |
| **3** | Ops toil on DB/cache/storage; want HA | Adopt managed Postgres/Redis/S3/queue | $ + some lock-in (good trade early) |
| **4** | One instance maxed; need HA/zero-downtime deploy | Load balancer + multiple **stateless** instances; sessions → shared store | Must be stateless; LB + health checks |
| **5** | Repeated reads dominate; latency/DB load high | CDN for static; cache hot data; DB **read replicas** | **Cache invalidation** (a hard problem); stale reads |
| **6** | Slow work blocks requests; spiky load | Queue + workers; do work async; emit events | Eventual consistency; queue ops; harder debugging |
| **7** | Single DB is the wall (writes/size) | Read replicas → **partition/shard** by key → CQRS/read models | Cross-shard queries hard; rebalancing; complexity |
| **8** | One module needs independent scale/deploy, *and* a team owns it | Extract that bounded context into a service | Network boundary, its own data, contract mgmt |
| **9** | Many teams blocked by one deploy; parts need very different scaling | Microservices + k8s/mesh/central obs | The full distributed-systems tax (below) |
| **10** | Global users / data-residency / DR across regions | Multi-region data + routing | Consistency across regions; cost; big complexity |

> **fbsy's place on the ladder:** it sits at **rung 0 by design** — a single stateless
> agent on a LAN box, durable state = config + last-sync result, retry only at the webhook
> boundary, "no message queue needed in v1." That is the *correct* engineering choice for
> its NFRs, and a reminder that **most software should not climb this ladder.**

---

## Performance engineering (do this before adding hardware)

Often the cheapest "scaling" is making the code/queries efficient. Method:

1. **Define the target** (the NFR: p99 < X ms at Y rps).
2. **Measure & profile** under realistic load — find the *actual* hot path (CPU, memory,
   I/O wait, lock contention). Use flame graphs / APM / traces.
3. **Fix the biggest bottleneck**, then re-measure. Repeat. (Amdahl's law: optimizing a 5%
   path can't help much.)
4. **Load-test** to find the breaking point and validate the fix.

Common high-leverage wins (in rough order of frequency):
- **The N+1 query** — fix with batching/joins/eager loading. The most common backend perf
  bug, period.
- **Missing indexes / bad queries** — read the query plan.
- **No connection pooling** — reuse DB/HTTP connections (fbsy reuses one `reqwest::Client`).
- **Caching** the expensive, frequently-read, rarely-changed thing.
- **Doing sync what could be async** — offload to a queue.
- **Chatty I/O** — batch network/disk calls.

Monitoring methods to know: **USE** (Utilization, Saturation, Errors — for resources) and
**RED** (Rate, Errors, Duration — for request-driven services).

---

## Statelessness & session handling

- **Stateless compute** is the enabler for rung 4+. Each request carries what it needs;
  shared state lives in a DB/cache.
- **Sessions:** store server-side in a shared store (Redis) or in a **signed token** (JWT)
  the client carries — *not* in instance memory. Avoid sticky sessions; they defeat the
  point.
- Anything that *must* be stateful (the database, a stateful socket) is isolated and scaled
  with its own (harder) strategy.

---

## Scaling the database (usually the real ceiling)

The hardest tier to scale, so it dominates architecture at the top of the ladder:

1. **Optimize** (indexes, queries, pooling) — cheapest.
2. **Scale up** (bigger DB box) — simple, finite.
3. **Read replicas** — offload reads; accept replication lag (an eventual-consistency
   decision — [PACELC](04-principles.md#cap--pacelc)).
4. **Caching** — take read load off the DB entirely.
5. **Partitioning / sharding** — split data across nodes by a key. Powerful but introduces
   cross-shard query pain, hot-shard risk, and rebalancing — a [one-way door](01-mindset.md),
   so choose the shard key with great care.
6. **CQRS / read models / specialized stores** — separate the write model from optimized
   read views; add a search/time-series/graph store for a specific access pattern.

> Write-scaling is the genuinely hard problem; reads are comparatively easy (replicas +
> cache). Design your data model and access patterns early — they're expensive to change.

---

## When to go distributed — the honest decision

Microservices solve **organizational** scaling and **independent deploy/scale/failure
isolation** — at the price of the **distributed-systems tax**: network failure, data
consistency without transactions, distributed tracing, deployment/versioning of many
units, and a platform to run it all.

**Default (2026): start with a [modular monolith](03-architecture.md), split later along
bounded contexts when a trigger fires.** Split a service out when **several** of these are
true:

- [ ] **Team scale:** more than ~30–50 engineers stepping on each other in one codebase.
- [ ] **Independent deploy cadence:** parts genuinely need to ship at different rhythms.
- [ ] **Independent scale:** one component's load profile is wildly different from the rest.
- [ ] **Failure isolation:** one part going down genuinely must not take the rest with it.
- [ ] **Clear, stable bounded context:** the seam is obvious and rarely crosses transactions.

If only one or two are true, you'll likely get a **distributed monolith** — all the cost of
microservices, none of the benefit. Most teams that adopted microservices in 2018–2022
shouldn't have; don't repeat it.

> Patterns you'll need *if* you go distributed: API gateway, service discovery, the
> **saga** pattern and the **transactional outbox** (consistency without distributed
> transactions), idempotent consumers, contract testing ([06](06-tooling.md)), and
> centralized observability ([07](07-operations.md)). And the laws bite:
> [CAP/PACELC](04-principles.md#cap--pacelc) and the fallacies of distributed computing.

---

## Capacity planning & cost

- **Headroom, not just "now."** Plan for peak (launches, seasonal spikes) with margin.
- **Autoscaling** (cloud) handles variable load — but set sane min/max and test it; a
  runaway scale-up is a runaway bill.
- **Cost is a quality attribute.** Track cost-per-request/tenant; the cheapest scaling move
  is often deleting waste (idle resources, chatty calls, over-provisioning), not buying
  more. **Efficiency is scalability.**

---

## Checklist (scaling)

- [ ] I have a measured bottleneck and a target NFR — not a guess.
- [ ] I exhausted vertical scaling and code/query optimization before going horizontal.
- [ ] The compute tier is stateless; state lives in backing services.
- [ ] Caching has a clear invalidation strategy; stale-read tradeoff is accepted consciously.
- [ ] DB scaling follows the order: optimize → scale up → replicas/cache → shard (last).
- [ ] Consistency model chosen deliberately ([CAP/PACELC](04-principles.md#cap--pacelc)).
- [ ] I'm staying on the lowest ladder rung that meets the NFRs.
- [ ] If splitting services: several split-criteria are true; seams = bounded contexts; not a distributed monolith.
- [ ] Capacity planned with headroom; cost-per-unit tracked.

---

## References

- M. Fowler, *MonolithFirst* — <https://martinfowler.com/bliki/MonolithFirst.html> · *How to break a monolith* — <https://martinfowler.com/articles/break-monolith-into-microservices.html>
- Modular monolith vs microservices (2025) — <https://www.ness.com/blog/modular-monolith-vs-microservices/>
- CAP/PACELC — [04 Principles](04-principles.md#cap--pacelc)
- Brendan Gregg, USE method — <https://www.brendangregg.com/usemethod.html> · RED method (Weave Works)
- Saga / Outbox patterns — <https://microservices.io/patterns/data/saga.html>
- *Designing Data-Intensive Applications*, M. Kleppmann (the definitive deep dive)
- AWS Well-Architected Framework — <https://aws.amazon.com/architecture/well-architected/>
