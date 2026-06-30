# 05 · Technology & Package Selection

> **Phase:** Architect → Build
> Choosing languages, frameworks, and dependencies — and *defending* the choice. A
> dependency is a long-term relationship, not a download.

← [04 Principles](04-principles.md) · [Index](00-INDEX.md) · Next: [06 Tooling](06-tooling.md)

---

## Mental model

> **Every dependency is a liability you accept in exchange for leverage.** Pick the ones
> whose leverage clearly outweighs their lifetime cost (learning, ops, security,
> upgrades, the eventual migration off them).

Three guiding heuristics:
1. **Fitness for purpose** beats popularity. The "best" tool is the one that fits *your*
   NFRs, team, and constraints.
2. **Boring technology wins by default.** Proven, well-documented, widely-deployed tech
   has fewer unknown failure modes. Spend [innovation tokens](01-mindset.md#core-mental-models)
   only on your differentiator.
3. **Optimize for TCO, not day-one velocity.** A framework that's fast to start but slow
   to operate/upgrade/hire-for is a bad trade over years.

---

## Choosing a language

Match the language's strengths to your *dominant* constraint. There is no universal best;
there is a best-fit per workload.

| Constraint / domain | Strong fits | Why |
|---------------------|-------------|-----|
| Native perf, memory safety, single static binary, low footprint | **Rust**, Go, C++/Zig | No GC pauses; Rust adds compile-time safety w/o GC |
| Fast service development, simple concurrency, ops simplicity | **Go** | Tiny binaries, great stdlib, easy deploy, fast builds |
| Huge ecosystem, full-stack JS, rapid iteration | **TypeScript/Node** | One language client+server, npm reach |
| Enterprise, long-lived, strong typing & tooling | **Java/Kotlin**, C# | Mature JVM/.NET, libraries, talent pool |
| Data/ML, scripting, glue, rapid prototyping | **Python** | Ecosystem (pandas, PyTorch), readability |
| Mobile | **Kotlin** (Android), **Swift** (iOS), **Dart/Flutter** (cross) | Platform-native vs one-codebase tradeoff |
| Web UI | **TypeScript** (+ a framework) | Ecosystem, types catch UI bugs |

Decision factors, weighted: **team familiarity** (usually dominant — Conway/velocity),
performance & resource NFRs, ecosystem maturity for your domain, hiring pool, tooling
quality, deployment target, and *longevity* (will it be maintained in 5 years?).

> `fbsy`'s choice of **Rust** is justified: it must be a single, dependency-free native
> binary that runs unattended on a customer's LAN machine, never leaks memory over weeks of
> uptime, and is memory-safe when parsing an untrusted device protocol. That maps directly
> to Rust's strengths — this is a legitimate innovation-token spend.

---

## Choosing a framework

- **Prefer the boring, well-supported default** for your language (e.g. a mature web
  framework with a big community) unless an NFR rules it out.
- **Libraries over frameworks where you can.** A *library* you call keeps you in control;
  a *framework* calls you and dictates structure (inversion of control). Frameworks give
  speed but impose lock-in and a ceiling — adopt them at the **edges** (adapters), and
  keep your [domain framework-free](03-architecture.md#the-dependency-rule-the-one-rule-to-remember).
- **Evaluate the exit.** How hard is it to leave? The harder, the more it's a
  [one-way door](01-mindset.md#core-mental-models) deserving an ADR.

---

## Evaluating a dependency (the checklist that prevents pain)

Treat adding a dependency like hiring: scrutinize before, not after.

**Health & maintenance**
- [ ] Active maintenance (recent commits/releases; issues triaged)?
- [ ] Bus factor — one maintainer or a team/foundation behind it?
- [ ] Maturity & stability (post-1.0, stable API, semver discipline)?
- [ ] Adoption (downloads/dependents) and a real community?

**Cost of ownership**
- [ ] Transitive dependency count (each is also your liability)?
- [ ] Binary-size / bundle-size / cold-start impact?
- [ ] Build-time impact?
- [ ] How painful are major upgrades historically?

**Risk**
- [ ] **License** compatible with your use (MIT/Apache-2.0 permissive vs GPL/AGPL copyleft;
  check for your distribution model)?
- [ ] Known CVEs / security track record? Does it handle untrusted input?
- [ ] Could you realistically vendor or replace it if abandoned?

> **The "is it worth a dependency?" test:** for trivial functionality, a few lines you own
> beat a micro-dependency you don't (the classic `left-pad` lesson). For hard, security-
> sensitive, or heavily-tested functionality (crypto, TLS, parsers, date math), **always**
> use the vetted library — never roll your own.

---

## Dependency hygiene & supply chain (ongoing)

Selecting is step one; *governing* dependencies is forever. (Deep dive: [08 Security](08-security.md#supply-chain).)

- **Pin & lock.** Commit the lockfile (`Cargo.lock`, `package-lock.json`, `go.sum`, …) for
  reproducible builds (12-factor #2).
- **Pin CI actions/images by digest**, not floating tags (the 2025 GitHub Actions
  supply-chain attacks targeted exactly this).
- **Automate updates** (Dependabot/Renovate) so upgrades are small and continuous, not a
  scary annual cliff.
- **Scan continuously** (`cargo audit`, `npm audit`, `govulncheck`, Trivy/Grype, etc.) in
  CI and fail on known-exploitable CVEs.
- **Generate an SBOM** (Software Bill of Materials) so you can answer "are we affected by
  CVE-X?" in minutes, not days.
- **Minimize.** The cheapest vulnerability to manage is the dependency you didn't add.

---

## Build vs buy vs borrow

| Option | Choose when | Watch out for |
|--------|------------|---------------|
| **Build** (write it yourself) | It's your **core domain** / differentiator; no good option exists; control is critical | Don't build undifferentiated heavy lifting (auth, payments, crypto, TLS) |
| **Buy** (SaaS / managed service) | Undifferentiated but essential (email, payments, observability, DB hosting) | Cost at scale, lock-in, data residency, the [one-way door](01-mindset.md) |
| **Borrow** (open source) | A vetted library covers it | License, maintenance, supply-chain risk (above) |

> Rule of thumb: **build your core, buy/borrow your context.** Map this onto DDD's
> core-vs-generic subdomains ([03](03-architecture.md#domain-driven-design-when-the-domain-is-the-hard-part)).

---

## A reusable decision matrix

When choosing between real options, score them against weighted criteria instead of
arguing. Make the weights reflect your NFRs.

| Criterion (weight) | Option A | Option B | Option C |
|--------------------|:-------:|:-------:|:-------:|
| Fit to NFRs (×3) | | | |
| Team familiarity (×3) | | | |
| Ecosystem / maturity (×2) | | | |
| Operability / TCO (×2) | | | |
| Security posture (×2) | | | |
| Exit cost / lock-in (×1) | | | |
| **Weighted total** | | | |

Record the result and the *why* as an [ADR](02-planning.md#architecture-decision-records-adrs).
The matrix isn't the decision — it's how you make the tradeoffs visible and defensible.

---

## Worked example: the fbsy stack

A study in *deliberately small* dependency selection (`Cargo.toml`):

| Need | Crate | Why this one (the tradeoff) |
|------|-------|------------------------------|
| Errors (app boundary) | `anyhow` | Context-rich errors for the binary's edges |
| Errors (typed/domain) | `thiserror` | Precise, matchable error types in the core |
| CLI parsing + UX | `clap` (derive) | The de-facto standard; declarative, good help/errors |
| HTTP client | `reqwest` + **`rustls`** | TLS **without** OpenSSL → static binary, fewer CVEs, no system deps |
| Serialization | `serde` / `serde_json` | The Rust standard; config + webhook payloads |
| Logging | `tracing` (+ subscriber) | Structured, async-aware; foundation for observability ([07](07-operations.md)) |
| Dates | `chrono` (no default features) | Trimmed features → smaller, fewer surprises |
| Self-update | `self-replace` | Replace the running binary safely cross-platform ([07](07-operations.md#auto-update)) |
| Hashing | `sha2` | Verify update integrity (CIA-integrity, [08](08-security.md)) |
| Paths / dirs | `directories` | Correct per-OS config/data locations |

Notice the discipline: **`rustls` over OpenSSL** (smaller attack surface, static linking),
trimmed feature flags, and *no* async runtime yet (`tokio`/`axum` are commented out in
`Cargo.toml` — added only when the HTTP server actually needs them = YAGNI in action). Each
crate maps to a real need; none is speculative.

---

## Checklist (tech selection)

- [ ] Language fits the dominant NFR *and* the team can wield it.
- [ ] Innovation tokens spent only on the differentiator; everything else is boring/proven.
- [ ] Each dependency passed the health/cost/risk evaluation; licenses are compatible.
- [ ] No hand-rolled crypto/TLS/auth; vetted libraries used for hard, sensitive code.
- [ ] Lockfile committed; CI actions/images pinned by digest.
- [ ] Update automation + vulnerability scanning + SBOM in place.
- [ ] Core built, context bought/borrowed.
- [ ] The choice is recorded as an ADR with the decision matrix.

---

## References

- Dan McKinley, *Choose Boring Technology* — <https://boringtechnology.club/>
- Rust best practices (2025) — <https://andrewodendaal.com/rust-maintainable-code-practices/>
- OpenSSF Scorecard (project health signals) — <https://securityscorecards.dev/>
- SLSA supply-chain framework — <https://slsa.dev/> · SBOM (CycloneDX/SPDX)
- SPDX license list — <https://spdx.org/licenses/>
