# Durable Agent Knowledge Rule

## Requirement

Before finishing meaningful work, update `.agents` when the conversation or work
produces information that a future developer or agent would otherwise need to
rediscover.

Meaningful information includes:

- Accepted requirements, scope, priorities, or architectural decisions.
- Verified codebase behavior, limitations, dependencies, and operational commands.
- Plans, milestones, task breakdowns, acceptance criteria, and open questions.
- Implemented changes, migrations, new conventions, and required follow-up.
- Test results, known failures, risks, blockers, and rejected approaches with reasons.
- Agent responsibilities, permissions, handoffs, and integration contracts.

Do not record:

- Greetings, conversational filler, or raw chat transcripts.
- Secrets, tokens, credentials, private keys, or sensitive target data.
- Guesses presented as facts or conclusions that were not verified.
- Large generated outputs, build artifacts, or information already clear from Git.
- The same knowledge in multiple competing files.

## Placement

- Stable codebase facts and commands: `.agents/shared/memory/`.
- Durable cross-day implementation plan: `.agents/shared/plans/`.
- Architecture/security/data conventions: `.agents/shared/rules/`.
- Accepted decisions: today's `history/YYYY-MM-DD/decisions.md`; promote stable
  repository-wide decisions to a matching rule or plan.
- Work completed today: today's `updates.md` and compressed `caveman.md`.
- Repository-wide convention or migration: `.agents/shared/changes/`.
- Delegated tasks and ownership: today's `handoffs.md`.

## Quality standard

Every retained entry should say:

1. What was learned, decided, or changed.
2. Why it matters.
3. Evidence or affected paths where applicable.
4. Current status and next action.
5. Date when the information can become stale.

Keep each file below 200 lines, update an existing canonical record when one
exists, and use Asia/Dhaka dates. `.agents` is curated knowledge, not an activity
dump. Source files, tests, Git history, and approved decisions remain canonical.
