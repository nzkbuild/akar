# AKAR v0.47.0 — Prepare/Finish Cross-Lane Dogfood

## 1. Executive Verdict

**2 AKAR commands per task holds across all five project lanes.** Rust, Node, Python,
Unknown-with-Makefile, and Unknown-no-hint fixtures all completed the full advisory loop
with exactly 2 AKAR commands per task (`prepare` and `finish`). Project-kind guidance
was correct in every lane — Rust got `cargo build && cargo test`, Node got `npm test`,
Python got `python -m pytest`, Unknown Makefile got `make test (discovered; run manually)`,
Unknown no-hint got `(no verification command discovered)`. No Cargo/npm/pytest/make
commands were invented or leaked between lanes. No project code was executed by AKAR.
No git mutations were performed by AKAR. No Claude settings were modified.

**The prepare/finish pattern is lane-agnostic and stable.** Two commands per task
is consistently achievable across all currently supported AKAR project types.

## 2. Baseline

| Check | Result |
|---|---|
| Commit | `8ad7b14` — feat: add AKAR prepare and finish commands |
| Version | `akar 0.46.0` |
| Working tree | clean |
| `cargo test` | 534 passed, 0 failed |
| `cargo run -- --version` | `akar 0.46.0` |
| `cargo run -- doctor` | WARN (split-rule learning patch, known) |
| `cargo run -- status` | HEALTHY, SPLIT_TASK (known) |
| `cargo run -- request "prepare finish cross-lane baseline check"` | NORMAL mode |
| `cargo run -- request --check` | PASS |
| `cargo run -- governor --no-exit-code` | SPLIT_TASK (known) |
| `cargo run -- learn --list` | 8 entries (1 active, 7 resolved) |
| `cargo run -- hooks --check` | PASS |
| `cargo run -- eval` | 28/28 PASS |

## 3. Why This Trial Matters

v0.46.0 introduced `akar prepare` and `akar finish` to consolidate the manual
advisory loop from 8 AKAR commands per task to 2. The v0.46 Node dogfood proved
the pattern works on one lane. But the consolidation relies on project-kind detection,
verification guidance, and NEXT_RUN compilation paths that differ per project type.

This trial proves:
1. Project-kind detection is correct across all five lanes.
2. Verification guidance stays lane-correct and never cross-contaminates.
3. NEXT_RUN compilation adapts correctly per project kind.
4. Safety boundaries hold regardless of project type.
5. The 2-command target is achievable on every lane.

## 4. Dogfood Method

Five external fixtures were created outside the AKAR repo. Each fixture:
- Had an initial bug or mismatch (multiply returning wrong result).
- Was initialized with git, committed, and AKAR-init'd.
- `.akar/` was handled with `.gitignore` (never auto-edited by AKAR).
- Ran exactly `akar prepare "<task>"` → manual fix → manual verify → `akar finish`.
- Safety boundaries were recorded per fixture (project tests, git commands, settings edits).
- Command counts were recorded: AKAR commands, git commands, project verification commands, edit/inspection commands.

## 5. Fixture Matrix

| # | Lane | Path | Marker | Initial state | Fix | Verify |
|---|---|---|---|---|---|---|
| A | Rust | `akar-dogfood-v047-prepare-finish-rust-fixture` | Cargo.toml | multiply returns a+b (1/4 fail) | a*b | cargo test |
| B | Node | `akar-dogfood-v047-prepare-finish-node-fixture` | package.json | multiply returns a+b (1/4 fail) | a*b | npm test |
| C | Python | `akar-dogfood-v047-prepare-finish-python-fixture` | pyproject.toml | multiply returns a+b (1/4 fail) | a*b | python -m pytest |
| D | Unknown Makefile | `akar-dogfood-v047-prepare-finish-unknown-makefile-fixture` | Makefile (test target) | calc.txt mismatches expected.txt (6 vs 9) | fix to 9 | make test (not available; manual compare) |
| E | Unknown no-hint | `akar-dogfood-v047-prepare-finish-unknown-nohint-fixture` | None | calc.txt mismatches expected.txt (7 vs 8) | fix to 8 | manual compare |

