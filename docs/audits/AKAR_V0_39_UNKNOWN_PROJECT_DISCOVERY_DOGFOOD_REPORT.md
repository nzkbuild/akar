# AKAR v0.39.0 — Unknown-Project Discovery Dogfood Audit Report

## 1. Executive Verdict

**PASS.** Two controlled external Unknown-project dogfood trials prove AKAR's
v0.38.0 verification discovery hints work as designed. Fixture A (Makefile `test:`
target) correctly surfaced `make test` as a Medium-confidence hint in doctor,
NEXT_RUN, and verify output. Fixture B (no markers, no hints) correctly reported
"no confident verification command discovered" and fell back to documented-verification
guidance. No commands were invented or executed by AKAR in either trial. The advisory
loop (preflight → request → postmortem) completed cleanly for both fixtures with
budget discipline intact.

## 2. AKAR Baseline

- **Version:** 0.38.0
- **Commit:** e379741
- **Tests:** 508 passed, 0 failed
- **Evals:** 28/28 PASS
- **Working tree:** clean
- **Branch:** master (ahead of origin/master by 33 commits)

## 3. Why This Trial Matters

v0.38.0 added verification discovery hints but tested them only against sterile
temp-directory fixtures. The Unknown-project path through NEXT_RUN compilation
(which handles discovery hints differently than known projects — all hints go
into Verification Required with a confirmation prefix) had never been exercised
end-to-end. The no-hint fallback path had never been exercised with a real
advisory loop. Both gaps had to be closed before Unknown projects could be
declared dogfood-proven.

## 4. Fixture A Description: Unknown Makefile

- **Path:** `../akar-dogfood-v039-unknown-makefile-fixture`
- **Files:** README.md, calc.txt, expected.txt, Makefile
- **Bug:** `multiply(3,3)=6` in calc.txt, expected `9`
- **Makefile:** `test:` target using `diff calc.txt expected.txt`
- **No project markers present**

## 5. Fixture A Marker Absence Proof

All five markers absent:
- `Cargo.toml`: absent
- `package.json`: absent
- `pyproject.toml`: absent
- `setup.py`: absent
- `requirements.txt`: absent

`akar init` and doctor correctly reported: "project kind — Unknown — no Rust,
Node, or Python markers found."

## 6. Fixture A Discovery Hint Result

**Doctor verification hints section:**
```
[PASS] verification hints: make test (Medium, Makefile)
```

Correct: hint command is `make test`, source is `Makefile`, confidence is
Medium (build tool file — not a known project config).

## 7. Fixture A Setup Path Result

- `akar init` → clean (`.akar/` directory created, no templates copied)
- `git status` → `.akar/` untracked
- Added `.gitignore` + commit → clean tree
- `akar hooks --install` → installed to `.akar/hooks/`
- `akar hooks --check` → PASS (source: project .akar/hooks)

## 8. Fixture A Doctor/Status/Verify Result

**Doctor:** WARN (expected — NEXT_RUN and baseline missing, project kind Unknown)
- project kind: WARN "Unknown"
- verification hints: PASS "make test (Medium, Makefile)"

**Status:** HEALTHY
- project: akar-dogfood-v039-unknown-makefile-fixture
- readiness: BLOCKED (no baseline, dirty tree)

**Verify:**
```
Verified:
  (no automated checks)
Manual checks:
  - no automated verify for Unknown projects — discovered verification hint(s): make test (Medium, Makefile)
  - check changed files match task scope
  - no secrets in output
```

Correct: `akar verify` did not run `make`, correctly included the discovery hint
in the manual-check message, and correctly refused automated execution for Unknown.

## 9. Fixture A NEXT_RUN Quality

**Current State:** project kind: Unknown, requested task present

**Allowed Commands:** No Cargo, npm, pytest, or make commands. Only AKAR CLI +
git read-only commands.

**Verification Required:**
```
- Ask the user before running discovered verification command: `make test`  *(Medium, Makefile)*
- Run the project's documented verification command.
- Inspect README or project scripts before choosing a test command.
```

Correct: `make test` is present with confirmation prefix, confidence and source
annotated, standard Unknown-project guidance included.

