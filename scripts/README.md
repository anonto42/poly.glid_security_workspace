# Project Scripts

Repository automation lives here instead of being embedded in workflow YAML.

- `ci/` contains validation, pinned tool setup, platform dependency setup, and
  workspace verification.
- `release/` contains release lockfile synchronization, platform packaging,
  AppImage metadata validation, and GitHub release publication. AppImage
  packaging bundles the runtime-directory guidance used by first-run setup.
- `site/` contains the reproducible Dioxus Web production build.

Run scripts from the repository root unless a script says otherwise.
