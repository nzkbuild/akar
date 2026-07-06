# AKAR Foundation Principles

AKAR v0.12.0+ carries local foundation knowledge to guide safe defaults and alternatives.

## First Principles

AKAR operates under these core principles:

### 1. Local-First
- All knowledge, playbooks, and evidence stored locally
- No hidden remote dependencies
- Human audit remains accessible

### 2. Advisory Before Execution
- AKAR classifies risk and provides guidance
- AKAR does not auto-execute destructive commands
- Human or Claude Code makes final execution decision

### 3. Evidence Before Confidence
- Hook events logged to `.akar/HOOK_EVENTS.jsonl`
- Telemetry recorded to `.akar/EVENT_LOG.jsonl`
- Baseline snapshots written to `.akar/DIFF_BASELINE.json`
- Verification uses actual command output

### 4. Scoped Loop Before Broad Work
- Preflight snapshot establishes baseline
- Work scoped to defined task budget
- Postmortem compares actual vs budget
- Exceeded budget triggers learning patch

### 5. Safe Alternative Before Retry
- Blocked command triggers playbook guidance
- Foundation playbooks suggest safe alternatives
- No silent retry of blocked actions

### 6. No Silent Destructive Cleanup
- `git reset`, `git clean`, `git stash` are forbidden by default
- `rm -rf /` and filesystem wipes always blocked
- Safe alternatives: inspect status, commit completed work, start fresh baseline

### 7. No Hidden Global Config Mutation
- AKAR does not edit `~/.claude/settings.json` automatically
- Hook installation requires explicit user confirmation
- Template files copied to project `.akar/hooks/` only

### 8. Human Audit Remains Final Authority
- AKAR provides classification and guidance
- Hook exit codes enforce blocks
- Human or Claude Code reviews evidence before proceeding
- Learning patches marked `proposed` until human approval

## Foundation Knowledge Layer

AKAR v0.12.0 includes:
- `AKAR_FOUNDATION.md` (this file) — first principles
- `SAFE_GIT_PLAYBOOK.md` — safe git patterns
- `SAFE_SHELL_PLAYBOOK.md` — blocked commands and safe alternatives
- `CLAUDE_CODE_HOOK_PLAYBOOK.md` — hook integration guidance
- `LOOP_ENGINEERING_PLAYBOOK.md` — proven AKAR workflow loop

These playbooks are embedded in `src/foundation.rs` as static guidance functions.

## Integration

Foundation guidance appears in:
- `akar safety "rm -rf /"` — BLOCKED output includes safe alternative
- `akar status` — BLOCKED readiness includes git dirty playbook
- `akar postmortem --diff` — EXCEEDED output includes budget playbook
- `akar hooks --check` — failure includes hook broken playbook

## Non-Goals

AKAR foundation does NOT:
- Auto-execute playbook actions
- Auto-commit changes
- Auto-push to remote
- Modify user project files without explicit command
- Change Claude Code configuration
- Override user instructions
- Make decisions on behalf of human operator
