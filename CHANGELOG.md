# Changelog

## v0.2.2 — 2026-07-04
Public GitHub polish: repo cleanup, README rewrite, CONTRIBUTING, SECURITY, CHANGELOG. Removed tracked runtime artifacts (.akar/, reports/). No new runtime features.

## v0.2.1 — 2026-07-04
Install guide, operating model, evaluation plan docs. Updated README with full command table and docs links.

## v0.2.0 — 2026-07-04
First stable runtime. Added `akar run` (stable workflow command). Upgraded `akar status` to show doctor/bootstrap/telemetry/postmortem/skills/request in one view. 28/28 evals passing.

## v0.1.9 — 2026-07-04
Mission preflight wiring. `akar preflight "<task>"` combines contract classification, request pressure, skill intelligence, and verification into one pre-execution strategy report.

## v0.1.8 — 2026-07-04
Skill intelligence runtime wiring. `akar skills` scans global + project commands, detects role conflicts (methodology + execution controller combo), writes SKILL_INVENTORY.md.

## v0.1.7 — 2026-07-04
Request Intelligence v0. `akar request` shows pressure mode (Normal/Saver/Compact/Boundary/Resume) from explicit counts or local telemetry signals.

## v0.1.6 — 2026-07-04
Learning Patch v0. `akar learn` reads postmortem evidence and proposes LP-NNNN patches for degraded/failed/unknown outcomes.

## v0.1.5 — 2026-07-04
Postmortem wiring. `akar postmortem` reads EVENT_LOG.jsonl and classifies latest outcome as clean/degraded/failed/unknown.

## v0.1.4 — 2026-07-04
Runtime telemetry wiring. `akar telemetry` shows event log summary. Mission flow appends compact events to `.akar/EVENT_LOG.jsonl`.

## v0.1.3 — 2026-07-04
Runtime hygiene. Zero warnings. .gitignore policy. Release checklist.

## v0.1.2 — 2026-07-04
Real `akar bootstrap`. Creates `.akar/` and copies 9 memory templates idempotently. `akar doctor` returns OK after bootstrap.

## v0.1.1 — 2026-07-04
Architecture refinement. AKAR OS framing, RFC system, Skill Intelligence docs, 25 evals.

## v0.1.0 — 2026-07-03
Initial Rust CLI scaffold. 15 modules, 121 tests, 20 evals passing. Core commands: status, doctor, bootstrap, verify, eval, mission, safety, skills, calibrate, hooks.
