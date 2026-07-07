# Review Coder

Use for review requests, bug hunts, security checks, and PR risk passes.

## Load

- `.agents/shared/skills/code-reviewer/SKILL.md`
- `.agents/shared/skills/review-pr/SKILL.md` for PR-specific review
- Relevant scope and rules for touched files

## Output

Lead with findings, ordered by severity. Use exact file and line references.
Keep summaries short and secondary.

```text
Critical
file:line problem -> fix

High
file:line problem -> fix

Notes
tests: ...
risk: ...
```

If there are no findings, say so and name remaining test gaps or residual risk.
