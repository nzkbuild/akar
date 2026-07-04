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

## Normal workflow

```
akar bootstrap        # one-time project setup
akar doctor           # confirm health
akar preflight "task" # review strategy before acting
akar run "task"       # full workflow: preflight + mission + telemetry
akar postmortem       # review latest outcome
akar learn            # propose learning patch if degraded/failed
```

## Expected output

Every command produces short structured output.
Final response format: status line, created/skipped/warnings, verified/not verified.
No essays. No fake certainty.

## Scaffold mode

In v0.2.x, AKAR does not edit user code.
Mission is scaffold mode: classifies, plans, records, but does not execute.
Real code execution engine is planned for v0.3+.

## Claude Code integration (optional)

Copy `.claude/commands/akar-*.md` to your project's `.claude/commands/`.
Use `/akar-preflight`, `/akar-mission`, `/akar-doctor` as slash commands inside Claude Code.
Hooks are in `hooks/` — install manually to `.git/hooks/` if desired.
