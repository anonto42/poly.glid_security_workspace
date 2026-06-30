# 08 · Security

> **Phase:** all (cross-cutting, "shift left")
> Security is not a feature you add — it's a property you preserve at every layer. Built on
> the [CIA triad](04-principles.md#cia): keep data **C**onfidential, **I**ntact, and
> **A**vailable.

← [07 Operations](07-operations.md) · [Index](00-INDEX.md) · Next: [09 Scaling](09-scaling.md)

---

## Mental model

> **Assume breach. Minimize what an attacker gains when (not if) one control fails.**
> Security is risk management, not absolute prevention.

The principles that operationalize [CIA](04-principles.md#cia):

- **Defense in depth** — layered controls; no single point of failure. One bypassed
  control shouldn't end the game.
- **Least privilege** — every component/user/process gets the *minimum* access needed,
  for the *minimum* time.
- **Secure by default** — the default configuration is the safe one; insecurity is
  opt-in and loud. (Make the secure path the *easy* path — that dissolves the
  security-vs-usability tradeoff.)
- **Fail safe / fail closed** — on error, deny rather than allow.
- **Complete mediation** — check authorization on *every* access, not just the first.
- **Zero trust** — never trust based on network location; authenticate and authorize every
  request. "Inside the firewall" is not a permission.
- **Shift left** — find security issues in design/code review/CI, not in a pen-test the
  week before launch. Cheapest to fix earliest.
- **Minimize attack surface** — every port, endpoint, dependency, and feature is a way in.
  The most secure code is the code that isn't there.

---

## Threat modeling (do this in *Architect*)

Before building, ask "how would I attack this?" Use your [C4 diagram](03-architecture.md)
and look at the **trust boundaries** (where data crosses from less-trusted to
more-trusted). The lightweight standard is **STRIDE**:

| Threat | Violates | Defense |
|--------|----------|---------|
| **S**poofing | Authenticity | Strong authentication |
| **T**ampering | Integrity | Signing, hashing, validation, immutability |
| **R**epudiation | Non-repudiation | Audit logs, signed actions |
| **I**nformation disclosure | Confidentiality | Encryption, access control, redaction |
| **D**enial of service | Availability | Rate limiting, quotas, autoscaling, timeouts |
| **E**levation of privilege | Authorization | Least privilege, complete mediation |

> Output: a short list of the top threats per boundary and the control for each. Revisit
> when the architecture changes.

---

## OWASP Top 10 (2025) — the risks to design against

The industry-consensus list of the most critical web application risks. The **2025**
edition shifts emphasis from isolated coding bugs toward **the security of the whole
software ecosystem** — most notably elevating **software supply-chain failures** (malicious
packages, compromised maintainers, tampered builds) and **insecure configuration**.

Recurring categories to design against (each maps to a control below):
- **Broken access control** — enforce authZ server-side, deny by default.
- **Cryptographic failures** — encrypt sensitive data in transit & at rest; use vetted
  libraries; never roll your own crypto.
- **Injection** (SQL, command, XSS…) — never mix untrusted input with code/queries;
  parameterize; encode output.
- **Insecure design** — threat-model; this is why security is architectural.
- **Security misconfiguration** — secure defaults, hardening, no debug endpoints in prod.
- **Vulnerable & outdated components** + **software supply-chain failures** — see
  [supply chain](#supply-chain).
- **Identification & authentication failures** — robust authN, session/token handling.
- **Software & data integrity failures** — verify integrity of updates, deserialized data,
  CI/CD ([auto-update](07-operations.md#auto-update)).
- **Logging & monitoring failures** — you can't respond to what you can't see
  ([07](07-operations.md)).
- **SSRF** — validate/segment outbound requests.

> Use the **OWASP Proactive Controls** and **ASVS** as the *constructive* counterpart:
> what to *do*, not just what to fear.

---

## Authentication vs Authorization

- **AuthN** = *who are you?* — verify identity (passwords + MFA, OIDC/OAuth2, mTLS, API
  keys, signed tokens).
- **AuthZ** = *what may you do?* — enforce permissions on every request (RBAC/ABAC),
  server-side, deny-by-default. **Never enforce authZ in the client.**
- Use **standards, not homegrown auth**: OAuth2/OIDC for delegated auth, well-tested
  session/JWT libraries. Short-lived tokens + refresh; rotate and revoke.

---

## Input validation & output encoding

- **All input is hostile** until validated — at the trust boundary, *fail fast*
  ([04](04-principles.md)). Validate type, range, length, format with an **allowlist**
  (define what's valid; reject the rest), not a denylist.
- **Parameterize** everything that touches an interpreter (SQL, shell, LDAP) — never
  string-concatenate untrusted data into a query/command.
- **Encode on output** for the destination context (HTML, URL, SQL, shell) to neutralize
  injection.
- This is also why a **memory-safe language matters** when parsing untrusted bytes — e.g.
  fbsy parsing a device's wire protocol in Rust avoids the buffer-overflow class entirely.

---

## Secrets

> A secret in source control is a secret that has leaked. Treat git history as public.

- **Never** commit secrets; keep them out of code and config files in the repo.
- **Inject at runtime** via environment ([12-factor #3](04-principles.md#twelve-factor)) or
  a **secret manager** (Vault, cloud KMS/Secrets Manager, SOPS-encrypted files).
- **Rotate** regularly and on any suspected exposure; prefer **short-lived** credentials.
- **Scan** for committed secrets in pre-commit and CI (gitleaks/trufflehog) — [06](06-tooling.md).
- **Redact from logs/errors** at the boundary — never depend on remembering per call site.
  (fbsy: `src/support/redaction.rs`.)

---

## Data protection

- **Encryption in transit:** TLS everywhere, modern config. (fbsy uses `rustls` — a
  memory-safe TLS stack, smaller attack surface than OpenSSL.)
- **Encryption at rest:** for sensitive data and backups; manage keys in a KMS.
- **Data minimization:** don't collect/store what you don't need (you can't leak data you
  never held). Classify data (public / internal / PII / secret) and apply controls by class.
- **PII & compliance:** know your obligations (GDPR, etc.) — consent, retention limits,
  right-to-deletion, residency. Design for them; they're NFRs ([02](02-planning.md)).

---

## Supply chain

*Supply-chain security.* The fastest-growing attack class (and newly emphasized in OWASP 2025). Your software is
only as trustworthy as everything it's built from and ships with.

| Practice | What | Tool |
|----------|------|------|
| **Pin & lock** | Reproducible builds; no surprise transitive bumps | committed lockfiles; CI actions/images pinned **by digest** |
| **Scan dependencies** | Catch known CVEs continuously, fail CI on exploitable ones | `cargo audit`, `npm audit`, `govulncheck`, Trivy/Grype, Dependabot |
| **SBOM** | A bill of materials so you can answer "are we affected by CVE-X?" fast | CycloneDX / SPDX |
| **Sign artifacts** | Consumers verify authenticity & integrity | **Sigstore/cosign** (keyless via OIDC), GPG, platform signing |
| **Build provenance** | Verifiable record of *how/where* it was built | **SLSA** levels; GitHub artifact attestations |
| **Harden CI/CD** | The build system is now a top target (2025 GitHub Actions attacks) | least-privilege tokens, pin actions, isolate runners, review third-party actions |

> **SLSA** (Supply-chain Levels for Software Artifacts) gives you a maturity ladder;
> **Sigstore** (cosign/Fulcio/Rekor) makes keyless signing + a transparency log practical —
> "SLSA Level 2 in an afternoon" with current GitHub tooling. For anything users download
> and execute (like fbsy), **signing + provenance is the integrity half of your
> [auto-update](07-operations.md#auto-update) story.**

---

## Runtime & deployment hardening

- **Run with least privilege:** non-root/dedicated service account; drop capabilities;
  read-only filesystem where possible.
- **Minimize exposure:** bind to `127.0.0.1` if it doesn't need to be public (fbsy binds
  its local API to loopback by default — a textbook attack-surface reduction); firewall
  the rest; close unused ports.
- **Minimal base images** (distroless/scratch) — fewer packages, fewer CVEs.
- **Keep the host patched**; automate OS updates.
- **Resource limits** (CPU/mem/conn) to contain DoS and runaway processes.

---

## Secure SDLC — weave it through every phase

Security is the cross-cutting thread of the whole lifecycle, not a gate at the end:

| Phase | Security activity |
|-------|-------------------|
| Plan | Security/privacy NFRs; compliance requirements |
| Architect | **Threat modeling**; trust boundaries; secure design review |
| Build | Input validation, secrets hygiene, secure coding, **code review** |
| Ship | SAST/DAST, dependency scan, secret scan, artifact signing in CI |
| Operate | Patching, monitoring/alerting, incident response, key rotation |
| Scale | Re-threat-model new boundaries; segment networks |

---

## Checklist (security)

- [ ] Threat-modeled the trust boundaries (STRIDE); top threats have named controls.
- [ ] AuthN via standards; authZ enforced server-side, deny-by-default, on every access.
- [ ] All external input validated (allowlist) at the boundary; queries parameterized; output encoded.
- [ ] No secrets in the repo; injected at runtime; rotated; redacted from logs; scanned in CI.
- [ ] TLS in transit; sensitive data + backups encrypted at rest; data minimized & classified.
- [ ] Dependencies pinned, scanned, SBOM produced; artifacts **signed** with provenance.
- [ ] CI/CD hardened (least-privilege tokens, actions pinned by digest).
- [ ] Process runs least-privilege; attack surface minimized (loopback bind, closed ports, minimal image).
- [ ] Security woven through every lifecycle phase, not bolted on at the end.

---

## References

- OWASP Top 10 (2025) — <https://owasp.org/Top10/2025/en/>
- OWASP Proactive Controls — <https://top10proactive.owasp.org/> · ASVS — <https://owasp.org/www-project-application-security-verification-standard/>
- OWASP Cheat Sheet Series — <https://cheatsheetseries.owasp.org/>
- STRIDE / threat modeling — <https://owasp.org/www-community/Threat_Modeling>
- SLSA — <https://slsa.dev/> · Sigstore — <https://www.sigstore.dev/>
- NIST Secure Software Development Framework (SSDF) — <https://csrc.nist.gov/Projects/ssdf>
- CIA triad — [04 Principles](04-principles.md#cia)
