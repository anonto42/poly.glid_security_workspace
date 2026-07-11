# Token-Efficient Communication Rule

## Goal

Use the fewest tokens that still produce a correct, clear, safe, and verifiable
result. Optimization means removing waste, not hiding necessary information.

## Responses

- Lead with the outcome, answer, blocker, or required decision.
- Include only information needed to understand the result and act on it.
- Do not repeat the request, tool output, earlier commentary, or unchanged context.
- Prefer a short paragraph or compact list over many headings and long explanations.
- Link to changed files instead of pasting their full contents.
- Report verification briefly: command/check, pass or fail, and material warnings.
- Mention risks, assumptions, failures, or required approvals when they affect use.
- Give deeper explanation, examples, or alternatives only when requested or needed
  for a high-risk or complex decision.
- Do not add generic praise, filler, unnecessary summaries, or automatic next steps.

## Context and tools

- Read the smallest relevant files and line ranges before expanding scope.
- Search first; do not load whole directories or large generated files blindly.
- Reuse existing memory, plans, templates, and verified results.
- Parallelize independent checks when useful, but do not duplicate work.
- Limit command output and ignore build artifacts, dependency trees, and caches unless
  they are directly relevant.
- Do not rerun successful checks without a concrete reason.
- Store concise verified conclusions in `.agents`; do not store raw output or chat.

## Exceptions

Use additional detail when required for:

- Security, privacy, destructive actions, migrations, or production operations.
- Complex architecture decisions and disputed tradeoffs.
- Debugging evidence needed to reproduce a failure.
- User requests for detailed teaching, documentation, or exhaustive review.

Correctness and safety always take priority over token reduction.
