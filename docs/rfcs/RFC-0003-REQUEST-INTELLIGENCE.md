# RFC-0003: Request Intelligence

- Status: Accepted
- Date: 2026-07-04
- Author: AKAR Kernel Team

## Problem

v0.1.0 treated request limits as hard walls. When the budget ran out, the
mission died mid-task with no continuation state, no summary of what was
done, and no path forward. The overnight run burned requests without any
pressure-aware adaptation, then stopped abruptly.

## Why Now

Request burn was the second-highest-impact gap from the overnight run.
Dead-ending a mission wastes all prior work and forces a full restart.

## Goals

- Treat request budget as a strategy signal, not a wall.
- Define pressure modes with concrete behavioral rules per mode.
- Always produce continuation state before stopping.
- Never hard-dead-end a mission.

## Non-Goals

- Does not increase the request budget.
- Does not bypass Claude Code's native limits.
- Does not promise mission completion under all pressure levels.

## Proposed Design

### RequestPressure Enum

| Level    | Trigger                        | Behavior                                      |
|----------|--------------------------------|-----------------------------------------------|
| Normal   | > 40% budget remaining         | Full operation; no restrictions               |
| Saver    | 25–40% remaining               | Prefer compact tools; skip cold context reads |
| Compact  | 15–25% remaining               | Drop cold context; use compact state only     |
| Boundary | 5–15% remaining                | Finish current step; verify minimally         |
| Resume   | < 5% remaining                 | Write NEXT_RUN.md; stop cleanly               |

### Behavior Per Level

**Normal:** Standard operation. Full context loads. All tools available.

**Saver:** Avoid re-reading files already in context. Prefer grep over full
reads. Skip exploratory steps; focus on the critical path.

**Compact:** Drop any context loaded more than 2 steps ago unless pinned.
Use compact state representation. No new large explorations.

**Boundary:** Complete the current atomic step only. Run minimal verification
(does it exist? does it pass the one critical check?). Do not start any new
step that cannot finish in 2–3 requests.

**Resume:** Write NEXT_RUN.md with: mission_id, last completed step, next
step, blockers, files changed, verification state. Then stop. The file is
the continuation contract for the next session.

### NEXT_RUN.md Format

```yaml
mission_id: <id>
stopped_at: <ISO timestamp>
pressure_level: Resume
last_completed_step: <description>
next_step: <description>
blockers: []
files_changed:
  - <path>
verification_state: <passed|partial|skipped>
notes: <optional>
```

### Pressure Detection

Pressure level is computed from the ratio of requests used to the mission's
declared request budget in the Task Contract. If no budget is declared, AKAR
uses a default of 50 requests and emits a warning.

## Acceptance Criteria

- [ ] Pressure modes documented and linked from kernel policy.
- [ ] NEXT_RUN.md format defined and example written.
- [ ] Resume mechanism tested: session stops cleanly, next session picks up.
- [ ] No mission terminates without either completion state or NEXT_RUN.md.

## Rollback

Remove pressure modes. Treat all limits as hard walls. Reintroduces
overnight-run dead-end behavior.

## Decision

Accepted. Effective v0.1.1 (documented). Implementation target: v0.1.2.
