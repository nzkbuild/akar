# AKAR v0.26.0 — NEXT_RUN Task Threading Report

## 1. Baseline

Starting point: AKAR v0.25.0, 420 tests passing, clean release build. The v0.24 first-external-dogfood trial (`docs/audits/AKAR_V0_24_FIRST_EXTERNAL_DOGFOOD_REPORT.md`) recorded five friction findings for later releases; v0.25.0 addressed the hook-template-discovery blocker. This release addresses two of the remaining findings: the generic NEXT_RUN objective, and the Cargo.lock clean-tree friction on `preflight --snapshot`.

## 2. Dogfood issue addressed

From the v0.24 report: "the NEXT_RUN objective is generic and does not echo the user's actual task" and "`preflight --snapshot` requires a clean tree so `cargo test`-generated `Cargo.lock` had to be committed first" with no guidance from AKAR about what to do with it.

## 3. Request task syntax

`akar request` now accepts the task text in two equivalent forms:

```
akar request "fix one small failing test"
akar request --task "fix one small failing test"
```

Parsing (`src/main.rs`, `"request"` match arm): `--task <text>` is checked first; if absent, the first positional argument after `request` that does not start with `--` is used. Omitting the task leaves `task = None` and the compiled prompt is byte-for-byte identical to pre-v0.26.0 output (verified by the "no-task preserves behavior" test).

## 4. NEXT_RUN task threading behavior

Task text flows through `loop_governor::compile_next_run_prompt(cfg, report, task)`:

- Trimmed; empty/whitespace-only text is treated as no task.
- Redacted via `config::redact()` and collapsed to one line via `one_line()` before being written anywhere.
- Written once to `## Current State` as `- requested task: <text>`.
- Written to `## Objective`, with wording depending on decision class (see §5).

`request --check`'s validator contract is unaffected: the base `objective_for_decision()` string is always emitted verbatim before any task-specific lines, so the existing `content.contains(expected_objective)` check still passes on task-threaded prompts.

## 5. Stop-class safety behavior

Decision-class-aware wording, so the task can never look like it overrides governor safety:

- **Continue / action-required** (READY, SNAPSHOT_NOW, RUN_POSTMORTEM, COMMIT_CHECKPOINT, SPLIT_TASK): task is appended as a secondary line — `- Task: <text>`.
- **Stop** (STOP_HOOK_BROKEN, STOP_REPEATED_BLOCK, UNKNOWN): the objective gains `- Primary objective: resolve the governor blocker above before attempting any task.` followed by `- Requested task after the blocker is resolved: <text>`. The literal string `- Task: <text>` never appears for stop-class decisions — verified by `stop_hook_broken_keeps_blocker_primary_and_task_secondary` and equivalent tests for STOP_REPEATED_BLOCK and UNKNOWN.

## 6. Cargo.lock advisory behavior

`cargo_lock_dirty_advisory(project_root)` (`src/main.rs`) fires only when: `Cargo.toml` exists in the project root, AND `git status --porcelain` reports dirty files, AND the *only* dirty file is `Cargo.lock` (or a nested `.../Cargo.lock`). It prints exactly:

> Cargo.lock changed or was generated. Review and commit it intentionally before snapshot, or remove it only if it is truly unwanted. AKAR will not decide for you.

as an extra line in `preflight --snapshot`'s existing dirty-tree refusal path. It does not change the refusal itself — snapshot still exits 1 on any dirty tree, including a dirty-only-Cargo.lock tree. AKAR does not stage, commit, ignore, or delete Cargo.lock on the user's behalf.

## 7. Tests added/updated

`src/loop_governor.rs` (+15): task threading into Current State (present, absent, empty-string-treated-as-absent); per-decision Objective threading for READY, SNAPSHOT_NOW, RUN_POSTMORTEM, COMMIT_CHECKPOINT, SPLIT_TASK; blocker-primary/task-secondary behavior for STOP_HOOK_BROKEN, STOP_REPEATED_BLOCK, UNKNOWN; task redaction (secret-like text stripped); `request --check` validator still PASSes a task-threaded prompt; `governor` command path confirmed to still not write NEXT_RUN.md even when called with task-shaped input; `write_governor_next_run` with a task writes both the Current State and Objective lines.

