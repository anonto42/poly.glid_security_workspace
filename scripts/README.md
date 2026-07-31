# Project Scripts

Repository automation lives here instead of being embedded in workflow YAML.

- `ci/` contains validation, pinned tool setup, platform dependency setup, and
  workspace verification.
- `release/` contains release lockfile synchronization, platform packaging,
  and GitHub release publication.
- `site/` contains the reproducible Dioxus Web production build.

Run scripts from the repository root unless a script says otherwise.
