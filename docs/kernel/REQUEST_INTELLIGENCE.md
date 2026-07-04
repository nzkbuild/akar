# Kernel Policy: Request Intelligence

## Core Rule

Request budget is a strategy signal, not a wall.

## Pressure Modes

| Level    | Budget Remaining | Behavior                                      |
|----------|------------------|-----------------------------------------------|
| Normal   | > 40%            | Full operation. All tools and context available.|
| Saver    | 25–40%           | Compact tools preferred. No re-reads. Skip exploration. |
| Compact  | 15–25%           | Drop cold context. Use compact state only.    |
| Boundary | 5–15%            | Finish current step. Minimal verify. No new steps. |
| Resume   | < 5%             | Write NEXT_RUN.md. Stop cleanly.              |

## Behavior by Level

### Normal
Standard operation. Full context loads. Exploratory reads allowed.

### Saver
- Prefer grep over full reads.
- Skip files already in context.
- Focus on the critical path only.

### Compact
- Drop any context loaded > 2 steps ago unless explicitly pinned.
- Use compact state representation.
- No new explorations larger than 1 file.

### Boundary
- Complete the current atomic step.
- Run one minimal verification check.
- Do not start a step that cannot finish in 2–3 requests.

### Resume
- Write NEXT_RUN.md (see format below).
- Stop cleanly. No further requests in this session.

## NEXT_RUN.md Format

```yaml
mission_id: <id>
stopped_at: <ISO timestamp>
pressure_level: Resume
last_completed_step: <description>
next_step: <description>
blockers: []
files_changed:
  - <path>
verification_state: <passed | partial | skipped>
notes: <optional>
```

## Rules

- Never hard-dead-end a mission without continuation state.
- If no request budget declared, default to 50 requests and emit warning.
- Pressure level is computed, not manually set.
- NEXT_RUN.md is the continuation contract for the next session.

## Effective

v0.1.1 (documented). v0.1.2 (live). See RFC-0003.