`src/main.rs` (+4, plus a `temp_git_repo` test helper): Cargo.lock advisory fires when Cargo.lock is the sole dirty file; does not fire when another file is also dirty; does not fire without a `Cargo.toml`; does not fire on a fully clean tree.

`src/doctor.rs`: one existing call site updated to pass `None` for the new task parameter (doctor's NEXT_RUN validity test does not exercise task threading — it's out of scope for doctor).

Total: 420 → 439 tests (+19).

## 8. Verification

All commands below were run from `C:\Users\nbzkr\Coding\akar` unless noted as external.

```
cargo build --release        → Finished `release` profile [optimized] target(s)
cargo test                   → test result: ok. 439 passed; 0 failed; 0 ignored
akar --version                → akar 0.25.0 (pre-bump; confirmed matches Cargo.toml before bump)
akar request "fix one small failing test"
                               → wrote .akar/NEXT_RUN.md; Objective contained "- Task: fix one small failing test"
                                 under the current repo's real governor decision (SPLIT_TASK, action-required class)
akar request --check          → NEXT_RUN check: PASS (sections/minimum content/safety contract/decision consistency all PASS)
akar governor --json --no-exit-code
                               → valid single-line JSON object, decision/reason/next_action/suggested_prompt/evidence_used present
akar doctor                   → doctor: WARN (pre-existing dirty-tree and active-split-rule advisories, unrelated to this release)
akar hooks --check             → status: PASS, source: source-tree
akar eval                      → overall: PASS (28/28)
```

External-repo verification (temp repo at `%TEMP%\akar_v026_external_dogfood`, outside the AKAR source tree, using the release binary at `C:\cargo-target\steroid-cli\release\akar.exe`):

1. Created a fresh git repo with `Cargo.toml` + `src/lib.rs` containing a deliberately buggy `add()` (`a * b` instead of `a + b`) and one failing test; committed.
2. `akar init` — bootstrapped `.akar/` successfully.
3. `akar request "fix one small failing test in the dogfood fixture"` — governor decision was SNAPSHOT_NOW (continue-class, clean tree, no baseline). Wrote `.akar/NEXT_RUN.md`.
4. Inspected `.akar/NEXT_RUN.md`: `## Current State` contained `- requested task: fix one small failing test in the dogfood fixture`; `## Objective` contained `Create a clean baseline snapshot before making changes.` followed by `- Task: fix one small failing test in the dogfood fixture`; all safety sections (Hard Rules, Forbidden Commands, Stop Conditions) present and unchanged from the standard contract.
5. `akar request --check` — `NEXT_RUN check: PASS`.
6. Temp repo deleted after verification; nothing was committed to the AKAR repo from it.

## 9. Honest conclusion

Task threading works as specified: it is purely additive context in `Current State` and `Objective`, it is redacted and single-lined before being written, and it never displaces the governor's safety-critical objective for stop-class decisions. The validator contract required no changes because the base objective string is always emitted first. The Cargo.lock advisory is a single extra printed line gated on a narrow precondition (Cargo.lock is the *only* dirty file) — it does not change snapshot's refuse-on-dirty behavior, and AKAR still makes no decision on the user's behalf about committing or discarding Cargo.lock. Both external and internal verification produced the expected NEXT_RUN content. No governor decision rule, exit code, telemetry event, hook behavior, or Claude Code configuration was touched.

## 10. Next recommended release

The v0.24 report's remaining unaddressed friction finding is: "`akar status` shows DEGRADED on every fresh external repo due to the hook-template FAIL" — this was actually already resolved by v0.25.0's embedded-fallback change (confirmed in this session's external dogfood run: `akar init` on the fresh repo reported doctor issues about missing NEXT_RUN/baseline, not a hook-template FAIL). No new blocking friction was found in this session's external trial. Recommended next step is a second external dogfood trial that carries a fix through the full loop (snapshot → task-threaded request → fix → postmortem → learn) on a fresh fixture, to collect fresh friction evidence now that task threading and the Cargo.lock advisory exist, before considering any further command surface changes.