**No invented commands:** grep for `Cargo`, `npm`, `pytest`, `Makefile` in
NEXT_RUN only matched the fixture name and the discovery hint — zero invented
verification commands.

## 10. Fixture A Task Execution and Verification

- **Fix:** Changed `multiply(3,3)=6` to `multiply(3,3)=9` (1 file, 1 line)
- **Verification:** Manual `diff` confirmed calc.txt and expected.txt match
- **`make` availability:** Not available on this Windows machine (confirmed
  via `which make`). The Makefile `test:` target could not be executed, but the
  manual diff served as equivalent verification.

## 11. Fixture A Postmortem Result

```
postmortem --diff:
  task:    Bugfix
  budget:  3 files, 60 LOC
  actual:  1 files, 1 added, 2 deleted (3 total changed LOC)
  status:  PASS
```

Within budget. Governor decision post-fix: RUN_POSTMORTEM (dirty tree with baseline).

## 12. Fixture B Description: Unknown No-Hint

- **Path:** `../akar-dogfood-v039-unknown-nohint-fixture`
- **Files:** README.md, calc.txt, expected.txt
- **README:** Plain project description, no whitelisted verification commands
- **Bug:** `multiply(2,4)=7` in calc.txt, expected `8`
- **No markers, no Makefile, no justfile**

## 13. Fixture B Marker/Hint Absence Proof

All seven marker/hint sources absent:
- `Cargo.toml`: absent
- `package.json`: absent
- `pyproject.toml`: absent
- `setup.py`: absent
- `requirements.txt`: absent
- `Makefile`: absent
- `justfile`: absent

README whitelist check: all six whitelisted commands absent (`npm test`,
`python -m pytest`, `pytest`, `cargo test`, `make test`, `just test`).

Doctor correctly reported:
```
[WARN] verification hints: no confident verification command discovered; use README or project docs
```

## 14. Fixture B Setup Path Result

- `akar init` → clean (`.akar/` directory created)
- Added `.gitignore` + commit → clean tree
- `akar hooks --install` → embedded fallback used (no source tree)
- `akar hooks --check` → PASS (source: embedded)

## 15. Fixture B Doctor/Status/Verify Result

**Doctor:** WARN
- project kind: WARN "Unknown"
- verification hints: WARN "no confident verification command discovered"
- Additional advisory: "verification hints — no confident verification command
  discovered; use README or project docs"

**Verify:**
```
Verified:
  (no automated checks)
Manual checks:
  - no automated verify for Unknown projects — use the project-specific test command
```

Correct: no invented verification commands, honest "use the project-specific
test command" fallback.

## 16. Fixture B NEXT_RUN Fallback Quality

**Current State:** project kind: Unknown, requested task present

**Verification Required:**
```
- Run the project's documented verification command.
- Inspect README or project scripts before choosing a test command.
```

No `make test`, `npm test`, `pytest`, or `cargo test` appears anywhere in
NEXT_RUN (confirmed with `grep -c`: zero matches for all four commands).

**Allowed Commands:** Only AKAR CLI + git read-only commands.

## 17. Fixture B Task Execution and Verification

- **Fix:** Changed `multiply(2,4)=7` to `multiply(2,4)=8` (1 file, 1 line)
- **Verification:** Manual `diff` confirmed calc.txt and expected.txt match

## 18. Fixture B Postmortem Result

```
postmortem --diff:
  task:    Bugfix
  budget:  3 files, 60 LOC
  actual:  1 files, 1 added, 2 deleted (3 total changed LOC)
  status:  PASS
```

Within budget, clean.

## 19. Hook Evidence Result

No live hook events were generated in either fixture — Claude Code's PreToolUse
hook was not wired in these fixtures. This is expected: hook integration was
proven in v0.35/v0.36 and this trial focused on verification discovery correctness.
- Fixture A HOOK_EVENTS.jsonl: absent
- Fixture B HOOK_EVENTS.jsonl: absent
- Doctor correctly reports absence without error

## 20. What AKAR Helped With

