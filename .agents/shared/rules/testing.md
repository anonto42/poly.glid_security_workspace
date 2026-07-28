# Testing and Verification

- Add focused tests near changed domain behavior.
- Filesystem tests must use isolated temporary folders and clean up explicitly.
- Cover rejection paths for traversal, symlinks, invalid names, missing grants,
  expired or revoked approvals, invalid signatures, and unsafe report paths
  when those areas change.
- For desktop changes, verify loading, empty, success, error, disabled, and
  confirmation states affected by the change.
- Run targeted package checks first.
- Run formatting, locked workspace check, and locked workspace tests before
  completing shared or cross-layer changes.
- Do not hide flaky behavior with arbitrary sleeps.