## 6. Rust Fixture Result

**Fixture:** `akar-dogfood-v047-prepare-finish-rust-fixture`

| Step | Action | Result |
|---|---|---|
| Initial test | `cargo test` | 3 pass, 1 fail (multiply) |
| Init | `akar init` | OK (1 warn: templates dir absent) |
| .gitignore | Added .akar/, Cargo.lock committed | Clean tree |
| Prepare | `akar prepare "fix the multiply bug..."` | Rust, carryo build && cargo test, READY, PASS |
| Fix | `a+b` → `a*b` in lib.rs | 1 line changed |
| Verify | `cargo test` | 4/4 PASS |
| Finish | `akar finish` | PASS, 1 file/2 LOC within 3/60 budget |
| Commit | `git add src/lib.rs && git commit` | Clean tree |

**AKAR commands:** 2 (prepare, finish)
**Git commands:** 5 (init, add ×2, commit ×2 for init + .gitignore)
**Project commands:** 2 (cargo test ×2)
**Total commands:** 9

## 7. Node Fixture Result

**Fixture:** `akar-dogfood-v047-prepare-finish-node-fixture`

| Step | Action | Result |
|---|---|---|
| Initial test | `npm test` | 3 pass, 1 fail (multiply) |
| Init | `akar init` | OK |
| .gitignore | Added .akar/ | Clean tree |
| Prepare | `akar prepare "fix the multiply bug..."` | Node, npm test (run manually), READY, PASS |
| Fix | `a+b` → `a*b` in calc.js | 1 line changed |
| Verify | `npm test` | 4/4 PASS |
| Finish | `akar finish` | PASS, 1 file/2 LOC within 3/60 budget |
| Commit | `git add src/calc.js && git commit` | Clean tree |

**AKAR commands:** 2 (prepare, finish)
**Git commands:** 5 (init, add ×2, commit ×2)
**Project commands:** 2 (npm test ×2)
**Total commands:** 9

## 8. Python Fixture Result

**Fixture:** `akar-dogfood-v047-prepare-finish-python-fixture`

| Step | Action | Result |
|---|---|---|
| Initial test | `python -m pytest` | 3 pass, 1 fail (multiply) |
| Init | `akar init` | OK |
| .gitignore | Added .akar/ + __pycache__/ | Clean tree |
| Prepare | `akar prepare "fix the multiply bug..."` | Python, python -m pytest (run manually), READY, PASS |
| Fix | `a+b` → `a*b` in calc.py | 1 line changed |
| Verify | `python -m pytest` | 4/4 PASS |
| Finish | `akar finish` | PASS, 1 file/2 LOC within 3/60 budget |
| Commit | `git add calc.py && git commit` | Clean tree |

**AKAR commands:** 2 (prepare, finish)
**Git commands:** 5 (init, add ×2, commit ×2)
**Project commands:** 2 (pytest ×2)
**Total commands:** 9

## 9. Unknown Makefile Fixture Result

**Fixture:** `akar-dogfood-v047-prepare-finish-unknown-makefile-fixture`

| Step | Action | Result |
|---|---|---|
| Init | `akar init` | OK |
| .gitignore | Added .akar/ | Clean tree |
| Prepare | `akar prepare "fix the mismatch..."` | Unknown, make test (discovered; run manually), READY, PASS |
| Fix | `multiply(3,3)=6` → `multiply(3,3)=9` in calc.txt | 1 line changed |
| Verify | Manual compare calc.txt vs expected.txt | PASS (make not available on this machine) |
| Finish | `akar finish` | PASS, 1 file/2 LOC within 3/60 budget |
| Commit | `git add calc.txt && git commit` | Clean tree |

