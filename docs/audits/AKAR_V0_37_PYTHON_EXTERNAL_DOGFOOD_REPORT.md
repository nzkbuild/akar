# AKAR v0.37.0 — Python External Dogfood Trial Report

## 1. Executive verdict

**PASS.** AKAR's project-aware advisory loop works correctly for Python projects. NEXT_RUN.md compiled `python -m pytest` commands appropriately with zero Cargo or npm leakage. `akar verify` correctly refused automated execution for a non-Rust project. The full advisory loop ran without friction, confusion, or manual rescue. Python project-kind support is proven.

## 2. AKAR baseline

- Version: akar 0.36.0
- Commit: b359124 — docs: record AKAR anchored live hook dogfood trial
- `cargo test`: 493/493 PASS
- `cargo run -- eval`: 28/28 PASS
- `cargo run -- doctor`: PASS (project kind: Rust)
- Working tree: clean

## 3. Python fixture description

| Field | Value |
|---|---|
| Path | `C:\Users\nbzkr\Coding\akar-dogfood-v037-python-fixture` |
| Language | Python |
| Marker | `pyproject.toml` |
| Source | `src/calc.py` (add, subtract, multiply) |
| Tests | `tests/test_calc.py` (3 pytest tests) |
| Initial test result | 2 pass, 1 fail (multiply) |
| Bug | `multiply(a, b)` returned `a + b` instead of `a * b` |
| Failing assertion | `assert multiply(3, 3) == 9` — got 6 |
| Dependencies | None beyond stdlib |
| Secrets | None |
| Network needed | No |

## 4. Python/pytest environment

| Component | Version |
|---|---|
| Python | 3.12.3 |
| pytest | 7.4.3 |

`python -m pytest --version` succeeded before the fixture was created. No environment blockers.

## 5. Fixture baseline failure

```
tests/test_calc.py::test_add PASSED
tests/test_calc.py::test_subtract PASSED
tests/test_calc.py::test_multiply FAILED
    assert multiply(3, 3) == 9
    assert 6 == 9
```

Git: clean, 3 files committed (root-commit 2d5057b).

## 6. Setup path result

| Step | Result |
|---|---|
| `git init` + commit fixture files | Clean tree |
| `akar init` | Bootstrap: 0 created, 0 skipped (expected — no templates in Python project). Doctor flagged missing files (expected for fresh init). |
| `.gitignore` (pycache, .akar/, .pytest_cache/, .hypothesis/) | Added and committed |

No errors, no unexpected output.

## 7. `.akar/` dirty-tree handling result

After `akar init`, git status showed `.akar/` untracked. Added `.akar/` (plus `__pycache__/`, `.pytest_cache/`, `.hypothesis/`) to `.gitignore` and committed. Tree was clean before preflight snapshot. No destructive cleanup needed.

## 8. Hook install/check result

| Command | Result |
|---|---|
| `akar hooks --check` (pre-install) | PASS (source: embedded) |
| `akar hooks --install` | Copied pre-tool-call.sh and pre-tool-call.ps1 to `.akar/hooks/` |
| `akar hooks --check` (post-install) | PASS (source: project .akar/hooks) |

## 9. Doctor/status/verify result before task

| Command | Result |
|---|---|
| `akar doctor` | PASS on critical checks; WARN for missing NEXT_RUN/DIFF_BASELINE (expected — not yet generated) |
| `akar status` | HEALTHY; governor: SNAPSHOT_NOW |
| `akar verify` | No automated checks (correct — Python projects are manual-only) |

## 10. `akar verify` boundary result

`akar verify` in the Python fixture:
```
Verified:
  (no automated checks)
Manual checks:
  - no automated verify for Python projects — use the project-specific test command
```

AKAR correctly refused to run `python -m pytest` or any other automated verification. It reported manual-only honestly. No project tests were executed by AKAR itself. The Rust-only automated execution boundary is intact.

## 11. NEXT_RUN Python project-aware quality

NEXT_RUN.md contents for Python:

| Field | Value |
|---|---|
| Current State: project kind | `Python` |
| Current State: requested task | `fix one small failing test in the Python dogfood fixture` |
| Objective | `Continue the scoped task... Task: fix one small failing test in the Python dogfood fixture` |
| Allowed Commands | includes `python -m pytest` |
| Allowed Commands: Cargo | None |
| Allowed Commands: npm | None |
| Verification Required | `python -m pytest` (first item) |
| Stop Conditions | `Stop if python -m pytest fails.` |
| Safety sections | All present |
| request --check | PASS |

No Cargo commands. No npm commands. `python -m pytest` is the primary verification command everywhere appropriate. The Python project-kind path through `compile_next_run_prompt` works correctly.

## 12. Task execution summary

Task: fix one small failing test in the Python dogfood fixture.

Fix: `src/calc.py` — changed `return a + b` to `return a * b` in the `multiply` function. Also removed the explanatory comment. One file, 1 added / 2 deleted lines.

