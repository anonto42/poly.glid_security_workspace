# 06 · Dev Tooling & CI/CD

> **Phase:** Build → Ship
> The machinery that gives you **fast, safe, repeatable** feedback and releases. Tooling is
> not overhead — it's the system that lets you move fast *without breaking things*.

← [05 Tech Selection](05-tech-selection.md) · [Index](00-INDEX.md) · Next: [07 Operations](07-operations.md)

---

## Mental model

> **Automate everything you'd otherwise do twice, and make the feedback loop as short as
> possible.** Every manual, remembered step is a future outage. Every minute of feedback
> latency is paid back many times a day.

Two ideas drive all tooling decisions:
1. **Shift left** — catch problems at the earliest, cheapest stage (editor < commit hook <
   CI < staging < production). A bug caught by the type checker costs seconds; the same bug
   in production costs an incident.
2. **The build/release/run separation** ([12-factor #5](04-principles.md#twelve-factor)) —
   an immutable artifact is built once, then promoted unchanged through environments.

---

## Layer 1 — Reproducible local environment

If "works on my machine" is possible, you've already lost. Make the environment a
declared, version-controlled artifact.

- **Pin toolchain versions** (`rust-toolchain.toml`, `.nvmrc`, `.python-version`, `go.mod`
  `go` directive). Everyone and CI use the identical version.
- **Lock dependencies** (`Cargo.lock`, `package-lock.json`, `go.sum`) — committed.
- **One-command setup** (`make setup` / a bootstrap script) and one-command run.
- **Containerize the dev env** (devcontainer/Docker/Nix) when the toolchain is complex or
  the team is large — eliminates drift entirely.
- **Keep dev/prod parity** ([12-factor #10](04-principles.md#twelve-factor)).

---

## Layer 2 — Quality gates (the editor → commit loop)

Run these locally *and* in CI. Local = fast feedback; CI = enforcement (never trust that
everyone ran them).

| Gate | Purpose | Rust example | Cross-stack |
|------|---------|--------------|-------------|
| **Formatter** | End style debates; uniform diffs | `cargo fmt` | prettier, gofmt, black, ktlint |
| **Linter** | Catch bugs & anti-patterns statically | `cargo clippy -D warnings` | eslint, golangci-lint, ruff |
| **Type / compile check** | Eliminate whole bug classes pre-runtime | `cargo check` (+ borrow checker) | tsc, mypy, javac |
| **Tests** | Verify behavior | `cargo test` | jest/vitest, go test, pytest, junit |
| **Dependency audit** | Known CVEs | `cargo audit` | npm audit, govulncheck, trivy |
| **Secret scan** | No keys committed | gitleaks | gitleaks, trufflehog |

> **Pre-commit hooks** run the fast gates (format, lint, quick tests) before code can be
> committed — this repo wires them via `scripts/install-hooks.sh` + `.githooks/pre-commit`.
> Keep hooks **fast** (seconds); push slow checks to CI. Make the gates **blocking** (e.g.
> `clippy -D warnings`) or they erode.

---

## Layer 3 — Version control discipline

- **Trunk-based development** (short-lived branches merged to `main` daily) is what elite
  DORA performers do; it minimizes merge pain and keeps `main` always-releasable. Long-lived
  feature branches are an anti-pattern at scale.
- **Conventional Commits** (`feat:`, `fix:`, `chore:`…) — machine-readable history that
  drives automated changelogs and semver bumps. (This repo already uses them.)
- **Small PRs.** Reviewable in <30 min; smaller = faster review, fewer bugs, easier
  rollback.
- **`main` is sacred:** protected branch, required green CI, required review. Never commit
  straight to `main` on a team.
- **Feature flags** decouple *deploy* from *release* — merge incomplete work safely behind
  a flag instead of holding a long branch.

---

## Layer 4 — Testing strategy

> Tests exist to let you **change code without fear**. Optimize for *confidence per second
> of runtime*, not coverage percentage.

### The test pyramid
```
        ╱ E2E ╲          few   — slow, brittle, high realism (full user flows)
      ╱─────────╲
    ╱ Integration ╲      some  — real adapters/DB; verify the seams
  ╱─────────────────╲
 ╱   Unit / domain    ╲  many  — fast, pure logic, no I/O
```
- **Unit** — pure functions, domain rules, mappers. Milliseconds. The bulk of your tests
  live here *because the [domain has no I/O](03-architecture.md)*.
- **Integration** — exercise adapters against real (or test-container) dependencies; verify
  the port↔adapter contract. (This repo's `tests/` — `sync_tests`, `device_protocol_tests`,
  `config_tests` — using fake adapters, is exactly this.)
- **E2E** — a few critical user journeys end-to-end. Expensive and flaky; keep them few.

### Beyond the pyramid
- **Property-based testing** (proptest/quickcheck/fast-check/Hypothesis) — assert
  invariants over generated inputs; finds edge cases you'd never write by hand. Ideal for
  parsers, mappers, serialization.
- **Contract tests** — verify a provider and consumer agree on an API shape (vital across
  services — [09](09-scaling.md)).
- **Snapshot/golden tests** — for serialized output / templates (this repo's
  `template_tests`).
- **Test the error paths**, not just the happy path — that's where production breaks.

### Coverage philosophy
Coverage is a *smell detector, not a target*. 100% coverage of trivial getters is worse
than 70% coverage of the logic that matters. Cover the **risk**, the **invariants**, and
every **bug you fix** (add a regression test).

> **The critical invariant deserves a dedicated test.** e.g. fbsy's *"never clear the
> device until the webhook upload succeeded"* — that's a data-loss bug if violated, so it
> gets an explicit, named test.

---

## Layer 5 — Continuous Integration (CI)

Every push runs the full gate set on a clean machine. Typical pipeline:

```
checkout → restore cache → fmt check → lint → type/compile → test (matrix) →
audit + secret scan → build artifact → (sign + SBOM) → upload
```

- **Fast & cached.** Cache deps/build; parallelize. Aim for <10 min; slow CI gets bypassed.
- **Matrix** across OS/arch/versions you must support (fbsy ships Windows + Unix → build
  both).
- **Fail loudly, fix immediately.** A red `main` blocks everyone — fixing it is priority #1.
- **Deterministic.** No network flakiness, no time/order dependence. Flaky tests destroy
  trust faster than no tests.

---

## Layer 6 — Continuous Delivery/Deployment (CD)

The immutable artifact from CI is **promoted** through environments. Choose a rollout
strategy by your risk tolerance:

| Strategy | How | Use when | Cost |
|----------|-----|----------|------|
| **Recreate** | Stop old, start new | Dev, tolerable downtime | Downtime |
| **Rolling** | Replace instances gradually | Default for stateless services | Two versions coexist briefly |
| **Blue-Green** | Two full envs; flip traffic | Need instant rollback, zero downtime | 2× infra during switch |
| **Canary** | Route small % to new, watch metrics, ramp | High-risk changes, big blast radius | Needs good observability + automation |
| **Feature flags** | Ship dark, enable per-cohort | Decouple release from deploy; A/B | Flag lifecycle/debt management |

> **Rollback must be a first-class, rehearsed button** — not a redeploy scramble. The
> ability to revert in minutes is what makes fast deploys *safe*. (For client/agent
> software like fbsy, "rollback" means staged update channels + the ability to pin/revert a
> version — [07 auto-update](07-operations.md#auto-update).)

---

## Layer 7 — Release management

- **Semantic Versioning** (`MAJOR.MINOR.PATCH`): breaking / feature / fix. Communicates
  upgrade risk in the number itself. (fbsy is at `0.2.x`.)
- **Automated changelogs** from conventional commits.
- **Immutable, versioned artifacts** — never rebuild "the same" release; promote the exact
  bytes that passed CI.
- **Sign artifacts & publish provenance** — code signing + SLSA provenance so consumers can
  verify integrity ([08 supply chain](08-security.md#supply-chain)). Essential for anything
  users download and run (like fbsy).
- **Tag releases** in VCS; the tag, the artifact, and the changelog all line up.

---

## Measuring delivery: DORA metrics

The four (now five) research-backed signals of delivery performance. Track them to know if
your tooling is actually working.

| Metric | What | Elite (~top 15%, 2025) |
|--------|------|------------------------|
| **Deployment Frequency** | How often you ship to prod | On-demand / multiple per day |
| **Lead Time for Changes** | Commit → running in prod | < 1 day |
| **Change Failure Rate** | % deploys causing a failure | < 15% |
| **Failed-Deployment Recovery Time** (MTTR) | Time to restore after a failed deploy | < 1 hour |
| **Rework Rate** (2025 addition) | % deploys that are unplanned fixes | low |

> The two pairs are a balance: **throughput** (frequency, lead time) vs **stability**
> (failure rate, recovery). Elite teams are *both*, because automation (tests, CI/CD, fast
> rollback) raises both at once. **Automated testing is the single highest-leverage
> investment** — it improves multiple DORA metrics simultaneously.

---

## Checklist (tooling)

- [ ] Toolchain pinned, deps locked, one-command setup; dev/prod parity.
- [ ] Format + lint + type-check + test + audit + secret-scan run locally **and** in CI.
- [ ] Pre-commit hook runs the fast gates; CI enforces all of them, blocking.
- [ ] Trunk-based, conventional commits, small PRs, protected `main`.
- [ ] Tests follow the pyramid; the critical invariants have named tests; error paths covered.
- [ ] CI is fast (<~10 min), cached, deterministic, matrixed across targets.
- [ ] Releases: immutable versioned artifacts, semver, automated changelog, **signed**.
- [ ] Rollback is a rehearsed, fast operation.
- [ ] DORA metrics tracked and reviewed (~quarterly).

---

## References

- DORA metrics — <https://dora.dev/guides/dora-metrics/> · *Accelerate* (Forsgren, Humble, Kim)
- Trunk-Based Development — <https://trunkbaseddevelopment.com/>
- Conventional Commits — <https://www.conventionalcommits.org/>
- Semantic Versioning — <https://semver.org/>
- Test pyramid (M. Fowler) — <https://martinfowler.com/articles/practical-test-pyramid.html>
- Deployment strategies — <https://martinfowler.com/bliki/BlueGreenDeployment.html>, <https://martinfowler.com/bliki/CanaryRelease.html>
- Feature flags — <https://martinfowler.com/articles/feature-toggles.html>
