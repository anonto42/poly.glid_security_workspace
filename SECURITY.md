# Security Policy

## Supported Versions

Until the first stable release, security fixes are handled on the `main`
branch.

| Version | Supported |
| --- | --- |
| `main` | Yes |
| Older commits | No |

## Reporting a Vulnerability

Please do not create a public GitHub issue for security vulnerabilities.

Report security issues privately through [GitHub Security Advisories](https://github.com/anonto42/poly.glid_security_workspace/security/advisories/new)
for this repository. Include as much useful detail as possible:

- Affected component, command, or feature
- Steps to reproduce
- Potential impact
- Suggested fix or mitigation, if known

## Sensitive Data

Never include these in issues, pull requests, logs, screenshots, or example
files:

- API keys, access tokens, signing keys, or passwords
- Local configuration and runtime data directories
- Private project files or paths that identify users or organizations
- Production databases, logs, or plugin packages containing confidential data

## Security Boundaries

PolyGlid is local-first, but contributors must preserve its safety controls:

- Permanent project-folder deletion requires explicit confirmation and is
  limited to a direct child of the registered workspace.
- Plugins use least-privilege capabilities and signature verification.
- The Wasmtime plugin runtime has resource limits.
- Output paths must be validated before files are written.

## Disclosure Process

Maintainers will review the report, confirm its impact, prepare a fix, and
publish release notes when appropriate. Credit may be given to reporters who
want public acknowledgement.
