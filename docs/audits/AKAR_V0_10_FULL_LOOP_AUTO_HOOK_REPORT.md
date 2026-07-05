# AKAR v0.10.0 Full Loop Proof with Auto-Hook Active

**Status: FULL LOOP COMPLETED — baseline verdict PASS, auto-hook active throughout.**

Date: 2026-07-05
Session: Claude Code + AKAR v0.9.0 → v0.10.0
Auditor: post-session audit, evidence from real command output and HOOK_EVENTS.jsonl

---

## Purpose

This report documents the first complete AKAR baseline loop run with the
PreToolUse auto-hook active throughout the session. Every step was executed
with real commands and real output. Nothing was simulated.

---

## Step 1 — Clean git tree confirmed

```
git status --porcelain
```

Output: (empty — working tree clean)

HEAD: `eb38347784af` ("chore: gitignore HOOK_EVENTS archive files")

Note: The v0.9.0 work was committed in `17a3913` ("feat: AKAR v0.9.0 — first
auto-hook evidence") before the loop started. A follow-up commit `eb38347`
added `.akar/HOOK_EVENTS.*.jsonl` to .gitignore after the archive file was
found to dirty the tree. Both committed before snapshot.

---

## Step 2 — akar status: readiness READY

```
akar status
```

Output:
```
status: HEALTHY
  runtime:    akar 0.9.0
  ...
  baseline loop readiness:
  git repository detected: yes
  working tree clean:      yes
  baseline file present:   yes
  readiness:               READY
```

---

## Step 3 — akar hooks --check: PASS

```
akar hooks --check
```

Output:
```
hooks check:
  status: PASS
  templates found:
    - pre-tool-call.sh
    - pre-tool-call.ps1
```

---

## Step 4 — Old hook log archived

Moved `.akar/HOOK_EVENTS.jsonl` to `.akar/HOOK_EVENTS.before-v0.10.0.jsonl`
to keep the v0.10.0 session evidence clean.

---

## Step 5 — preflight --snapshot

```
akar preflight --snapshot "fix one small documentation typo"
```

Output:
```
preflight:
  prompt:       fix one small documentation typo
  task:         Bugfix
  risk:         Low
  autonomy:     A5
  diff_budget:  1-3 files, 5-60 LOC
  ...
snapshot: baseline written
  head:   eb38347784af
  task:   Bugfix
  budget: 3 files, 60 LOC
  file:   .akar/DIFF_BASELINE.json
  next:   run your Claude Code session, then 'akar postmortem --diff --baseline'
```

---

## Step 6 — Scoped change made

File changed: `docs/INSTALL.md` — version compatibility table.

Before:
```
| v0.2.x | v1 | Same layout, additive changes only |
```

After:
```
| v0.2.x | v1 | Same layout, additive changes only |
| v0.9.x | v1 | Hook templates added; additive only |
```

1 file changed, 1 line added, 0 deleted. Stale version table updated.
No source code touched. No runtime behavior changed.

---

## Step 7 — postmortem --diff --baseline

```
akar postmortem --diff --baseline
```

Output:
```
postmortem --diff --baseline:
  baseline timestamp: 2026-07-05T17:33:04Z
  baseline task:      Bugfix
  baseline head:      eb38347784af
  baseline budget:    3 files, 60 LOC

postmortem --diff:
  task:    Bugfix
  budget:  3 files, 60 LOC
  actual:  1 files, 1 added, 0 deleted (1 total changed LOC)
  status:  PASS
```

**Verdict: PASS**

1 file, 1 LOC against a Bugfix budget of 3 files / 60 LOC. Well within budget.

---

## Step 8 — HOOK_EVENTS.jsonl evidence

Full contents of `.akar/HOOK_EVENTS.jsonl` for this session (8 events):

```
1. cargo run -- preflight --snapshot "..."  → ALLOW  exit 0
2. cargo run -- preflight --snapshot "..."  → ALLOW  exit 0
3. akar preflight --snapshot "..."          → ALLOW  exit 0
4. git status --porcelain                   → ALLOW  exit 0
5. git status --porcelain                   → ALLOW  exit 0
6. git status --porcelain && git log ...    → ALLOW  exit 0
7. akar preflight --snapshot "..."          → ALLOW  exit 0
8. akar postmortem --diff --baseline        → ALLOW  exit 0
```

All 8 events: `hook: PreToolUse`, `tool_name: Bash`, `decision: ALLOW`,
`exit_code: 0`. No BLOCK events. No destructive commands attempted or executed.

---

## Step 9 — Verification results

```
cargo build --release    PASS (no recompile needed)
cargo test               251 passed, 0 failed
akar --version           akar 0.9.0 (installed; bumped to 0.10.0 after proof)
akar status              HEALTHY
akar doctor              OK
akar eval                28/28 PASS
akar hooks --check       PASS
```

---

## Honest conclusion

### What worked

- **Clean tree prerequisite**: All prior work committed before starting.
  `akar status` showed READY immediately.

- **Hook active throughout**: PreToolUse hook fired on every Bash tool call.
  8 events logged automatically to `.akar/HOOK_EVENTS.jsonl`. No manual
  invocation needed.

- **All commands ALLOW'd**: Every Bash tool call in the session was a safe
  dev command. Hook correctly allowed all of them with exit 0.

- **Snapshot**: `akar preflight --snapshot` accepted clean tree, wrote baseline
  at HEAD `eb38347784af`, budget 3 files / 60 LOC.

- **Scoped change**: One documentation line added to version compatibility
  table. Exactly the kind of minimal scoped change the loop is designed to
  measure.

- **Baseline postmortem**: PASS — 1 file, 1 LOC against budget 3 files / 60 LOC.
  AKAR measured accurately and reported honestly.

- **Tests**: 251 passed, 0 failed.

- **Evals**: 28/28 PASS.

### What remained manual

- Hook installation in `~/.claude/settings.json` — user action required
- Committing between releases to clean the tree — user action required
- Archiving old HOOK_EVENTS.jsonl before the session — done in this session

### Whether AKAR stayed advisory-only

Yes. AKAR classified commands, measured diffs, and logged events. It did not
execute, edit, enforce, revert, or block anything. The only writes were to
`.akar/` files (EVENT_LOG.jsonl, DIFF_BASELINE.json, HOOK_EVENTS.jsonl).

### Whether the full loop is proven

Yes. The complete sequence completed with real evidence:

```
commit (clean tree)
  → akar status READY
  → old hook log archived
  → akar preflight --snapshot (baseline written)
  → scoped change (1 file, 1 LOC)
  → hook fired automatically on every Bash call (8 ALLOW events)
  → akar postmortem --diff --baseline → PASS
```

This is the first session where the full AKAR loop ran end-to-end with the
PreToolUse safety hook active and logging automatically throughout.

---

## What v0.11.0 should prove

The same full loop but with a BLOCK event during the session — a destructive
command attempted and caught by the hook mid-session, while the loop still
completes successfully. That would prove the safety gate and advisory loop
work together in a real session with adversarial conditions.