**AKAR commands:** 2 (prepare, finish)
**Git commands:** 5 (init, add ×2, commit ×2)
**Project commands:** 0 (make not available; manual verify)
**Total commands:** 7

Note: make was not available on this Windows machine. The verification discovery hint
correctly identified `make test` from the Makefile, but the user performed manual
comparison instead. This is expected behavior — discovered commands are advisory only.

## 10. Unknown No-Hint Fixture Result

**Fixture:** `akar-dogfood-v047-prepare-finish-unknown-nohint-fixture`

| Step | Action | Result |
|---|---|---|
| Init | `akar init` | OK |
| .gitignore | Added .akar/ | Clean tree |
| Prepare | `akar prepare "fix the mismatch..."` | Unknown, (no verification command discovered), READY, PASS |
| Fix | `multiply(2,4)=7` → `multiply(2,4)=8` in calc.txt | 1 line changed |
| Verify | Manual compare calc.txt vs expected.txt | PASS |
| Finish | `akar finish` | PASS, 1 file/2 LOC within 3/60 budget |
| Commit | `git add calc.txt && git commit` | Clean tree |

**AKAR commands:** 2 (prepare, finish)
**Git commands:** 5 (init, add ×2, commit ×2)
**Project commands:** 0 (no automated test available)
**Total commands:** 7

## 11. Project-Kind Preservation

| Lane | Project Kind Reported | Verification Command | Cross-contamination? |
|---|---|---|---|
| Rust | Rust | `cargo build && cargo test` | None |
| Node | Node | `npm test (run manually)` | None |
| Python | Python | `python -m pytest (run manually)` | None |
| Unknown Makefile | Unknown | `make test (discovered; run manually)` | None |
| Unknown no-hint | Unknown | `(no verification command discovered)` | None |

Zero cross-contamination: no cargo surfaced in Node/Python, no npm surfaced in Rust/Python,
no pytest surfaced in Rust/Node. Makefile hint was only discovered when the Makefile existed.
Both Unknown fixtures correctly identified as Unknown.

## 12. Verification Guidance Preservation

| Lane | Prepare output | NEXT_RUN.md | Manual-only? |
|---|---|---|---|
| Rust | `cargo build && cargo test` | Cargo commands in Allowed/Verification | Rust gets automated verify |
| Node | `npm test (run manually)` | npm test in Allowed/Verification with manual prefix | Yes |
| Python | `python -m pytest (run manually)` | pytest in Allowed/Verification with manual prefix | Yes |
| Unknown Makefile | `make test (discovered; run manually)` | make test with confidence annotation | Yes |
| Unknown no-hint | `(no verification command discovered)` | Documented-verification fallback | Yes |

All non-Rust lanes correctly received the "(run manually)" suffix. Unknown Makefile
correctly received "(discovered; run manually)". Unknown no-hint correctly received
the no-discovery message. Rust correctly received automated verification.

## 13. Safety Boundary Preservation

| Boundary | Rust | Node | Python | Unknown Makefile | Unknown no-hint |
|---|---|---|---|---|---|
| Did not run project tests | Yes | Yes | Yes | Yes | Yes |
| Did not install dependencies | Yes | Yes | Yes | Yes | Yes |
| Did not edit source | Yes | Yes | Yes | Yes | Yes |
| Did not git add | Yes | Yes | Yes | Yes | Yes |
| Did not git commit | Yes | Yes | Yes | Yes | Yes |
| Did not modify Claude settings | Yes | Yes | Yes | Yes | Yes |
| Did not auto-edit .gitignore | Yes | Yes | Yes | Yes | Yes |
| Did not use destructive git | Yes | Yes | Yes | Yes | Yes |

All five boundaries held across all five lanes. AKAR never crossed into project-code
territory. Every fixture required the user to: author the fix, run verification, and
commit manually. AKAR stayed strictly advisory.

