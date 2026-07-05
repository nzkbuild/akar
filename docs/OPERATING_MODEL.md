# AKAR Operating Model

## What AKAR is

AKAR is a local runtime governance layer for Claude Code.
It classifies tasks, enforces diff budgets, runs verification, records telemetry, and summarizes outcomes.
It sits between the user's prompt and Claude Code's execution.

## What AKAR is not

- Not a local LLM
- Not a daemon
- Not a replacement for Claude Code
- Not a skill pack
- Not an autonomous code editor
- Not a cloud service

## Where AKAR sits

```
User prompt
  ↓
AKAR preflight (classify, budget, skill check, request check)
  ↓
Claude Code + model execution
  ↓
AKAR postmortem (telemetry, outcome, learn if needed)
```

---

## D. Onboarding

### First-run setup

The fastest path to a working setup:

```
akar init
```

This runs bootstrap + doctor in sequence and prints a short guide for what to do next. It is idempotent — safe to run again if something looks wrong.

For projects where you also want Claude Code slash commands:

```
akar init --claude
```

This adds instructions for wiring up `/akar-preflight`, `/akar-doctor`, and related slash commands.

### What bootstrap creates

`akar bootstrap` (called internally by `akar init`) copies 9 memory template files into `.akar/` and creates `~/.claude/akar/`. It never overwrites files that already exist.

### First commands after init

```
akar status              — confirm runtime health
akar preflight "task"    — see the strategy before touching anything
akar run "task"          — full workflow in one command
```

---

## E. Lifecycle

### Normal workflow

```
akar init              — one-time project setup
akar doctor            — confirm health before a session
akar preflight "task"  — review strategy before acting
akar run "task"        — full workflow: preflight + mission + postmortem
akar postmortem        — review what happened
akar learn             — propose learning note if degraded or failed
```

### Mission state machine

Every `akar run` or `akar mission` call moves through these states:

```
IDLE
→ INTAKE        classify the prompt
→ BUILD_CONTEXT assemble minimal context pack
→ CONTRACT      produce task contract (type, risk, diff budget)
→ EXECUTE       scaffold mode: record plan, do not modify files
→ VERIFY        run verification recipe
→ REVIEW        check outcome against contract
→ MEMORY_UPDATE compact update to .akar/ if useful
→ DONE
```

In v0.2.x, AKAR is in scaffold mode: it classifies, plans, and records — but does not execute code changes. Real execution is planned for v0.3+.

### Failure path

```
ANY_STATE
→ DETECT_FAILURE
→ FALLBACK
→ DOCTOR_REPAIR (if safe)
→ VERIFY_REPAIR
→ RESUME or BLOCKED
```

If AKAR encounters a failure it cannot safely repair, it reports the state honestly and stops. It never silently continues past a known bad state.

### Session end

AKAR has no background process to stop. Each command runs and exits. Telemetry is written to `.akar/EVENT_LOG.jsonl` at the end of each mission.

---

## F. Passive Runtime

AKAR does not run in the background. It is purely command-driven.

The "passive runtime" is the set of `.akar/` memory files that accumulate over time:

- `EVENT_LOG.jsonl` grows as missions run
- `STATE.md` is updated after each session
- `LESSONS.md` grows as `akar learn` proposes patches

### Observe

```
akar telemetry    — compact view of recent events
akar postmortem   — latest outcome classification
akar status       — full health snapshot
```

### Classify

`akar eval` runs 28 internal scenarios and reports pass/fail. Use this to verify AKAR itself is classifying correctly after an update.

### Prepare

`akar preflight "task"` prepares a strategy before any execution: task type, risk, diff budget, skill conflicts, verification plan, recommendation.

### Record

Every `akar mission` or `akar run` appends a compact event to `EVENT_LOG.jsonl`. Events are redacted (no secrets), append-only, and rotated when they exceed the size threshold.

### Recover

```
akar doctor         — read-only check
akar doctor --fix   — apply safe reversible fixes
akar init           — full re-onboarding if something is broken
```

---

## G. Claude Code Integration

### Optional — AKAR works without it

AKAR is fully functional as a standalone CLI. Claude Code integration adds convenience slash commands inside the editor.

### Slash commands

Copy the provided command files to your project's `.claude/commands/`:

```
cp .claude/commands/akar-*.md <your-project>/.claude/commands/
```

Available slash commands:

| Command | Equivalent CLI |
|---|---|
| `/akar-bootstrap` | `akar bootstrap` |
| `/akar-doctor` | `akar doctor` |
| `/akar-mission` | `akar mission "<task>"` |
| `/akar-verify` | `akar verify` |
| `/akar-status` | `akar status` |
| `/akar-eval` | `akar eval` |

### Hooks

AKAR ships two optional pre-commit hooks:

```
hooks/pre-commit-akar.sh    — bash/POSIX
hooks/pre-commit-akar.ps1   — PowerShell
```

Install by copying to `.git/hooks/pre-commit` (or merging if a hook already exists).

The hook runs `akar doctor` before each commit and blocks the commit if doctor returns issues.

### Settings example

`.claude/settings.akar.json.example` shows how to wire AKAR commands as Claude Code hooks (run on session start, on stop, etc.). Copy and merge into `~/.claude/settings.json` if desired.

### What AKAR does NOT touch

- `~/.claude/settings.json` — never modified automatically
- `~/.claude/CLAUDE.md` — never modified
- Any file outside `.akar/` and `~/.claude/akar/` during normal operation

---

## H. Health and Recovery

### Health states

| State | Meaning |
|---|---|
| `HEALTHY` | Doctor OK, bootstrap OK, no skill conflicts |
| `DEGRADED` | One or more issues found, but AKAR is still usable |
| `BROKEN` | Critical issue — AKAR cannot run reliably until fixed |
| `SELF-HEALING` | `akar doctor --fix` is actively repairing |

Check health at any time:

```
akar status
```

### Doctor behavior

`akar doctor` is always read-only. It checks:

- `.akar/` exists
- `~/.claude/akar/` exists
- Required template files are present
- No critical config issues

`akar doctor --fix` applies safe, reversible fixes:

- Creates missing directories
- Copies missing template files from the templates directory
- Backs up any file before overwriting it
- Never deletes anything

### Recovery expectations

| Issue | Recovery |
|---|---|
| `.akar/` missing | `akar init` or `akar bootstrap` |
| Template file missing | `akar doctor --fix` |
| Skill conflict | Review `.claude/commands/`, disable conflicting skill |
| EVENT_LOG.jsonl corrupt | Delete or truncate the file — telemetry is non-critical |
| Binary outdated | Pull source, `cargo build --release`, replace binary |

### Circuit breaker

Removed in v0.4.1. The circuit breaker module was unused in production paths
and has been deleted. AKAR does not currently block mission execution after
repeated failures — it reports failures via postmortem and learning notes only.

### What AKAR never does during recovery

- Never deletes user files without a backup
- Never modifies git history
- Never touches files outside `.akar/` and `~/.claude/akar/`
- Never silently continues past a known broken state

---

## Expected output format

Every command produces short structured output.

```
status line
  key: value
  key: value

next: hint for what to do
```

No essays. No fake certainty. If something is not verified, AKAR says so.

Final response format for missions:

```
done.

changed:
  - file.rs

verified:
  - cargo build: ok
  - cargo test: ok

not verified:
  - browser rendering (no browser driver)

notes:
  - stayed within diff budget (2 files, 45 LOC)
```
