# Legacy `.agents-extra` Summary

Source removed after merge: `.agents-extra/`

The merged folder described a DNS-focused monorepo, not PolyGlid.

Useful historical facts:

- It expected a Go backend and React/TypeScript frontend.
- It referenced `make setup`, `make dev`, `make test`, and generated OpenAPI
  client workflows.
- It described DNS records, DNSSEC, zone validation, registrar flows, and audit
  logging as the primary product domain.
- It stored memory notes under `.agents-extra/shared/memories/`.

Do not apply these as active PolyGlid rules. PolyGlid is currently a Rust CLI and
Wasm component runtime with plugins under `plugins/`.
