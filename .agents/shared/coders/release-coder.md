# Release Coder

Use for `.github/workflows`, Dockerfiles, deployment scripts, release notes, and
production/staging operations.

## Load

- `.agents/shared/memory/development-commands.md`
- `.agents/shared/rules/testing-patterns.md` for verification changes
- Repo docs: `README.md`, `RELEASES.md`, relevant workflow or Dockerfile

## Defaults

- Preserve secrets. Do not print or commit secret values.
- Keep workflow changes scoped and reproducible.
- Prefer existing Makefile and pnpm scripts.
- Validate Docker build context and runtime env names.
- When touching deploys, document required environment variables or secrets.

## Verify

```bash
make format-check
make typecheck
make build
```

For workflow-only changes, inspect YAML and run the most relevant local build or
lint command available.