## 14. Prepare Behavior Across Lanes

Prepare correctly performed the following in every fixture:

| Behavior | Rust | Node | Python | Unk Makefile | Unk no-hint |
|---|---|---|---|---|---|
| Created DIFF_BASELINE.json | Yes | Yes | Yes | Yes | Yes |
| Generated NEXT_RUN.md | Yes | Yes | Yes | Yes | Yes |
| Validated internally (check: PASS) | Yes | Yes | Yes | Yes | Yes |
| Printed governor decision | Yes (READY) | Yes (READY) | Yes (READY) | Yes (READY) | Yes (READY) |
| Printed project kind | Yes (Rust) | Yes (Node) | Yes (Python) | Yes (Unknown) | Yes (Unknown) |
| Printed verification guidance | Yes | Yes | Yes | Yes | Yes |
| Printed next instruction | Yes | Yes | Yes | Yes | Yes |
| Required clean tree | Yes | Yes | Yes | Yes | Yes |

No unexpected behavior observed. Prepare is consistent across all lanes.

## 15. Finish Behavior Across Lanes

Finish correctly performed the following in every fixture:

| Behavior | Rust | Node | Python | Unk Makefile | Unk no-hint |
|---|---|---|---|---|---|
| Required baseline | Yes | Yes | Yes | Yes | Yes |
| Measured diff against baseline | Yes | Yes | Yes | Yes | Yes |
| Printed postmortem result | PASS | PASS | PASS | PASS | PASS |
| Printed diff summary | Yes | Yes | Yes | Yes | Yes |
| Printed learning summary | none | none | none | none | none |
| Printed governor decision | RUN_POSTMORTEM | RUN_POSTMORTEM | RUN_POSTMORTEM | RUN_POSTMORTEM | RUN_POSTMORTEM |
| Printed manual commit guidance | Yes | Yes | Yes | Yes | Yes |
| Preserved budget behavior | Yes | Yes | Yes | Yes | Yes |
| Did not auto-resolve patches | Yes | Yes | Yes | Yes | Yes |

All budget verdicts were PASS. Diff measurements were correct (1 file, 1 added, 1 deleted,
2 total LOC for single-line fixes; Unknown fixtures had 1 added, 1 deleted for the
calc.txt line replacement). Governor consistently showed RUN_POSTMORTEM at finish time
(working tree dirty with baseline present — expected state before the user commits).

## 16. Command-Count Comparison

| Release | Lane | AKAR commands per task | Trend |
|---|---|---|---|
| v0.45.0 | Node | 8 | Baseline (measured) |
| v0.46.0 | Node | 2 | 75% reduction |
| v0.47.0 | Rust | 2 | Same 2 |
| v0.47.0 | Node | 2 | Same 2 |
| v0.47.0 | Python | 2 | Same 2 |
| v0.47.0 | Unknown Makefile | 2 | Same 2 |
| v0.47.0 | Unknown no-hint | 2 | Same 2 |

**The 2-command target is lane-agnostic.** Across five distinct project types, prepare/finish
consistently reduces the per-task AKAR command count from 8 to 2. No lane required extra
AKAR commands.

### Per-fixture breakdown

| Fixture | AKAR | Git | Project verify | Edits | Total |
|---|---|---|---|---|---|
| Rust | 2 | 5 | 2 | 1 | 10 |
| Node | 2 | 5 | 2 | 1 | 10 |
| Python | 2 | 5 | 2 | 1 | 10 |
| Unk Makefile | 2 | 5 | 0 | 1 | 8 |
| Unk no-hint | 2 | 5 | 0 | 1 | 8 |

Git commands are consistent (5: init + initial add + commit + .gitignore add + commit).
Project verification commands are consistent (2: pre-test confirmation + post-fix verify).
The Unknown fixtures have 0 project verification commands due to no automated test runner.

## 17. Manual Burden Remaining

