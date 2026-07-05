# AKAR Scope Drift Report

Date: 2026-07-05
Basis: original Phase 0/1 instruction vs actual v0.3.0 state.

---

## Original mission (Phase 0 + Phase 1 only)

Phase 0: audit the repo, write AKAR_ADOPTION_NOTES.md, identify first safe target.

Phase 1: Rust CLI scaffold with:
- `akar --version`
- `akar status` (placeholder)
- `akar doctor` (placeholder)
- `akar bootstrap` (placeholder)
- `akar verify` (placeholder)
- `akar eval` (placeholder)
- Folder structure: docs/kernel/, templates/, evals/, examples/, tests/
- Minimal README

Explicit constraints: "No complex architecture yet. No hook integration yet.
No doctor --fix yet. No memory engine yet."

---

## What was actually built

### Modules implemented beyond Phase 1 scope

| Module | Scope | Comment |
|---|---|---|
| contract.rs | Phase 1+ | Full prompt classifier, 13 task types, 4 risk levels, diff budgets |
| context_pack.rs | Phase 1+ | Full hot/warm/cold tier file scanner |
| mission.rs | Phase 1+ | Full state machine, telemetry, context pack, contract, verify |
| postmortem.rs | Phase 1+ | JSONL parser, outcome classifier, follow-up generator |
| event_log.rs | Phase 1+ | Full append-only log with manual ISO 8601 date math |
| learn.rs | Phase 1+ | Learning patch generator |
| preflight.rs | Phase 1+ | Full strategy advisor combining 4 subsystems |
| request_intelligence.rs | Phase 1+ | Pressure mode advisor, NEXT_RUN.md writer |
| skill_registry.rs | Phase 1+ | Full skill scanner, role classifier, conflict detector |
| model_profile.rs | Phase 1+ | Model capability profiles, drift detection, git integration |
| workflow.rs | Phase 1+ | Full pipeline: doctor→preflight→mission→postmortem |
| design.rs | Phase 1+ | Design DNA check, frontend file scanner |
| circuit_breaker.rs | Phase 1+ | Circuit breaker struct (unused in production paths) |
| safe_fix.rs | Phase 1+ | Doctor --fix engine with backup |
| backup.rs | Phase 1+ | File backup/restore with timestamp suffixes |
| init.rs | Phase 1+ | First-run onboarding with shell detection |

Only `config.rs`, `bootstrap.rs`, `doctor.rs`, `verify.rs`, `eval.rs`,
and `safety.rs` are plausibly in Phase 1 scope. 16 of 23 modules are drift.

### Commands added beyond Phase 1 scope

Phase 1 specified 5 commands. v0.3.0 has 17 commands.

Beyond scope: `mission`, `run`, `preflight`, `request`, `postmortem`,
`telemetry`, `learn`, `skills`, `calibrate`, `hooks`, `init`.

### Version inflation

Phase 1 implied a scaffold at ~v0.1.x. The project reached v0.3.0 — 12
minor versions in a single development session.

### Test count

194 tests for a scaffold. Phase 1 said "tests if practical."

---

## How drift happened

Each session extended the previous one without re-checking original constraints.
The session instructions were framed as "implement AKAR" (the full product),
not "stay in Phase 1." Each module was locally justified but collectively
exceeded the phase boundary.

This is not a failure — the code is real and works. But the audit mission
asks for honesty: the v0.3.0 label implies stability and polish that was
not earned through real-world use. It was earned through session velocity.

---

## What changed character, not just scope

1. **28 evals** are the most honest thing in the codebase. They test real
   behavior across real inputs. But several are smoke tests (see Reality Audit).

2. **The event log** is a genuine persistence mechanism. It works correctly.
   But it records AKAR's own state machine transitions, not real Claude Code
   events.

3. **Skill conflict detection** reads the real ~/.claude/ directory. It finds
   real skills. The conflicts it reports (200 skills, 3 conflicts) are real.
   But the enforcement is zero — it's a warning printer.

4. **Secret redaction** in config.rs is a genuine safety feature, well-tested,
   and correctly applied throughout. This is good drift.

---

## Summary verdict

AKAR v0.3.0 is a well-built advisory CLI. Its architecture is coherent and
its modules are internally consistent. But it was built at Phase 4-5 velocity
while the stated mission was Phase 1. The result is a product with more surface
area than depth — many commands, all report-only, none enforced.
