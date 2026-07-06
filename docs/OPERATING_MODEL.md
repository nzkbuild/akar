# AKAR Operating Model

## What AKAR is

AKAR is a local, **advisory-only** runtime governance layer for Claude Code.
It classifies tasks, reports diff budgets, runs verification recipes on demand, records local telemetry, and summarizes outcomes.
It sits alongside the user's prompt and Claude Code's execution — it prints advice and records what happened, but does not execute the task, edit project files, call models, or run missions.

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

This prints instructions for wiring up the AKAR slash commands that ship in `.claude/commands/`.

### What bootstrap creates

`akar bootstrap` (called internally by `akar init`) copies 9 memory template files into `.akar/` and creates `~/.claude/akar/`. It never overwrites files that already exist.

### First commands after init

```
akar status              — confirm runtime health
akar preflight "task"    — see the strategy before touching anything
akar run "task"          — full advisory workflow in one command (prints strategy + records telemetry; does not execute)
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

AKAR is in scaffold mode by design (v1 architecture freeze): it classifies, plans, and records — but does not execute code changes. Execution is not a version lag; it is explicitly out of scope until a v1.0 design review authorizes a bounded execution path.

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
- `STATE.md` is a template the user edits; AKAR does not auto-update it after a session
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

Available slash commands (these are the only `.claude/commands/akar-*.md` files AKAR ships):

| Command | Equivalent CLI |
|---|---|
| `/akar-bootstrap` | `akar bootstrap` |
| `/akar-doctor` | `akar doctor` |
| `/akar-mission` | `akar mission "<task>"` |
| `/akar-verify` | `akar verify` |
| `/akar-status` | `akar status` |
| `/akar-eval` | `akar eval` |

Note: `/akar-preflight` and `/akar-doctor-fix` are sometimes referenced in older notes but do not exist as command files. Use the CLI directly (`akar preflight "<task>"`, `akar doctor --fix`).

### Hooks

The current, proven hook is a **Claude Code PreToolUse** hook (not a git pre-commit hook). AKAR ships two templates:

```
templates/hooks/pre-tool-call.sh    — bash/POSIX
templates/hooks/pre-tool-call.ps1   — PowerShell
```

The hook calls `akar safety "<command>"` before each Bash tool call, blocks destructive commands (exit 2), and logs every decision to `.akar/HOOK_EVENTS.jsonl`. This is the only piece of real runtime enforcement AKAR provides, and it is owned by the user: AKAR never edits `~/.claude/settings.json` automatically.

Verify the templates are valid:

```
akar hooks --check
```

Install the templates into `.akar/hooks/` (requires confirmation, backs up first):

```
akar hooks --install
```

You must register the hook in `~/.claude/settings.json` manually. If `akar` is not on the subprocess PATH Claude Code uses for hooks, the hook fails open (ALLOW + warning) — so confirm `akar` is resolvable from that PATH.

An older `hooks/pre-commit-akar.{sh,ps1}` design (git pre-commit running `akar doctor`) still exists in `hooks/` for reference, but the PreToolUse hook above is the proven integration.

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

Final response format for missions (advisory — AKAR does not execute the task):

```
ADVISORY ONLY — `akar mission` walks the state machine in scaffold mode. It does NOT:
  - execute code
  - edit files
  - call models
  - run the mission

done.

Mission:
- prompt: fix the login bug
- type: Bugfix
- risk: Low
- diff budget: 3 files, 60 LOC

Verified:
- scaffold mode (commands not executed)

Not verified:
- actual execution
```

The "changed: file.rs / verified: cargo build: ok" shape shown in older notes describes a future execution-capable mission, not current AKAR. Today AKAR reports strategy and records telemetry only; the human (or Claude Code, driven by the user) does the work.
