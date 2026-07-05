# AKAR v0.3.0 Reality Audit

Date: 2026-07-05
Auditor: post-compaction session, read every src/*.rs file + key docs.

---

## 1. What is AKAR actually today?

A Rust CLI binary with 23 modules, 194 tests, and 28 evals. It classifies
user prompts, prints strategy reports, writes local telemetry, and suggests
learning patches. It does not execute code, edit files, or integrate with
Claude Code at runtime.

## 2. What does it truly do end-to-end?

```
akar run "fix the login button"
  1. doctor: checks .akar/ and ~/.claude/akar/ exist (config::validate)
  2. preflight: classifies prompt via keyword rules, prints diff budget
  3. mission: walks a state machine (Intake -> Done), writes one JSONL event
  4. postmortem: reads the JSONL event back, classifies outcome
```

That is the full loop. Every step is report-only. No file is read, written,
or modified in the user's project.

## 3. What is still scaffold or report-only?

Everything. Specifically:

- **Mission Execute state**: logs "execute: skipped in scaffold mode" (mission.rs:135)
- **Mission Verify state**: logs recipe label but never runs commands (mission.rs:153)
- **Mission MemoryUpdate state**: logs "skipped in scaffold mode" (mission.rs:171)
- **Context pack**: builds a file list but never reads file contents
- **Learning patches**: writes a Markdown stub with generic rules, never applies them
- **Circuit breaker**: struct exists, never used outside its own tests
- **Design module**: checks if DESIGN_DNA.md exists, nothing else
- **Model profile**: reads env vars, returns hardcoded heuristics, never persists
- **Session fingerprint / drift detection**: fully implemented, never called from main.rs
- **Dependency governor**: fully implemented, never called from main.rs
- **Migration safety check**: fully implemented, never called from main.rs

## 4. What commands are real vs advisory?

| Command | Real | Advisory |
|---|---|---|
| `akar --version` | Real | - |
| `akar status` | Real | - |
| `akar doctor` | Real (checks dirs exist) | - |
| `akar doctor --fix` | Real (creates dirs, copies templates) | - |
| `akar bootstrap` | Real (copies .md templates) | - |
| `akar init` | Real (bootstrap + doctor + shell detect) | - |
| `akar verify` | Real (runs cargo build/test) | - |
| `akar eval` | Real (28 behavioral checks) | - |
| `akar safety` | Real (keyword classification) | Advisory (no enforcement) |
| `akar skills` | Real (scans .claude/commands/) | Advisory (no enforcement) |
| `akar calibrate` | Real (reads env vars) | Advisory (no persistence) |
| `akar preflight` | - | Advisory (print-only) |
| `akar request` | - | Advisory (print-only) |
| `akar mission` | Partial (state machine, no execution) | Advisory |
| `akar run` | Partial (chains above, no execution) | Advisory |
| `akar postmortem` | - | Advisory (reads telemetry) |
| `akar telemetry` | - | Advisory (reads log) |
| `akar learn` | - | Advisory (writes stub) |
| `akar hooks` | - | Print-only (no install) |

## 5. Test coverage assessment

**Meaningful tests:**
- config::redact() — 6 tests covering sk- keys, bearer, hex, kv secrets
- safety::classify_command() — 8 tests covering risk levels
- event_log — append, read, rotate, json escape
- bootstrap — idempotency, no-overwrite guarantee
- verify — recipe detection, failure classification, run_recipe
- backup — backup/restore cycle, find_latest_backup
- contract — prompt classification across 7 task types
- skill_registry — role classification, conflict detection, scan_multi

**Shallow or self-fulfilling tests:**
- `run_mission_reaches_done_state` — always passes because nothing can fail in scaffold mode
- `workflow_returns_nonempty_report` — asserts !is_empty(), not correctness
- `eval_helper_constructs_correctly` — tests a struct constructor
- `build_pack_smoke` — asserts total_files >= 0 (always true)
- `detect_shell_returns_a_value` — asserts it doesn't panic
- `doctor_check` eval — passes if it doesn't panic
- `design_check` eval — passes if it doesn't panic
- `request_pressure_compaction` eval — asserts "compact" != "stop" (tautology)

Of 194 tests, approximately 30-40 are smoke tests that assert non-panic or
non-empty. They provide confidence the code runs, not that it's correct.

## 6. Docs that overclaim

- README says "Classifies your task before the agent touches anything" — true
- README says "Enforces a diff budget so a small fix stays small" — false,
  it prints a diff budget but does not enforce anything
- README says "Detects skill conflicts" — true, but only reports them
- README says "Records local-only telemetry after each mission" — true
- README says "Proposes a learning note if something degraded or failed" — true,
  but the learning note is a generic stub, not derived from actual failure analysis
- OPERATING_MODEL.md describes a "passive runtime" that "observes, classifies,
  prepares, records, recovers" — only observes, classifies, and records are real
- INSTALL.md describes the full architecture as if it's operational

## 7. Private/local paths that could leak

- `.akar/EVENT_LOG.jsonl` — contains project names, prompts (redacted), timestamps
- `.akar/SKILL_INVENTORY.md` — lists user's installed skills
- `.akar/LEARNING_PATCHES.md` — contains project names and failure descriptions
- `~/.claude/akar/` — global state directory
- `SKILL_INVENTORY.md` is in .gitignore — good
- `EVENT_LOG.jsonl` is in .gitignore — good
- `LEARNING_PATCHES.md` is NOT in .gitignore — risk if user commits .akar/ contents

## 8. What will fail first in real Claude Code use?

1. **The mission has no execution.** A user running `akar run "fix the bug"`
   expects something to happen. Nothing does. The report says "scaffold mode"
   but a new user will think it's broken.

2. **Skill conflict detection is noisy.** It scans all 200+ skills in
   ~/.claude/ and reports conflicts between skills the user isn't even using
   in the current session. Every preflight says "resolve skill conflicts first."

3. **Request intelligence has no real signal.** It counts events in EVENT_LOG.jsonl
   and maps to pressure modes. Without real API usage data, the thresholds
   (20 events = Saver, 50 = Compact) are arbitrary.

4. **Learning patches are generic.** Every failure gets the same rule:
   "Investigate failure before retrying." There's no actual analysis of
   what went wrong.

5. **No hook integration.** The `akar hooks` command prints instructions
   but doesn't install anything. Claude Code can't call AKAR automatically.
