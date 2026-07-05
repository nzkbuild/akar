# AKAR v0.8.0 First Verified Baseline Loop Report

**Status: FULL BASELINE LOOP COMPLETED — verdict PASS.**

Date: 2026-07-05
Session: Claude Code + AKAR v0.7.1 → v0.8.0
Auditor: post-session audit, evidence from real command output

---

## Purpose

This report documents the first successful end-to-end AKAR baseline loop run on
the AKAR project itself. Every step was executed with real commands and real
output. Nothing was simulated.

---

## Step 1 — git status before snapshot

Command:

```
git status --short
```

Output:

```
(empty — working tree clean)
```

Finding: Working tree was clean. All v0.7.1 work had been committed in commit
`034aa17b8a72` ("feat: AKAR v0.7.1 — full loop readiness") before the loop started.

---

## Step 2 — akar status: readiness READY

Command:

```
akar status
```

Output:

```
status: HEALTHY
  runtime:    akar 0.7.1
  project:    akar
  doctor:     OK
  bootstrap:  OK
  telemetry:  779 event(s)
  postmortem: clean
  skills:     OK
  request:    NORMAL
  ram_budget: <150 MB target

  baseline loop readiness:
  git repository detected: yes
  working tree clean:      yes
  baseline file present:   no
  readiness:               READY
```

Finding: Readiness was READY. Git repo detected, working tree clean, no stale
baseline. All prerequisites met.

---

## Step 3 — preflight --snapshot

Command:

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
  request_mode: NORMAL
  skills:       zero-skill mode (AKAR kernel only)
  verification:
    - run: cargo build
    - run: cargo test
  stop_conditions:
    - original symptom no longer reproducible
    - tests pass
  recommendation: Proceed — low risk task. Stay within diff budget

snapshot: baseline written
  head:   034aa17b8a72
  task:   Bugfix
  budget: 3 files, 60 LOC
  file:   .akar/DIFF_BASELINE.json
  next:   run your Claude Code session, then 'akar postmortem --diff --baseline'
```

Finding: Baseline written. HEAD `034aa17b8a72`, budget 3 files / 60 LOC.

---

## Step 4 — scoped change made

File changed: `docs/INSTALL.md` line 45.

Before:
```
Expected: `akar 0.3.0`
```

After:
```
Expected: `akar 0.7.1`
```

Finding: One file changed. One line added, one line deleted (2 total LOC).
Stale version number corrected. No source code touched. No runtime behavior
changed.

---

## Step 5 — safety hook invocation

Command:

```
akar safety "cargo test"
```

Output:

```
safety assessment:
  command: cargo test
  risk:    Safe
  status:  allowed
  reason:  standard build/test command
```

Finding: Safety gate works. `cargo test` is classified Safe and allowed.
Hook was invoked manually — not auto-installed in Claude Code settings.json.

---

## Step 6 — postmortem --diff --baseline

Command:

```
akar postmortem --diff --baseline
```

Output:

```
postmortem: 779 total event(s), 20 mission(s) in recent log
  outcome: clean
  latest:  mission/done task=Bugfix risk=Low autonomy=A5 warnings=0 prompt=fix the login button
  follow-up:
    - no action needed — last mission completed cleanly

postmortem --diff --baseline:
  baseline timestamp: 2026-07-05T12:37:41Z
  baseline task:      Bugfix
  baseline head:      034aa17b8a72
  baseline budget:    3 files, 60 LOC

postmortem --diff:
  task:    Bugfix
  budget:  3 files, 60 LOC
  actual:  1 files, 1 added, 1 deleted (2 total changed LOC)
  status:  PASS
```

**Verdict: PASS**

1 file changed, 2 LOC, against a Bugfix budget of 3 files / 60 LOC. Well within
budget. No learning patch written (none needed for PASS).

---

## Step 7 — verification results

```
cargo build --release    PASS (akar v0.7.1, no recompile needed)
cargo test               PASS — 235 passed, 0 failed
akar --version           akar 0.7.1
akar status              HEALTHY
akar doctor              OK
akar eval                28/28 PASS
```

---

## Honest conclusion

### What worked

- **Clean tree prerequisite**: All prior work was committed before starting.
  `akar status` showed `READY` immediately.

- **Snapshot**: `akar preflight --snapshot` accepted the clean tree, wrote
  `.akar/DIFF_BASELINE.json` at HEAD `034aa17b8a72`, budget 3 files / 60 LOC.

- **Scoped change**: One documentation file, one stale version string corrected.
  Exactly the kind of minimal change the loop is designed to measure.

- **Safety gate**: `akar safety "cargo test"` → Safe, allowed. Classifier
  works correctly for normal dev commands.

- **Baseline postmortem**: `akar postmortem --diff --baseline` read the saved
  baseline, measured the actual diff (1 file, 2 LOC), compared against budget
  (3 files, 60 LOC), and reported PASS. No enforcement, no revert, advisory only.

- **Tests**: 235 passed, 0 failed. All modules intact.

- **Evals**: 28/28 PASS. No behavioral regressions.

### What remained manual

- Hook installation: `akar safety` was invoked manually, not via a Claude Code
  pre-tool-call hook. The hook template exists and would fire if installed.

- Committing the session change: the documentation fix itself is uncommitted at
  the time of this report. That is the next step.

### Whether AKAR stayed advisory-only

Yes. AKAR did not execute code, edit source files, enforce budgets, revert
changes, install hooks, or modify Claude Code configuration. Every command read
state and printed output. The only writes were to `.akar/` files
(EVENT_LOG.jsonl, DIFF_BASELINE.json).

### Whether the baseline loop is proven

Yes. The full sequence completed with real evidence:

```
commit (clean tree) → akar status READY → akar preflight --snapshot
→ scoped change → akar safety (manual) → akar postmortem --diff --baseline → PASS
```

This is the first session where AKAR measured its own work against a pre-session
baseline and reported an honest PASS.

---

## What v0.9.0 should prove

The same loop with a Claude Code hook auto-firing `akar safety` on every tool
call — so the safety gate fires automatically, not just when invoked manually.
That would close the last remaining manual step in the loop.
