# AKAR v0.40.0 — Multi-Task Session Dogfood Audit Report

## 1. Executive Verdict

**PASS.** A controlled three-task sequential session on a single Node fixture
proves AKAR handles baseline snapshots, NEXT_RUN task threading, governor
transitions, postmortem diff budgets, and clean checkpointing across multiple
tasks without stale state leakage.

## 2. AKAR Baseline

- **Version:** 0.39.0
- **Commit:** 22a495e
- **Tests:** 508 passed, 0 failed
- **Evals:** 28/28 PASS
- **Working tree:** clean

## 3. Why This Trial Matters

Every previous dogfood trial was single-task: one preflight → one fix → one
postmortem → done. Real sessions have multiple sequential tasks (fix a bug,
add a feature, update docs). This trial tests whether AKAR's state machine
handles the full cycle correctly across task boundaries: baseline refresh,
NEXT_RUN task text updates, governor transitions (READY → RUN_POSTMORTEM →
commit → clean → READY for next task), and postmortem measurement against the
correct baseline each time.

## 4. Fixture Description

- **Path:** `../akar-dogfood-v040-multitask-node-fixture`
- **Project kind:** Node (package.json)
- **Files:** `package.json`, `src/calc.js`, `test/calc.test.js`, `README.md`
- **Test command:** `npm test` → `node --test test/*.test.js`
- **Initial state:** add and subtract working, multiply bug (returns a+b), 2 pass/1 fail
- **Node version:** v24.11.0

## 5. Setup Path Result

- `akar init` → clean (`.akar/` created, "templates directory not found" warning)
- Tree dirty from `.akar/` → added `.gitignore`, committed
- `akar hooks --install` → installed to `.akar/hooks/` (2 templates)
- `akar hooks --check` → PASS (source: project .akar/hooks)
- `akar doctor` → WARN (expected: no NEXT_RUN, no baseline), project kind: Node (PASS), hints: `npm test` (High)
- `akar verify` → manual-only for Node, includes discovery hint
- Clean tree at setup end

## 6. `.akar/` Dirty-Tree Handling Result

Standard pattern: `akar init` creates `.akar/` → tree dirty → added `.gitignore`
with `.akar/` entry → committed → tree clean. Same friction as all previous
external dogfood trials. No new issue.

## 7. Hook Install/Check Result

- `akar hooks --check` (embedded) → PASS
- `akar hooks --install` → copied 2 templates to `.akar/hooks/`
- `akar hooks --check` (project) → PASS
- No Claude settings modified — hooks stay manual per AKAR design
- No live hook events generated (expected — hooks not wired to Claude Code)

## 8. Doctor/Status/Verify Baseline Result

**Doctor:**
- project kind: PASS "Node — NEXT_RUN uses project-appropriate commands"
- verification hints: PASS "npm test (High, package.json)"
- NEXT_RUN: WARN (missing, expected before first request)
- DIFF_BASELINE: WARN (missing, expected before first preflight)

**Verify:** "no automated verify for Node projects — discovered verification hint(s): npm test (High, package.json)"

## 9. Task 1 Objective and Loop Result

**Task:** "fix the multiply bug in the multi-task Node fixture"
**Type:** Bugfix, 3 files/60 LOC

**Preflight:** baseline written, head `7eb8ff8`
**Request:** mode=NORMAL, governor READY, NEXT_RUN written
**request --check:** PASS (all 4 checks)
**NEXT_RUN quality:**
- Current State: project kind Node, requested task present
- Objective: "Continue the scoped task. - Task: fix the multiply bug..."
- Allowed Commands: `npm test` present, zero Cargo leakage
- Verification Required: `npm test` present

**Fix:** Changed `multiply` from `a + b` to `a * b` (1 file, 1 line)

## 10. Task 1 Postmortem Result

```
postmortem --diff:
  baseline head: 7eb8ff8d47eb
  task: Bugfix, budget: 3 files, 60 LOC
  actual: 1 files, 1 added, 1 deleted (2 total changed LOC)
  status: PASS
```

Commits: fix multiply bug (`f58d9b0`)

## 11. Task 1 Checkpoint/Clean-State Result

