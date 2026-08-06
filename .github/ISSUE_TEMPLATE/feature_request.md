name: Feature request
about: Suggest a product or developer-experience improvement
title: ''
labels: ''
body:
  - type: markdown
    attributes:
      value: |
        Thanks for the suggestion! Keep it focused and explain the problem, not
        just the proposed solution. Check existing issues before filing to
        avoid duplicates.
  - type: textarea
    id: problem
    attributes:
      label: Problem
      description: What problem does this feature solve? What can't you do today?
    validations:
      required: true
  - type: textarea
    id: solution
    attributes:
      label: Proposed solution
      description: How should it work? Include any design or UX details.
    validations:
      required: true
  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives considered
      description: What else did you consider?
  - type: input
    id: affected
    attributes:
      label: Affected area
      description: Where is this change needed? (desktop, website, crates, CI/release, docs.)