Even with prepare/finish, the user must still:
1. Run initial verification manually to confirm the bug.
2. Read `.akar/NEXT_RUN.md` and hand it to the AI.
3. Manually apply the fix.
4. Manually run project verification.
5. Manually review git diff/status.
6. Manually commit.

AKAR commands are down from 8 to 2 — but the total session is still 8–10 commands
when counting git, project, and edit/inspect actions. The remaining AKAR burden is
minimal; the remaining manual burden is in project-specific actions (test, commit)
that AKAR correctly refuses to automate.

The largest remaining AKAR friction is not command count — it's that the user must
remember to run `prepare` before the task and `finish` after. Auto-invocation
(hooks or CLAUDE.md snippets) is the next frontier.

## 18. What Worked

1. **Project-kind detection is rock-solid.** All five lanes were correctly identified
   with zero false positives or cross-contamination.
2. **Verification guidance adapts correctly per lane.** Rust got cargo, Node got npm,
   Python got pytest, Unknown Makefile got the discovered hint, Unknown no-hint got
   the no-discovery fallback.
3. **Prepare output is sufficient.** The user gets everything they need in one output:
   task, project kind, baseline info, request mode, check result, governor decision,
   verification guidance, and next step.
4. **Finish output is sufficient.** Budget verdict, diff summary, learning patches,
   governor decision, health summary, and manual commit guidance all present.
5. **Safety boundaries are consistent.** No lane triggered any boundary violation.
   AKAR never executed project code, mutated git, or modified settings.
6. **Budget measurement is accurate.** All five fixtures measured 2 LOC of change
   (1 added + 1 deleted for single-line replacements), correctly within the 3-file
   60-LOC Bugfix budget.
7. **Governor decisions are coherent.** READY at prepare time (clean tree + baseline),
   RUN_POSTMORTEM at finish time (dirty tree + baseline). This is consistent and correct.
8. **`.gitignore` handling is manual but clear.** AKAR's `.akar/` notice from init
   correctly surfaces the need to handle `.akar/` state without making the decision.

## 19. What Hurt

1. **`.akar/` dirty-tree friction on init.** Every fixture required manual `.gitignore`
   handling after `akar init`. This is a known design trade-off (AKAR doesn't decide
   for you) but it adds 2 git commands per fixture. A future `.gitignore` auto-add
   toggle could eliminate this — but would need careful safety design.
2. **Cargo.lock in Rust fixtures.** The Rust fixture required an extra commit for
   Cargo.lock (generated by `cargo test`). Like `.akar/`, this is a project concern,
   not an AKAR concern — but it adds friction to the first session.
3. **Governor SPLIT_TASK despite zero budget issues.** In the main AKAR repo, the
   governor reports SPLIT_TASK due to a pre-existing learning patch. This is unrelated
   to prepare/finish — it's an artifact of the AKAR repo's own LEARNING_PATCHES.md.
   But it means `prepare` in the AKAR repo would show SPLIT_TASK instead of READY.
   This is correct behavior (the governor reads local evidence) but could confuse
   a first-time user who doesn't understand why their governor disagrees with the
   dogfood report.
