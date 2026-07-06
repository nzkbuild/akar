# AKAR v0.28.0 — AKAR State Dirty-Tree Advisory Report

## 1. Baseline

Starting point: AKAR v0.27.0, 439 tests passing, clean release build, working tree clean. The v0.27.0 second external dogfood trial (`docs/audits/AKAR_V0_27_SECOND_EXTERNAL_DOGFOOD_REPORT.md`) found that `akar init` leaves `.akar/` untracked with no `.gitignore` guidance, so the very next command (`preflight --snapshot`) refuses on a "dirty" tree that is dirty only because of AKAR's own generated output — with no advisory pointing at the cause, unlike the existing Cargo.lock advisory from v0.26.0.

## 2. Dogfood issue addressed

From the v0.27.0 report §13: "`akar init` leaves `.akar/` completely untracked with no `.gitignore` guidance, so the very next command (`preflight --snapshot`) refuses on a 'dirty' tree that is dirty only because of AKAR's own generated output. A first-time user following the documented setup sequence hits a refusal caused entirely by AKAR itself, with no advisory pointing at the fix." This release adds that advisory, mirroring the v0.26.0 Cargo.lock advisory pattern exactly.

## 3. Detection behavior

`akar_state_dirty_advisory(project_root)` (`src/main.rs`) runs `git status --porcelain`, collects the dirty/untracked file list, and fires only when every entry is `.akar` itself or falls under `.akar/` (checked via `f == ".akar" || f == ".akar/" || f.starts_with(".akar/")`). If a single dirty file falls outside `.akar/`, the function returns `None` and the generic dirty-tree message stands alone. This is deliberately narrow — the same "only" pattern used by `cargo_lock_dirty_advisory` — so it never fires when `.akar/` state is dirty alongside real source changes.

## 4. Advisory text

When triggered, the advisory prints:

> AKAR local state is making the tree dirty. Review it, then intentionally add .akar/ to .gitignore or commit the files you want tracked before taking a snapshot. AKAR will not decide for you.
> .akar/ holds local runtime state (NEXT_RUN.md, DIFF_BASELINE.json, HOOK_EVENTS.jsonl, EVENT_LOG.jsonl, installed hook templates) by default.
> Do not use destructive cleanup (git clean, git reset --hard) to force this away — review the files first.
> Rerun 'akar preflight --snapshot "<task>"' once the tree is intentionally clean.

It explicitly warns against destructive cleanup and tells the user to rerun snapshot after deciding intentionally — it never tells AKAR to decide, delete, or commit anything itself.

## 5. Refusal behavior unchanged

The advisory is printed as an additional `eprintln!` line inside the existing `Ok(false)` dirty-tree branch of `cmd_preflight`'s snapshot path (`src/main.rs`), immediately after the pre-existing Cargo.lock advisory check. `process::exit(1)` still runs unconditionally afterward. Snapshot still refuses on any dirty tree, including a dirty-only-`.akar/` tree — confirmed by the new `preflight_still_refuses_dirty_akar_only_tree` test, which asserts `diff_budget::is_working_tree_clean` (the same function the refusal path calls) returns `Ok(false)` for a tree with only `.akar/NEXT_RUN.md` dirty.

## 6. Cargo.lock advisory compatibility

Both advisory functions are independent, narrow "only" checks with no interaction between them. When both Cargo.lock and `.akar/` are dirty at the same time, neither `cargo_lock_dirty_advisory` nor `akar_state_dirty_advisory` fires — each requires its own dirty set to be the *entire* dirty set, and here the dirty set is a mix of both, so both `None` out. No combined message was implemented, per the task's explicit preference to avoid a combined special case in this release. Verified by `cargolock_and_akar_state_advisories_do_not_both_fire_when_both_dirty`.

## 7. Tests added/updated

`src/main.rs` (+9, all in the existing `tests` module):
- `akar_state_advisory_fires_when_only_next_run_is_dirty`
- `akar_state_advisory_fires_when_only_hook_template_is_dirty`
- `akar_state_advisory_fires_when_only_hook_events_is_dirty`
- `akar_state_advisory_does_not_fire_with_non_akar_file_dirty`
- `akar_state_advisory_does_not_fire_with_akar_plus_source_dirty`
- `akar_state_advisory_does_not_fire_on_clean_tree`
- `akar_state_advisory_does_not_recommend_destructive_commands`
- `cargolock_and_akar_state_advisories_do_not_both_fire_when_both_dirty`
- `preflight_still_refuses_dirty_akar_only_tree`

Total: 439 → 448 tests (+9). No existing tests were modified.

## 8. Verification

All commands below were run from `C:\Users\nbzkr\Coding\akar` unless noted as external.

```
cargo build --release        → Finished `release` profile [optimized] target(s)
cargo test                   → test result: ok. 448 passed; 0 failed
akar --version                → akar 0.28.0
akar doctor                   → doctor: WARN (pre-existing dirty-tree/split-rule advisories in this repo, unrelated)
akar status                   → status: HEALTHY
akar request "dogfood verification" → wrote .akar/NEXT_RUN.md
akar request --check          → NEXT_RUN check: PASS
akar governor --json --no-exit-code → valid JSON, all fields present
akar hooks --check             → status: PASS, source: source-tree
akar eval                      → overall: PASS (28/28)
```

External-repo verification (fresh temp repo outside the AKAR source tree, using the release binary):

1. Created a brand-new git repo, no `.gitignore`, one committed `README.md`; confirmed the tree was clean before AKAR touched it.
2. `akar init` — bootstrapped (no `.akar/` content yet dirties the tree, since `init` alone did not write `.akar/NEXT_RUN.md`).
3. `akar request "populate akar state"` — wrote `.akar/NEXT_RUN.md`.
4. `git status --porcelain` confirmed `?? .akar/` — untracked and the sole cause of a dirty tree.
5. `akar preflight --snapshot "test akar state advisory"` — refused with `preflight --snapshot: working tree is dirty`, followed by the AKAR local state advisory text verbatim (mentioning `.akar/`, "AKAR will not decide for you", the runtime-state file list, and the destructive-cleanup warning).
6. AKAR did not modify, gitignore, delete, or commit anything in the temp repo — the repo was left exactly as the advisory found it, then removed as cleanup.

## 9. Honest conclusion

The advisory fires precisely when `.akar/` alone is the cause of a dirty tree, never when real source changes are also present, and never suggests a destructive command — it consistently points the user toward a `.gitignore` or explicit-commit decision and a rerun. Snapshot's refuse-on-dirty behavior is completely unchanged; this release only adds an explanatory line, exactly mirroring the v0.26.0 Cargo.lock advisory shape. No governor rule, exit code, telemetry event, hook behavior, NEXT_RUN compiler/validator, or Claude Code configuration was touched.

## 10. Next recommended release

The v0.27.0 report's remaining findings — the PreToolUse hook logging to the Claude Code session's cwd instead of the target repo's cwd, and NEXT_RUN's hardcoded cargo-only command lists on non-Rust projects — are still open and were explicitly out of scope for this release. Recommend addressing the hook cwd-resolution issue next, since it affects the accuracy of hook evidence collected during any future external dogfood trial.