- **Discovery hint surfaced correctly (Fixture A):** The `make test` hint in
  doctor, NEXT_RUN, and verify gave a non-technical user a concrete starting
  point instead of a blank "documented verification command" prompt
- **No false positives (Fixture B):** When no hints existed, AKAR honestly
  reported that fact rather than guessing
- **Budget discipline worked for both fixtures:** Preflight → snapshot →
  postmortem tracked the minimal one-line fix correctly in both cases
- **Project-kind detection worked:** Both fixtures correctly identified as Unknown
- **Embedded hook template worked:** `akar hooks --install` and `akar hooks --check`
  functioned without the AKAR source tree in both fixtures

## 21. What AKAR Made Worse

- **Dirty-tree friction on init:** The `.akar/` directory created by `akar init`
  dirtied the tree immediately, requiring the user to gitignore it before the
  preflight snapshot would work. This is a known friction point (since v0.27)
  that persists — AKAR still does not auto-ignore its own directory.
- **Verify stays manual-only:** For Unknown projects, `akar verify` correctly
  refuses automated execution, but the user still has to figure out manual
  verification themselves. The discovery hint helps, but it doesn't execute
  anything.

## 22. Confusing or Misleading Output

- Fixture A: "templates directory not found" on init (misleading — the embedded
  fallback still works, as proven by `hooks --install` succeeding)
- Both fixtures: doctor shows WARN for things that are expected on a fresh
  project (no NEXT_RUN, no baseline) — not misleading per se, but could alarm
  a new user

## 23. Manual Rescue Required

Minimal — the standard advisory-loop setup path (gitignore `.akar/`, hooks install)
is well-documented. No data loss, no corruption, no destructive recovery needed.

## 24. Unknown Project Alpha Verdict

**PASS.** The Unknown-project path through AKAR's advisory loop works end-to-end:
- Init correctly detects Unknown
- Doctor reports project kind and discovery hints honestly
- Verify stays manual-only with appropriate hints
- NEXT_RUN surfaces hints safely (confirmation prefix, confidence-labeled) and
  never invents commands
- The no-hint fallback is coherent and honest
- Budget discipline is maintained

## 25. Stable Alpha Status After This Trial

All four project-type lanes are now dogfood-proven:

| Project Kind | Dogfood Version | Verdict |
|-------------|----------------|---------|
| Rust | v0.24.0 (and every release since) | PASS |
| Node | v0.33.0 | PASS |
| Python | v0.37.0 | PASS |
| Unknown (with hints) | v0.39.0 | PASS |
| Unknown (no hints) | v0.39.0 | PASS |

## 26. Required Fixes Before v1.0.0

1. **Multi-task session dogfood** — all trials have been single-task (one baseline,
   one fix, one postmortem). A session with sequential tasks needs proof.
2. **Dirty-tree recovery guidance** — the `.akar/`-dirties-tree-on-init friction
   needs docs or a `doctor` hint so new users don't get stuck.
3. **Cross-platform hook validation** — macOS and Linux hook templates need
   independent verification.
4. **Hook install automation decision** — decide whether `akar hooks --install`
   should eventually write to `~/.claude/settings.json` or stay manual forever.
5. **"templates directory not found" wording** — this init warning is confusing
   when the embedded fallback works; should be clarified.

## 27. Honest Conclusion

The v0.38.0 discovery hints module works correctly for Unknown projects. A user
with a Makefile gets `make test` surfaced as a hint with a confirmation gate.
A user with no hints gets an honest "nothing found, check your docs" message.
AKAR didn't invent, guess, or execute a single command in either trial.

The Unknown-project experience is still fundamentally manual — the user
must pick a verification method and run it themselves — but the discovery
hints make that manual task concrete rather than open-ended. For the non-technical
user this feature was designed for, seeing "make test (Medium, Makefile)" is
a meaningful improvement over "Run the project's documented verification command."

## 28. Next Recommended Release

**v0.40.0 — Multi-Task Session Dogfood.** Run a full advisory loop with two
sequential tasks on a single fixture (preflight → fix → postmortem → preflight →
fix → postmortem) to prove the governor transitions correctly between tasks and
the baseline/postmortem chain stays coherent across task boundaries.
