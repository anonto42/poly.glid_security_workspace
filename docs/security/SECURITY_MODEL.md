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

```yaml
id: polyglid.recon_probe
name: Recon Probe
version: 0.1.0
entry_world: security-tool
capabilities:
  - DnsResolve
  - NetworkConnect
```

The host compares requested capabilities against user-approved permissions.

## Runtime Controls

The host runtime should enforce:

- execution timeout
- memory limit
- maximum concurrent tasks
- cancellation
- denied-by-default capabilities
- structured error reporting
- audit log for sensitive actions

## Safety Position

PolyGlid should avoid building or shipping features whose main purpose is unauthorized access, persistence, stealth, credential abuse, or command-and-control activity. If a feature has dual-use risk, design it as an authorized defensive audit or lab-only validation tool with explicit guardrails.