- Post-fix: tree dirty, governor → RUN_POSTMORTEM (correct)
- `learn --list`: 0 patches (correct — nothing to learn from a one-line fix)
- `git commit`: clean tree confirmed
- Governor after clean: would show READY on next cycle

## 12. Task 2 Objective and Loop Result

**Task:** "add a square function to the multi-task Node fixture"
**Type:** Bugfix, 3 files/60 LOC

**Preflight:** baseline written, head `f58d9b0` (Task 1's new HEAD — **fresh baseline confirmed**)
**request --check:** PASS
**NEXT_RUN quality:**
- Current State: requested task = "add a square function..." (**not Task 1's text**)
- Objective: "- Task: add a square function to the multi-task Node fixture" (**correct, not stale**)

**Fix:** Added `square(a)` to `src/calc.js`, added test `square(4) == 16`, updated exports (2 files)

## 13. Task 2 Postmortem Result

```
postmortem --diff:
  baseline head: f58d9b027c90
  task: Bugfix, budget: 3 files, 60 LOC
  actual: 2 files, 10 added, 2 deleted (12 total changed LOC)
  status: PASS
```

Commits: add square function (`c1c4341`)

## 14. Task 2 Checkpoint/Clean-State Result

- Tests: 4/4 pass (3 original + 1 new `square` test)
- `learn --list`: 0 patches
- `git commit`: clean tree confirmed
- No state contamination from Task 1

## 15. Task 3 Objective and Loop Result

**Task:** "document the available calculator functions in the multi-task Node fixture"
**Type:** Bugfix, 3 files/60 LOC

**Preflight:** baseline written, head `c1c4341` (Task 2's HEAD — **second fresh baseline confirmed**)
**request --check:** PASS
**NEXT_RUN quality:**
- Current State: requested task = "document the available calculator functions..." (**correct**)
- Objective: "- Task: document the available calculator functions..." (**not Task 1 or Task 2's text**)

**Fix:** Updated README.md with list of four functions (1 file, 7 lines)

## 16. Task 3 Postmortem Result

```
postmortem --diff:
  baseline head: c1c4341ef12e
  task: Bugfix, budget: 3 files, 60 LOC
  actual: 1 files, 7 added, 0 deleted (7 total changed LOC)
  status: PASS
```

## 17. Task 3 Checkpoint/Clean-State Result

- Tests: 4/4 pass (doc change didn't affect code)
- `learn --list`: 0 patches
- `git commit`: clean tree confirmed
- Final state: 3 tasks, 3 commits, clean tree

## 18. Baseline Refresh Behavior

| Task | Baseline HEAD | Postmortem HEAD | Match |
|------|--------------|-----------------|-------|
| 1    | `7eb8ff8`    | `7eb8ff8`       | Yes |
| 2    | `f58d9b0`    | `f58d9b0`       | Yes |
| 3    | `c1c4341`    | `c1c4341`       | Yes |

Each `preflight --snapshot` correctly captured the current HEAD as the baseline.
Each `postmortem --diff --baseline` correctly diffed against that task's baseline,
not against the initial fixture baseline or a previous task's baseline.

## 19. NEXT_RUN Stale-Task Check

No stale task text detected:
- Task 1: "fix the multiply bug..." in both Current State and Objective
- Task 2: "add a square function..." — Task 1 text absent
- Task 3: "document the available calculator functions..." — Task 1 and Task 2 text absent

NEXT_RUN task threading correctly updates across task boundaries.

## 20. Governor Transition Behavior

Governor decisions followed the expected pattern for all three tasks:

1. **Pre-task (clean tree, baseline present):** READY
2. **Post-fix (dirty tree, baseline present):** RUN_POSTMORTEM
3. **Post-commit (clean tree):** Would return to READY on next preflight+request cycle

No transition was skipped or misrouted. The "dirty + baseline → RUN_POSTMORTEM"
rule and "clean + baseline → READY" rule held across all task boundaries.

## 21. Learn State Behavior

`akar learn --list` returned 0 patches after every task. Correct — the fixes
were deliberate and completed within budget. AKAR did not incorrectly flag
any task as a learning opportunity.

## 22. Hook Evidence Result

No live hook events were generated — Claude Code's PreToolUse hook was not
wired in this fixture. Hook integration was proven in v0.35/v0.36. This trial
focused on multi-task state management.

- HOOK_EVENTS.jsonl: absent (expected)
- Doctor correctly reports absence without error

## 23. What AKAR Helped With

- **Baseline management:** Each task got its own baseline automatically via
  `preflight --snapshot`. The user didn't need to track HEAD manually.
- **Task threading:** NEXT_RUN always showed the correct current task, never
  stale text from a previous task.
- **Budget discipline:** All three tasks stayed within budget, and postmortem
  correctly measured each against its task-specific baseline.
- **Clean transitions:** Git commits between tasks allowed the governor to
  transition correctly (READY → fix → RUN_POSTMORTEM → commit → clean → READY).
- **No state contamination:** Task 2's postmortem didn't include Task 1's
  changes. Task 3's postmortem didn't include Task 2's changes.

## 24. What AKAR Made Worse

- **"templates directory not found" on init:** Same misleading warning as before.
  The embedded fallback works fine.
- **Doctor WARNs on fresh project:** NEXT_RUN and baseline are expected to be
  missing on a fresh project, but doctor uses WARN severity. Could alarm new users
  who think something is actually wrong.
- **"Bugfix" classification for documentation:** Task 3 (README update) was
  classified as Bugfix rather than something like Docs. Not incorrect — the budget
  was appropriate — but less semantically precise than it could be.

## 25. Confusing or Misleading Output

- "templates directory not found" — same as v0.39.0 observation
- Doctor lists "NEXT_RUN.md present — missing — run 'akar request'" as a WARN
  advisory even though it's expected before first use
- Governor says "Run akar postmortem --diff --baseline before continuing" even
  after postmortem has already run — this is correct (governor only reads state,
  doesn't know postmortem was just run unless the tree is committed), but could
  confuse someone who just ran postmortem

## 26. Manual Rescue Required

None. All three tasks followed the standard pattern without error.

## 27. Multi-Task Alpha Verdict

**PASS.** The AKAR advisory loop handles sequential tasks correctly:
- Baselines refresh on each preflight
- NEXT_RUN task text updates without staleness
- Governor transitions are coherent across task boundaries
- Postmortem measures against the correct baseline each time
- Git commits enable clean progression between tasks
- Learn state doesn't accumulate false positives

## 28. Stable Alpha Status After This Trial

Five lanes now dogfood-proven:

| Project Kind | Sessions | Verdict |
|-------------|----------|---------|
| Rust | Single | PASS |
| Node | Single | PASS |
| Python | Single | PASS |
| Unknown | Single | PASS |
| Node | Multi-task | PASS |

The multi-task proof was collected on Node, but the state machinery (baseline,
NEXT_RUN, governor, postmortem) is project-kind-agnostic. A multi-task Python
or Rust session would use the same machinery.

## 29. Required Fixes Before v1.0.0

1. **Dirty-tree recovery guidance for init** — `.akar/` dirties the tree on
   first init; needs a doctor hint or docs section.
2. **Cross-platform hook validation** — macOS and Linux hook templates need
   independent verification.
3. **Hook install automation decision** — decide whether `akar hooks --install`
   should eventually write to `~/.claude/settings.json`.
4. **"templates directory not found" wording** — misleading when embedded
   fallback works.
5. **Fresh-project doctor severity** — NEXT_RUN and baseline WARNs on a fresh
   project could be downgraded to INFO or reworded as "expected on new project."

## 30. Honest Conclusion

Three tasks, three baselines, three postmortems, zero stale state. The advisory
loop machinery handles sequential sessions without modification. Task threading
stays current, baselines refresh correctly, and budget discipline holds across
task boundaries. The same friction points from previous trials (dirty-tree-on-init,
"templates not found" warning) persist but are cosmetic, not functional.

A multi-task session is not more complex for AKAR than a single-task session —
the state machine just cycles three times instead of once. This is exactly the
design intent, and the trial proves it works.

## 31. Next Recommended Release

**v0.41.0 — Fresh-User Wording Polish.** Address the three cosmetic issues that
have persisted across multiple dogfood trials: downgrade fresh-project doctor
severity, fix the "templates not found" init warning, and add a doctor hint
for the `.akar/` dirty-tree-on-init friction. No new features, no behavior
changes — just honest wording improvements.
