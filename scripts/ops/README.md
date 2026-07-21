# PolyGlid Operations

`polyglid-ops.mjs` is the stable entry point for repository automation. GitHub
Actions and local development use the same commands so workflow logic stays out
of YAML where possible.

```bash
npm run ops -- help
npm run ops -- detect HEAD~1 HEAD
npm run ops -- validate
npm run ops -- site-build
npm run ops -- repo-sync
```

## Responsibilities

- `polyglid-ops.mjs` dispatches commands and propagates failures.
- `detect-changes.sh` classifies changed paths without performing work.
- `sync-repo.mjs` applies `repinfo.json` through the GitHub CLI.
- `deploy-site.yml` owns GitHub Pages deployment.
- `repo-sync.yml` owns repository metadata synchronization.
- `ci.yml` owns validation and build/test routing.

Repository synchronization requires `GITHUB_REPOSITORY` and an authenticated
`gh` CLI. In Actions, authentication comes from the `GH_PAT` repository secret.
