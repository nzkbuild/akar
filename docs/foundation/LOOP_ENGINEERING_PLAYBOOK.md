# Loop Engineering Playbook

The proven AKAR session loop. Follow these phases in order. Do not skip phases.

## Phase 1 — Status Readiness

Before any work begins, verify the runtime is ready.

```
akar status
```

Check:
- `readiness: READY` — working tree is clean, baseline file is present
- `readiness: BLOCKED` — working tree is dirty or baseline is missing

If BLOCKED:
- Inspect with `git status` and `git diff`
- Commit completed prior work explicitly
- Do not proceed to preflight until readiness is READY

## Phase 2 — Preflight Snapshot

Before starting session work, take a diff baseline snapshot.

```
akar preflight --snapshot "task description"
```

This writes `.akar/DIFF_BASELINE.json` with:
- Current HEAD commit hash
- Task type and budget (files max, LOC max)
- Timestamp

Requirements:
- Working tree must be clean before snapshot
- If dirty, commit prior work first (Phase 1)
- Never snapshot on a dirty tree

## Phase 3 — Auto-Hook Safety

Confirm the PreToolUse hook is active before any tool-calling session.

```
akar hooks --check
```

Check:
- Status: PASS — templates valid, hook can fire
- Status: FAIL — templates missing or invalid

If FAIL:
- Do not proceed with a tool-calling session
- Follow CLAUDE_CODE_HOOK_PLAYBOOK.md to restore hook
- Verify HOOK_EVENTS.jsonl has entries after hook is restored

## Phase 4 — Scoped Work

Execute the task within the established budget.

During work:
- Stay within the task type budget (files and LOC)
- Run `akar safety "command"` before any uncertain shell command
- If a command is blocked: stop, read the reason, check playbook for safe alternative
- Do not retry a blocked command

Budget reference:
- Bugfix: 3 files, 60 LOC
- Feature: 8 files, 200 LOC
- Refactor: 6 files, 150 LOC
- Docs: 5 files, 120 LOC
- Test: 6 files, 150 LOC

If scope grows beyond budget: stop, split the task, start a new loop.

## Phase 5 — Postmortem Baseline Diff

After work is complete, measure what changed against the baseline.

```
akar postmortem --diff --baseline
```

This compares the current working tree against the snapshot from Phase 2.

Results:
- WITHIN budget: proceed to verification
- EXCEEDED budget: stop, read the exceeded reason, split task or reduce scope

If EXCEEDED:
- Do not commit over-budget work
- Split the task into smaller units
- Each unit gets its own preflight snapshot and loop

## Phase 6 — Verification

Run the project verification suite.

```
cargo build --release
cargo test
```

Or the equivalent for the project language.

Requirements:
- All tests must pass
- Build must succeed
- No new warnings introduced

If verification fails:
- Fix the failure before proceeding
- Do not skip or bypass tests
- Do not use --no-verify flags

## Phase 7 — Audit Report

Review the evidence before committing.

Check:
- `.akar/HOOK_EVENTS.jsonl` — any unexpected blocks?
- `.akar/EVENT_LOG.jsonl` — telemetry complete?
- `akar postmortem` — outcome clean?
- `git diff --stat` — only expected files changed?

If evidence is incomplete or unexpected:
- Investigate before committing
- Do not commit with unresolved anomalies

## Phase 8 — Commit Checkpoint

Commit only the work from this loop.

```
git status
git diff --stat
git add src/specific_file.rs
git add docs/specific_doc.md
git commit -m "feat: description matching task"
```

Rules:
- Stage only files changed by this task
- Do not broad-stage with git add .
- Use a commit message that matches the task type prefix
- Do not push (push is a separate explicit decision)

After commit: run `git status` to confirm clean tree. Loop complete.

## Anti-Loop Rules

These rules prevent the loop from getting stuck or expanding dangerously:

### Do Not Retry a Blocked Command
If a command is blocked by the hook, it is blocked for a reason. Read the reason. Check the playbook. Find a safe alternative. Do not run the same blocked command again.

### Do Not Continue After Missing Hook Evidence
If the hook was expected to fire but HOOK_EVENTS.jsonl has no entry, the hook may be broken. Stop and restore the hook before continuing.

### Do Not Snapshot on a Dirty Tree
A baseline snapshot on a dirty tree will produce incorrect diff measurements. Always commit prior work before snapshotting.

### Do Not Broaden Task After Budget Exceeded
If the postmortem reports EXCEEDED, the task scope grew beyond budget. Stop the current loop. Split the work. Start a new loop with a new preflight snapshot.

### Use Stored Evidence Before Next Action
Before running the next command, check what the previous command produced. Hook events, telemetry, and postmortem data are stored locally. Read evidence before acting.

## Loop Summary

```
akar status          -> readiness: READY?
akar preflight --snapshot "task"
akar hooks --check   -> PASS?
[do the work]
akar postmortem --diff --baseline  -> WITHIN budget?
cargo build --release && cargo test
[review HOOK_EVENTS.jsonl]
git add <files> && git commit -m "..."
git status           -> clean tree
```