4. **make not available on Windows.** The Unknown Makefile fixture couldn't run
   `make test`. The verification discovery hint was correct — it found the Makefile
   and suggested `make test` — but the user had to fall back to manual comparison.
   This is a platform limitation, not an AKAR bug. The discovery hint includes the
   command but doesn't check whether the tool is installed. This is by design
   (AKAR doesn't probe for tool availability).

## 20. Bugs or Confusing Output Found

No bugs found in prepare or finish. No incorrect project-kind detection. No
cross-contamination. No safety boundary violations.

**Confusing output noted:**
- `akar doctor` after init: "source template directory not present in this repo"
  is repeated as a warning on every fixture init. This is known (v0.41.0 wording)
  and correct (installed AKAR doesn't have the source templates). Not a regression.
- Governor READY at prepare time vs RUN_POSTMORTEM at finish time: the governor
  transitions from READY (clean + baseline) to RUN_POSTMORTEM (dirty + baseline)
  as expected. This could confuse users who expect the governor to say the same
  thing, but it correctly reflects the changed state.
- Unknown fixtures get a project kind WARN in doctor output. This is expected and
  documented — Unknown is WARN by design.

## 21. Required Fixes Before AI-Facing Delivery

No fixes required to prepare or finish themselves. The commands are correct across
all lanes.

Before AI-facing delivery (CLAUDE.md snippets or auto-invocation), three
non-code items need attention:

1. **Governor state at prepare time needs context.** When the governor says READY,
   it means "ready to start the session." When it says RUN_POSTMORTEM at finish,
   it means "the session is done, run postmortem." Both are correct but the decision
   name alone doesn't distinguish pre-task from post-task state. AI-facing delivery
   would benefit from a decision that says "START_SESSION" rather than READY when
   called from prepare context.
2. **prepare could surface contract budget more prominently.** The budget (files/LOC)
   is shown in the baseline line but could be a standalone line for AI parsing.
3. **finish could report whether verification was detected as already run.**
   Currently it says "Run project verification if not already done." An AI reading
   this doesn't know whether the user already ran tests.

None of these are blocking for v0.47.0. They belong in the AI-facing delivery
design phase (v0.48.0).

## 22. Stable Prepare/Finish Verdict

**Prepare and finish are stable across all five project lanes.** The commands:
- Correctly detect project kind in every lane
- Correctly generate lane-appropriate verification guidance
- Correctly discover verification hints for Unknown projects
- Correctly validate NEXT_RUN.md internally
- Correctly measure diff against baseline
- Correctly enforce safety boundaries (no project execution, no git mutations)
- Correctly surface governor decisions and health warnings
- Consistently deliver 2 AKAR commands per task

No regressions, no bugs, no cross-contamination. The prepare/finish pattern
is ready for the next phase: AI-facing delivery.

## 23. Recommended Next Release

**v0.48.0 AI-Facing Delivery Design Refinement**

Prepare and finish work across all lanes. The remaining bottleneck is not command
count — it's that AKAR context must still be manually relayed to the AI (the user
must tell Claude to read `.akar/NEXT_RUN.md`). The v0.44.0 design proposed a
managed CLAUDE.md snippet as the primary delivery mechanism. v0.48.0 should refine
that design:

- Design the exact CLAUDE.md snippet content for each project kind
- Design the toggle mechanism (enable/disable/config.toml)
- Design the auto-invocation trigger (when does AKAR context reach Claude?)
- Address the three non-code items from Section 21
- Decide scope: is CLAUDE.md snippet enough, or is a hook-based trigger needed?
- Produce a design report, not an implementation — evidence before code

Do not recommend: capsules, token optimizer, Codex/OpenCode adapters, skill resolver,
autopilot, memory engine, daemon, or auto-execution.

## 24. Honest Conclusion

**The prepare/finish consolidation is proven to work across every lane AKAR supports.**
Five fixtures, five different project types, five different verification strategies,
and in every case: 2 AKAR commands, correct project-kind guidance, zero safety
boundary violations, zero project-code execution.

The 75% command reduction first measured in v0.46.0 Node dogfood holds across:
- Rust (cargo test)
- Node (npm test)
- Python (pytest)
- Unknown with Makefile hint (discovered make test)
- Unknown with no hints (manual verification only)

This is not surprising — prepare and finish compose existing operations that were
already lane-aware — but it needed proof. The proof is now complete.

The remaining gap between current AKAR and the North Star is not in command
consolidation. It's in AI-facing delivery. The user still manually hands
`.akar/NEXT_RUN.md` to Claude. The next release should design how to close that
gap without crossing the safety boundaries that make AKAR trustworthy.