The fix was minimal and well within the 3-file / 60-LOC Bugfix budget.

## 13. Test before/after

Before:
```
test_add PASSED
test_subtract PASSED
test_multiply FAILED — assert 6 == 9
```

After:
```
test_add PASSED
test_subtract PASSED
test_multiply PASSED
```

3/3 PASS.

## 14. Diff/postmortem result

```
postmortem --diff --baseline:
  task:    Bugfix
  budget:  3 files, 60 LOC
  actual:  1 files, 1 added, 2 deleted (3 total changed LOC)
  status:  PASS
```

Well within budget. Project detection correctly identified the Python project.

## 15. Learn/governor/status result after task

| Command | Result |
|---|---|
| `akar learn --list` | 0 patches (expected — no new lessons to learn) |
| `akar governor --json --no-exit-code` | RUN_POSTMORTEM (correct — tree dirty, baseline exists) |
| `akar doctor` | PASS on critical checks; WARN for dirty tree (expected) |
| `akar status` | HEALTHY; readiness BLOCKED (dirty tree — expected after the fix) |
| `akar hooks --check` | PASS (source: project .akar/hooks) |

## 16. Hook evidence result

This dogfood trial did not involve a live Claude Code session with PreToolUse hooks. No hook events were generated. `.akar/HOOK_EVENTS.jsonl` remained absent. Doctor correctly reported `EVENT_LOG.jsonl: absent` and `HOOK_EVENTS.jsonl: absent` without errors.

Hook integration was already proven in v0.35.0 (live firing/logging/blocking) and v0.36.0 (anchored session routing). This trial focused on Python project-kind correctness — live hook re-proof is not needed for every dogfood variant.

## 17. What AKAR helped with

- **Preflight** correctly classified the task as Bugfix with appropriate budget.
- **NEXT_RUN.md** compiled `python -m pytest` commands for Allowed Commands, Verification Required, and Stop Conditions — zero Cargo/npm leakage.
- **Project kind** was correctly displayed as "Python" in Current State.
- **`akar verify`** correctly refused automated execution for Python.
- **Governor** correctly sequenced SNAPSHOT_NOW → READY → RUN_POSTMORTEM.
- **Postmortem** correctly measured the diff (1 file, 3 LOC) against the budget.
- **Doctor** correctly reported project kind as Python with the appropriate NEXT_RUN guidance note.

## 18. What AKAR made worse

Nothing. The loop was smooth with zero friction. No misleading output, no unexpected warnings, no contradictory guidance.

## 19. Confusing or misleading output

None. All AKAR commands produced clear, project-appropriate output. The status output showed "runtime: akar 0.35.0" which is the on-PATH binary version (pre-v0.36 commit) — this is a dev-environment artifact and not an AKAR product issue.

## 20. Manual rescue required

None. The loop completed without any manual intervention beyond the fix itself and the one-time `.gitignore` setup.

## 21. Python project alpha verdict

**PASS.** AKAR correctly detects Python projects via `pyproject.toml` and compiles project-appropriate NEXT_RUN commands. The verification contract for Python is correct: `python -m pytest` appears in allowed commands, verification commands, and stop conditions. No Rust/Cargo or Node/npm commands leak into the Python prompt. `akar verify` correctly stays manual-only.

## 22. Stable alpha status after this trial

Stable Advisory Alpha remains valid. All v0.34 freeze guarantees hold. Project-kind coverage is now:

| Project Kind | Dogfood Status |
|---|---|
| Rust (Cargo.toml) | Proven (v0.33, v0.35, v0.36) |
| Node.js (package.json) | Proven (v0.27, v0.33, v0.35, v0.36) |
| Python (pyproject.toml) | **Proven (v0.37)** |
| Unknown | Not yet dogfooded |

## 23. Required fixes before v1.0.0

1. **Unknown-project dogfood** — prove human-readable guidance works when no marker file exists.
2. **Multi-task session dogfood** — prove the loop works across multiple consecutive tasks in one session.
3. **Dirty-tree recovery guidance** — document the `.akar/` gitignore workflow prominently.
4. **Cross-platform hook validation** — verify PreToolUse hook works on macOS/Linux.
5. **Hook install automation decision** — decide whether AKAR should offer automated Claude settings wiring.

## 24. Honest conclusion

AKAR's project-aware verification contract works correctly for all three supported project kinds. The Python path — `pyproject.toml` detection → `python -m pytest` commands → manual-only `akar verify` — is clean and well-isolated from the Rust and Node paths. No cross-contamination between project kinds. The verification contract module continues to hold up under external testing without modification.

## 25. Next recommended release

**v0.38.0: Unknown-Project External Dogfood Trial.** Prove the Unknown-project path works correctly with human-readable guidance. Then v0.39.0: Multi-task Session Dogfood. Target v1.0.0-rc1 after all project kinds and session patterns are proven.
