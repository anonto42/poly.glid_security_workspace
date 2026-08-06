# Security Model

PolyGlid is local-first: all project data stays on the user's machine. Even so,
the codebase enforces explicit safety controls that contributors must preserve.

## Supported Versions

Security fixes are applied to the `main` branch until the first stable release.
Older commits are not supported.

## Reporting

Do **not** open public issues for vulnerabilities. Use [GitHub Security
Advisories](https://github.com/anonto42/polyglid/security/advisories/new) and
see [SECURITY.md](../SECURITY.md).

## Core Controls

- **Deny by default.** Plugins are untrusted. Every requested capability must be
  approved by the permission store and the active security profile before it
  reaches the Wasmtime host-call boundary. The `CoreEngine` forwards the exact
  request for a second enforcement check.
- **Least privilege.** Host capabilities are scoped to the run target. The WIT
  contract exposes only `dns.resolve` (scoped host) and `reports.write`
  (scoped to `reports_dir`). Filesystem, process, environment, and raw network
  access are never exposed without a designed host capability and tests.
- **Signature verification.** Plugins are signature-checked per the active
  security profile (`Strict`, `Balanced`, `Development`). Local plugins require a
  signature under the Strict profile; revoked or untrusted publishers are
  rejected.
- **Runtime limits.** Plugin execution is bounded by `max_wasm_fuel`.
- **Safe paths.** Output paths are validated; permanent project-folder deletion
  requires explicit confirmation and is limited to a direct child of the
  registered workspace.
- **Auditable events.** Permission grants, denials, check failures, and
  run start/finish events flow through `polyglid-events` so the UI can surface
  them and the store can audit them.

## Sensitive Data

Never commit secrets, tokens, signing keys, local configuration, runtime data,
or private project files. See [SECURITY.md](../SECURITY.md) for the full list.

## Disclosure Process

Maintainers review reports, confirm impact, prepare a fix, and publish release
notes when appropriate. Reporters may be credited publicly on request.
