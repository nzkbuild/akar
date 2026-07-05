# AKAR v1 Architecture Freeze Proposal

Date: 2026-07-05
Status: PROPOSED — not yet approved

---

## What AKAR is

A local Rust CLI that acts as a discipline layer around Claude Code sessions.
It classifies intent, enforces diff budgets, detects skill conflicts, records
telemetry, and proposes learning patches — all locally, all read-only about
the user's project, all without modifying Claude Code configuration.

## What AKAR is not

- Not a code execution engine
- Not an AI model or model router
- Not a Claude Code plugin, extension, or hook installer
- Not a daemon or background process
- Not a cloud service or telemetry reporter
- Not a replacement for Claude Code
- Not a benchmark or evaluation framework for models
- Not a memory engine (it proposes patches; it does not apply them)

---

## Boundary definitions

### Installation boundary

AKAR installs as a single binary in PATH. It creates two directories on
first run:
- `<project>/.akar/` — project-local state (templates, logs, patches)
- `~/.claude/akar/` — global AKAR config only

AKAR does not install hooks, does not modify `.claude/settings.json`, does
not touch `.claude/commands/`, and does not modify any file in the user's
project.

### Global vs project state

| File | Scope | Writeable by AKAR |
|---|---|---|
| `~/.claude/akar/` | Global | Yes (bootstrap only) |
| `.akar/EVENT_LOG.jsonl` | Project | Yes (mission telemetry) |
| `.akar/LEARNING_PATCHES.md` | Project | Yes (learn command) |
| `.akar/SKILL_INVENTORY.md` | Project | Yes (skills command) |
| `.akar/NEXT_RUN.md` | Project | Yes (request command) |
| `.akar/*.md` templates | Project | Yes (bootstrap, idempotent) |
| `~/.claude/commands/` | Global | Never |
| `~/.claude/settings.json` | Global | Never |
| User project source files | Project | Never |

### Claude Code integration boundary

AKAR reads `~/.claude/commands/` to scan skills. It never writes there.
Claude Code can call AKAR via hooks — the user installs hooks manually.
AKAR provides hook scripts but does not install them.

AKAR does not depend on Claude Code being installed. It works standalone.

### Command vs passive runtime boundary

All AKAR commands are synchronous, invoked explicitly by the user or by a
Claude Code hook. There is no daemon. There is no filesystem watcher.
There is no background process. AKAR runs, prints, exits.

### Model/gateway boundary

AKAR detects the active model from environment variables (ANTHROPIC_MODEL,
CLAUDE_MODEL, OPENAI_MODEL) for display purposes only. It never calls any
model API. It never routes requests. It never proxies traffic.

### Skill authority boundary

AKAR scans and classifies skills. It reports conflicts. It recommends modes.
It does not disable, wrap, or modify any skill. Skill enforcement is the
user's responsibility.

### Execution model boundary

AKAR's mission state machine is scaffold-only until v1.0. The Execute and
MemoryUpdate states exist in the code but do nothing. This is intentional
and must not be changed before a v1 design review.

### Telemetry/privacy boundary

All telemetry is local. Nothing leaves the machine. The `redact()` function
in config.rs strips secret patterns before any log write. AKAR does not
collect usage analytics. `.akar/EVENT_LOG.jsonl` is gitignored by default.

### Rollback/recovery boundary

`akar doctor --fix` backs up files before overwriting them (backup.rs).
Backup files are named `<file>.bak.<epoch_seconds>`. AKAR does not delete
backups. The user is responsible for cleanup.

### Benchmark boundary

`akar eval` runs 28 behavioral checks. These are regression tests for AKAR
itself, not benchmarks for model quality or Claude Code performance.
The eval suite must not be positioned as a model evaluation harness.

---

## Candidates for deletion or simplification before v1 design

**Delete (dead code, never called from main.rs):**
- `model_profile::detect_drift()` and `SessionFingerprint` — useful concept,
  but requires session state that AKAR cannot observe
- `model_profile::calibrate_from_prompt()` — never called, would require API access
- `model_profile::detect_git_branch()` / `detect_git_root()` — git calls
  belong in a future git-integration module, not model_profile

**Simplify:**
- `circuit_breaker.rs` — correct implementation, but never used outside tests.
  Either wire it into mission.rs retry logic or delete it.
- `design.rs` — the DESIGN_DNA.md check is reasonable; the frontend file scanner
  is premature. Reduce to the DNA check only.
- `context_pack.rs` — builds a tier list but never reads file contents.
  Either read contents (v1 feature) or reduce to a path enumerator.
- `request_intelligence.rs` — the event-count thresholds (20/50) are arbitrary.
  Freeze the explicit used/limit path; deprecate the event-count inference.

**Do not expand before v1 design:**
- `mission.rs` Execute state — no code execution until v1 design review
- `learn.rs` patch application — no auto-apply until v1 design review
- `skill_registry.rs` enforcement — no skill disabling until v1 design review
- `model_profile.rs` persistence — no model profile storage until v1 design review

---

## Freeze list — must not change before v1 design

These are stable, well-tested, and load-bearing:

1. `event_log.rs` — append-only JSONL, json_escape, rotate_if_needed
2. `config.rs` — Config::discover(), redact(), home_dir()
3. `bootstrap.rs` — idempotent template copy
4. `backup.rs` — backup_file(), restore_backup()
5. `safe_fix.rs` — CreateMissingDir, CreateMissingTemplate
6. `contract.rs` — classify_prompt(), DiffBudget tiers
7. `safety.rs` — classify_command(), check_secrets(), govern_dependency()
8. `verify.rs` — detect_recipe(), run_recipe(), format_results()
9. The .gitignore policy for .akar/ runtime artifacts

---

## Next 3 releases

### v0.4.0 — Honest scaffold
**Must prove:** The user experience of running AKAR is not confusing.
- Add `akar run` output that clearly distinguishes "scaffold advice" from
  "execution complete"
- Fix LEARNING_PATCHES.md not being in .gitignore
- Remove or clearly label dead code (SessionFingerprint, calibrate_from_prompt)
- Fix skill conflict detection to scope to session-relevant skills only
  (not all 200 in ~/.claude/)
- Do not add new commands

**Must not build:** any execution engine, hook installer, or model API call.

### v0.5.0 — Real hook integration
**Must prove:** Claude Code can call AKAR automatically via a hook, and the
output is useful.
- Ship a working pre-tool-call hook script (bash + PowerShell)
- `akar hooks --install` copies scripts to .git/hooks/ with user confirmation
- The hook calls `akar safety "$COMMAND"` before dangerous commands
- Verify the hook fires in a real Claude Code session

**Must not build:** any execution in the mission state machine.

### v0.6.0 — Real diff budget enforcement
**Must prove:** AKAR can measure whether a diff exceeds its budget.
- After a Claude Code session, `akar postmortem --diff` reads the actual
  git diff and compares it to the preflight budget
- Reports: budget was N files / M LOC, actual was X files / Y LOC
- Writes a learning patch if budget was exceeded

**Must not build:** any code execution, file editing, or model API calls.

---

## What must not be built before v1 design review

- Model API calls of any kind
- Automatic skill disabling or enabling
- Automatic memory patch application
- Daemon or background process
- Cloud telemetry
- SQLite or any database dependency
- Vector DB or embedding engine
- GUI or web UI
- Any modification to ~/.claude/settings.json without explicit user confirmation
