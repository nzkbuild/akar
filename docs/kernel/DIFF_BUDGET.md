# Diff Budget

## Size Classes

**Micro** — 1-2 files, 1-30 LOC
- No new abstractions, no new dependencies
- No plan required

**Small** — 2-5 files, 30-200 LOC
- No new dependencies
- Brief inline reasoning sufficient

**Medium** — 5-12 files, 200-600 LOC
- Internal plan required before editing
- No new external dependencies without flagging

**Large** — 12+ files or 600+ LOC
- Explicit mission statement required
- Checkpoint required at start and end

**Critical** — any size touching auth, payment, db schema, legal, or security
- Research + plan + checkpoint required
- Must surface impact before proceeding

## Rules

Must:
- Classify the task before starting
- Reclassify if scope grows during execution
- Stop and surface scope expansion rather than silently absorbing it

Should:
- Keep a small task small — resist the urge to clean up surroundings

Never:
- Let a micro task silently become a medium task
- Add abstractions or deps not required by the task
