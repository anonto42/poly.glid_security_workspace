# ADR 0001: Product-oriented project layout

- **Status:** Accepted
- **Date:** 2026-07-14
- **Deciders:** PolyGlid maintainers

## Context

The repository must be navigable by capability name without requiring knowledge
of each component's implementation language or framework. A monorepo mixes
runnable products, shared libraries, contracts, and automation, and each must
declare its own manifest and boundaries so automation and contributors can
discover them consistently.

## Decision

Organize the repository by **product role** rather than by language or by
generic bucket (`lib`, `bin`, `tools`).

```text
apps/        Runnable products: desktop shell and website.
crates/      Reusable workspace libraries: core, runtime, client, config,
             events, plugin-api.
contracts/   Canonical WIT contract shared across host and plugins.
scripts/     CI, verification, packaging, and release automation.
.github/     Workflows, issue, and pull request templates.
docs/        Architecture, development, security, and decision records.
```

Language and framework choices are implementation details declared **inside**
each crate's or app's manifest (`Cargo.toml`, `Dioxus.toml`), not at the
repository top level.

## Consequences

- A developer finds a capability by product name, without knowing its language.
- Each project owns its source, tests, assets, manifest, and documentation.
- Replacing an implementation language does not change the repository's
  top-level information model.
- Automation (Moon, release-please) discovers project boundaries from manifests
  consistently.
- The Dioxus desktop (`apps/desktop`) is the clear product entry point rather
  than a prototype hidden behind a working name.
