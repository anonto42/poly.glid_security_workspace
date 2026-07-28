# Security Boundaries

## Project Filesystem

- Validate workspace and project names as one safe path segment.
- Discovery accepts only direct, non-symlink child folders.
- Permanent deletion requires explicit UI confirmation.
- Before recursive deletion, canonicalize both paths and require the project to
  be a non-symlink direct child of the workspace root.
- Never weaken these checks for convenience or test setup.

## Plugins and Runtime

- Verify plugin identity, manifest, checksum, signature, and publisher trust at
  the appropriate lifecycle boundary.
- Capability decisions must include the complete binding used by execution.
- Default to denial when a grant is absent, expired, revoked, or outside scope.
- Keep Wasmtime fuel, epoch interruption, and memory limits active.
- Do not inherit ambient filesystem, environment, network, or process access
  into the WASI context.
- Host operations must check effective grants and their structured scope.
- Report filenames must remain relative, single-component, and traversal-free.
- Surface security failures explicitly; do not downgrade them to success or
  silently ignore them.
