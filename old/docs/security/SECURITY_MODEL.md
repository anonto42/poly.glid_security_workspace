# Security Model

PolyGlid should be designed as an authorized security testing and defensive diagnostics workspace. The architecture must assume plugins can be buggy, untrusted, or hostile.

## Trust Boundaries

```text
Trusted:
- Rust host engine
- permission store
- config validation
- event router

Conditionally trusted:
- installed plugins after validation
- local config files

Untrusted by default:
- third-party plugins
- plugin input
- network targets
- imported reports
```

## Default Rule

All sensitive capabilities are denied by default.

Plugins cannot directly:

- read arbitrary files
- write arbitrary files
- open network connections
- listen on ports
- spawn processes
- read environment variables
- access SSH keys or tokens

The host may expose a narrow capability after validation and user approval.

## Capability Types

Initial capability model:

```text
NetworkConnect
NetworkListen
FilesystemRead
FilesystemWrite
ConfigRead
ReportWrite
Crypto
DnsResolve
```

Each capability should support scope.

Examples:

```text
NetworkConnect:
  allowed_hosts: ["example.com"]
  allowed_ports: [80, 443]

FilesystemRead:
  allowed_paths: ["~/polyglid/wordlists"]

ReportWrite:
  allowed_paths: ["~/polyglid/reports"]
```

## Plugin Manifest

Every plugin should declare requested capabilities.

First-party plugins use a `polyglid.toml` manifest next to the plugin source or
next to the built component.

```toml
id = "polyglid.recon_probe"
name = "Recon Probe"
version = "0.1.0"
entry_world = "security-tool"
capabilities = ["dns-resolve", "network-connect"]
```

Scoped requests use explicit tables:

```toml
[[capability_requests]]
capability = "network-connect"
host = "example.com"
port = 443

[[capability_requests]]
capability = "filesystem-read"
path_prefix = "/tmp/polyglid"
```

The host compares requested capabilities against user-approved permissions.
The CLI development harness grants a capability explicitly with
`--allow <capability>`; no capability is approved by default.

Persistent approvals live in an app config loaded with `POLYGLID_CONFIG`.
See `config.example.toml` for `[[approved_capabilities]]` examples. Temporary
CLI `--allow` grants are useful for development but should not replace explicit
stored approvals in real workflows.

## Runtime Controls

The host runtime should enforce:

- execution timeout
- memory limit
- maximum concurrent tasks
- cancellation
- denied-by-default capabilities
- structured error reporting
- audit log for sensitive actions

PolyGlid currently enforces a deterministic Wasmtime fuel budget through
`max_wasm_fuel` in config. A plugin that exhausts its fuel traps instead of
running indefinitely.

## Host Capability Adapters

The first concrete adapters are narrow WIT imports:

- `dns.resolve` resolves only the current run target instead of exposing raw
  networking.
- `reports.write` writes a named report file under the configured `reports_dir`
  and rejects absolute paths, separators, and traversal.

Core denies execution unless the plugin manifest requests the matching
capability and the host approval store grants it.

## Safety Position

PolyGlid should avoid building or shipping features whose main purpose is unauthorized access, persistence, stealth, credential abuse, or command-and-control activity. If a feature has dual-use risk, design it as an authorized defensive audit or lab-only validation tool with explicit guardrails.
