name: Bug report
about: Report a bug or unexpected behavior
title: ''
labels: ''
body:
  - type: markdown
    attributes:
      value: |
        Thanks for reporting! Please fill out the information below so we can
        reproduce and fix the issue. Before opening a new bug, check existing
        issues to avoid duplicates.
  - type: textarea
    id: description
    attributes:
      label: Description
      description: A clear description of what the bug is.
    validations:
      required: true
  - type: textarea
    id: reproduction
    attributes:
      label: Steps to reproduce
      description: Minimal, deterministic steps to reproduce the behavior.
      render: bash
      placeholder: |
        1. ...
        2. ...
        3. ...
    validations:
      required: true
  - type: textarea
    id: expected
    attributes:
      label: Expected behavior
      description: What did you expect to happen?
    validations:
      required: true
  - type: textarea
    id: actual
    attributes:
      label: Actual behavior
      description: What actually happened? Include any error text, logs, or
        screenshots.
    validations:
      required: true
  - type: input
    id: environment
    attributes:
      label: Environment
      description: OS, architecture, and PolyGlid version (`cargo run -p polyglid-desktop -- --version` if applicable).
    placeholder: "e.g. Ubuntu 24.04 x86_64, PolyGlid 0.10.1"
  - type: textarea
    id: context
    attributes:
      label: Additional context
      render: markdown
