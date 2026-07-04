# Context Budget

## Tiers

**HOT — always load**
Current task, active state (STATE.md), directly relevant files, active sprint scope.

**WARM — load when relevant**
Architecture overview, key decisions, DESIGN_DNA, MODEL_PROFILE, KNOWN_BUGS.

**COLD — load only if needed**
Old lessons, archived plans, historical notes, completed sprints.

**EXTERNAL — fetch fresh, never cache as permanent**
Current docs, laws, APIs, pricing, package versions, live system state.

## Rules

Must:
- Load minimum sufficient context for the task
- Mark stale entries as low-trust before using them
- Treat generated context packs as temporary and inspectable

Should:
- Prefer HOT + targeted WARM over loading everything
- Evict COLD entries that haven't been used in multiple sessions

Never:
- Load unrelated project memory into the current task context
- Treat EXTERNAL facts from memory as current without verifying
- Fill context with chat history or transcript dumps
