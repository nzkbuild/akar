# Memory Schema

## Files

| File | Purpose |
|------|---------|
| PROJECT_DNA.md | Goals, stack, constraints, non-negotiables |
| STATE.md | Current goal, done, next, blockers |
| DECISIONS.md | Key decisions and rationale |
| LESSONS.md | Durable lessons learned |
| KNOWN_BUGS.md | Active known bugs |
| TEST_DEBT.md | Untested areas and coverage gaps |
| DESIGN_DNA.md | Visual/UX principles and component patterns |
| MODEL_PROFILE.md | Observed model behaviors and quirks |
| EVENT_LOG.jsonl | Timestamped event stream |
| VERIFY_RECIPE.md | How to verify this project end-to-end |

## Entry Format

Each lesson/decision entry must include:
- `date` — when recorded
- `scope` — global or project slug
- `confidence` — low / medium / high
- `source` — observed / user_decision / verified_docs / inferred
- `expires` — date or "never"
- `summary` — one sentence
- `prevention` — one sentence (for bugs/lessons)
- `supersedes` — entry ID if this replaces an older entry

## Rules

Never:
- Write essay-style memory entries
- Dump chat transcripts into memory
- Store secrets or credentials
- Record external facts (pricing, versions, APIs) as permanent truth
