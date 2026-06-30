# 07 · Operations & Lifecycle Behaviors

> **Phase:** Operate ("Day 2")
> Software runs far longer than it's written. This chapter is how it *behaves in the wild*:
> configuration, observability, **auto-update**, resilience, and what you do when (not if)
> it breaks.

← [06 Tooling](06-tooling.md) · [Index](00-INDEX.md) · Next: [08 Security](08-security.md)

---

## Mental model

> **Design for the operator, not just the user.** The hardest, longest, most expensive
> phase is running the thing. "It works on my machine" is the start; "it stays working,
> unattended, and tells me when it can't" is the goal.

Operability is largely **architectural** ([01](01-mindset.md#quality-attributes)) — you
can't bolt it on. Build these in from the walking skeleton, not after the incident.

---

## Configuration management

> Config is everything that **varies between deployments** (URLs, credentials, ports,
> feature flags, tuning). Code is what stays the same. Keep them separate
> ([12-factor #3](04-principles.md#twelve-factor)).

- **Layer with precedence:** defaults → config file → environment variables → CLI flags
  (each overrides the previous). fbsy does this (`config.example.json` + env + flags).
- **Validate at startup, fail fast.** Parse config into a *typed* model and reject invalid
  config before doing any work — never discover a bad value mid-run. (fbsy: `BridgeConfig`
  validates before runtime starts.)
- **Secrets are not config files.** Keep them out of the repo and out of logs; inject via a
  secret manager or env. → [08](08-security.md#secrets).
- **Make config discoverable:** a `doctor`/`config check` command that prints the resolved,
  redacted config and validates connectivity is worth its weight in support tickets. (fbsy
  has `doctor`.)

---

## Observability — the three pillars

> **Monitoring tells you *whether* the system is healthy (known questions). Observability
> lets you ask *why* it's unhealthy — including questions you didn't anticipate — without
> shipping new code.** The goal isn't "having data"; it's reducing **MTTR**.

| Pillar | Answers | What it is | Tool of record |
|--------|---------|------------|----------------|
| **Logs** | *Why* did this specific thing happen? | Immutable, timestamped records of discrete events | Structured logging |
| **Metrics** | *Is* something wrong? (trends, alerts) | Aggregatable numbers over time (counters, gauges, histograms) | Prometheus / OTLP |
| **Traces** | *Where* in the flow did it happen? | A request's path across functions/services with timing | OpenTelemetry / Jaeger |

> Investigation flow: **metrics** reveal something's wrong → **traces** show where →
> **logs** explain why.

### Make logs useful
- **Structured (key-value/JSON), not prose.** "level=error event=sync_failed device=A
  err=timeout" beats "Something went wrong with the sync." (fbsy uses `tracing` for this.)
- **Log to stdout/stderr as an event stream** ([12-factor #11](04-principles.md#twelve-factor));
  let the platform/agent route and store. Don't manage log files inside the app if you can
  avoid it.
- **Correlation IDs.** Tag every log line of one operation with a shared id so you can
  reconstruct it. Essential once anything is async or distributed.
- **Levels with intent:** ERROR = needs human attention; WARN = degraded but handled; INFO
  = lifecycle milestones; DEBUG = developer detail.
- **Redact secrets/PII at the logging boundary** — never rely on remembering per call-site.
  (fbsy has `src/support/redaction.rs`.) → [08](08-security.md).

### OpenTelemetry (OTel) is the standard
A vendor-neutral standard for all three signals: an **API/SDK** (instrumentation +
context propagation), the **OTLP** wire protocol, **semantic conventions**, and the
**Collector** (receive → process → export). Instrument once, send anywhere — avoids
vendor lock-in. Start with **auto-instrumentation**, then add custom spans/metrics where
they pay off.

### SLI / SLO / SLA & error budgets
- **SLI** — a measured indicator (e.g. % of syncs succeeding, p99 latency).
- **SLO** — your internal target for an SLI (e.g. 99.9% success monthly).
- **SLA** — a contractual promise to a customer (looser than the SLO).
- **Error budget** = `1 − SLO`. It's permission to take risk: budget left → ship faster;
  budget burned → freeze features and fix reliability. This turns "how reliable?" from an
  argument into a number.

---

## Lifecycle behaviors (process discipline)

- **Health endpoints:** *liveness* ("am I alive?" → restart if not) vs *readiness* ("can I
  take traffic?" → route around if not). Distinct, both needed for orchestration.
- **Graceful startup:** validate config + dependencies, *then* announce ready. Fail fast if
  a hard dependency is missing.
- **Graceful shutdown** ([12-factor #9](04-principles.md#twelve-factor)): catch SIGTERM,
  stop accepting new work, finish or checkpoint in-flight work, release resources, exit.
  (fbsy's `runtime` handles process lifecycle/shutdown.) Robust to *sudden* death too —
  assume the process can be killed at any instant, so persisted state must always be
  consistent.
- **Idempotent, crash-safe operations:** design so that a kill mid-operation leaves no
  corruption and a retry is safe. (fbsy's invariant — *never clear the device until the
  webhook succeeds* — is exactly this.)

---

## Auto-update

*Delivering new versions safely.* For software that runs on machines you don't control (agents, desktop/mobile apps, CLIs,
IoT, on-prem like **fbsy**), the update channel is **both a critical feature and a prime
attack target**. A compromised or buggy update mechanism can brick or own every install at
once — treat it with the rigor of a security boundary.

### The lifecycle of a safe update
```
 check (poll/notify) → resolve version (channel + rollout %) →
 download → VERIFY (signature + hash) → stage → swap atomically →
 (restart) → health-check → keep or ROLLBACK → report telemetry
```

### Non-negotiables (the threats and their defenses)
| Requirement | Why | How |
|-------------|-----|-----|
| **Integrity & authenticity** | Stop tampered / spoofed updates (CIA-Integrity) | **Sign** releases; client verifies signature **and** a content hash (fbsy ships `sha2`) before executing. Never trust HTTPS alone. |
| **Transport security** | Stop MITM | HTTPS/TLS for metadata + artifact (fbsy: `rustls`). |
| **Rollback-attack protection** | Stop attacker pinning you to an old, vulnerable version | Track current version; **refuse to "update" to an older one**; signed, *expiring*, versioned metadata. |
| **Atomicity** | A half-written binary must never run | Download to temp → verify → atomic swap/replace (fbsy: `self-replace`). |
| **Recoverability** | A bad update must not brick the install | Keep previous version; auto-rollback on failed post-update health check. |
| **Staged rollout** | Limit blast radius of a bad release | Release to a small % / ring first; widen as telemetry stays green (the canary idea from [06](06-tooling.md), applied to clients). |
| **Consent & control** | Respect the operator | Channels (stable/beta), the ability to pin/defer, and clear changelogs. |

### The reference design: TUF
**The Update Framework (TUF)** is the CNCF-graduated standard that formalizes the above. Its
core ideas, worth stealing even if you don't adopt it wholesale:
- **Separation of roles with separate keys** (`root`, `targets`, `snapshot`, `timestamp`)
  and **threshold signatures**, so *one* compromised key isn't game over.
- **Designed assuming the repo/keys *will* be compromised** — minimize the damage when it
  happens.
- **Built-in protection** against rollback, mix-and-match, and freeze attacks via signed,
  versioned, *expiring* metadata.

> Senior takeaway: for any auto-updater, at minimum **sign + verify + hash-check + atomic
> swap + version-monotonic + auto-rollback + staged rollout.** fbsy already has the
> primitives (`self-replace`, `sha2`, `rustls`, an `update` use case); the maturity ladder
> is to add signing/verification, monotonic version checks, and ring-based rollout.

---

## Resilience patterns (surviving failure)

The network is unreliable and dependencies *will* fail ([fallacies of distributed
computing](04-principles.md#distributed-systems-laws-when-you-have-a-network)). Every
call across a boundary needs a failure plan.

| Pattern | Problem it solves | Note |
|---------|-------------------|------|
| **Timeouts** | A hung dependency hanging *you* | Every remote call gets one. The #1 omission. |
| **Retries + exponential backoff + jitter** | Transient blips | Only for **idempotent** ops; jitter prevents thundering herds; cap attempts. |
| **Circuit breaker** | Hammering a down dependency | Trip open after N failures, fail fast, periodically probe to recover. |
| **Bulkhead** | One slow dependency starving all resources | Isolate resource pools so one failure can't sink the ship. |
| **Rate limiting / backpressure** | Overload (yours or theirs) | Shed or queue load past capacity instead of collapsing. |
| **Graceful degradation** | Partial outage | Serve a reduced experience (cache, default) rather than total failure. |
| **Idempotency keys** | Duplicate delivery from retries | Make repeats safe ([04](04-principles.md), [09](09-scaling.md)). |

> fbsy applies the minimal-but-right subset: retry/backoff *only* at the HRMS webhook
> boundary, and the idempotent "don't clear until success" invariant. You don't need all
> patterns everywhere — apply them at the boundaries that can actually fail.

---

## Backups & disaster recovery

- **Define RPO & RTO** as NFRs: **RPO** (Recovery Point Objective) = max acceptable data
  loss (how far back); **RTO** (Recovery Time Objective) = max acceptable downtime. They
  drive backup frequency and DR design.
- **3-2-1 backups:** 3 copies, 2 media, 1 offsite/immutable.
- **A backup you haven't restored is a hope, not a backup.** Rehearse restores on a
  schedule.
- **Protect against ransomware/operator error** with immutable/versioned backups.

---

## Incident response (when it breaks)

- **Runbooks:** for each alert, a written "if this fires, do that." Reduces MTTR and
  removes hero-dependence.
- **On-call & escalation:** clear ownership, sane rotation, alert on *symptoms users feel*
  (SLO burn), not every metric (alert fatigue kills response).
- **Blameless postmortems:** after any significant incident, write up timeline → root cause
  → contributing factors → action items. Blame the *system/process*, not the person — that's
  how you actually prevent recurrence. Track action items to done.

---

## Checklist (operations)

- [ ] Config is layered, typed, validated at startup, with secrets injected (not committed).
- [ ] Structured logs to stdout with correlation IDs; secrets/PII redacted at the boundary.
- [ ] Metrics + traces (OTel) for the paths that matter; SLOs + error budget defined.
- [ ] Health checks (liveness/readiness) and graceful start/shutdown implemented.
- [ ] Operations are idempotent and crash-safe; critical invariants survive a kill.
- [ ] Auto-update (if applicable): signed + hash-verified + atomic + monotonic + rollback + staged.
- [ ] Boundary calls have timeouts; retries use backoff+jitter on idempotent ops only.
- [ ] Backups defined by RPO/RTO and **restore-tested**.
- [ ] Runbooks exist; alerts are symptom-based; postmortems are blameless and tracked.

---

## References

- OpenTelemetry — <https://opentelemetry.io/docs/concepts/observability-primer/>
- Google SRE Book (SLI/SLO/error budgets) — <https://sre.google/books/>
- The Update Framework (TUF) — <https://theupdateframework.io/> · spec — <https://theupdateframework.github.io/specification/latest/>
- Software update mechanism security — <https://safeguard.sh/resources/blog/software-update-mechanism-security>
- Release It! (M. Nygard) — circuit breaker, bulkhead, timeouts
- AWS Builders' Library: timeouts, retries, backoff & jitter — <https://aws.amazon.com/builders-library/timeouts-retries-and-backoff-with-jitter/>
