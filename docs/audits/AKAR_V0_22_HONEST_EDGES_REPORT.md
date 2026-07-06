# AKAR v0.22.0 — Honest Edges Report

**Date:** 2026-07-06
**Scope:** close the highest-risk honesty gaps found by the v0.21 audit before dogfood. Advisory banners, stale-doc corrections, dead-code removal, drift fixes. No new features. No execution. No auto-run. No auto-apply. No new commands.
**Method:** read the v0.21 audit (`AKAR_V0_21_CURRENT_REALITY_AUDIT.md`), the affected `src/*.rs` modules, and the flagged docs; applied surgical edits; rebuilt and ran the full verification matrix after each change.

---

## 1. Baseline

Confirmed in Phase 0 (no files modified before verification):

| Check | Result |
|---|---|
| `git log --oneline -5` | HEAD = `9179056 docs: audit AKAR current reality` |
| `git status` | working tree clean; 16 commits ahead of origin/master (unpushed) |
| `cargo run -- --version` | `akar 0.21.0` |
| `cargo test` | **401 passed; 0 failed; 0 ignored** |
| `cargo run -- eval` | **28/28 PASS** |
| `cargo run -- status` | `status: HEALTHY`, readiness `READY`, governor `READY` |
| `cargo run -- request` | writes `.akar/NEXT_RUN.md`, prints governor block |
| `cargo run -- request --check` | `NEXT_RUN check: PASS` |
| `cargo run -- hooks --check` | `status: PASS`, both templates found |

Baseline is v0.21.0 (audit-only). Tests pass. Tree is clean.

---

## 2. Honesty fixes made

1. **Advisory banners on `akar run` and `akar mission`.** Both commands now lead with an unmistakable banner stating they are advisory only and do NOT execute code, edit files, call models, or run the mission, and pointing to `akar request` for a Claude-ready prompt.
2. **Shadowed NEXT_RUN writer removed.** `cmd_request` no longer calls the older resume-mode `write_next_run`; only the compiled-prompt writer (`write_governor_next_run`) runs. The dead `write_next_run` function and the orphaned `next_run_recommended` field were deleted.
3. **Duplicate budget table centralized.** The four budget-tier caps now live as `pub const BUDGET_CAP_*` in `contract.rs`; both `DiffBudget::micro/small/medium/large` and `diff_budget::budget_for_task_name` read from them. The "no second budget table" comment is now true.
4. **Eval-only policy functions relabeled.** `safety::govern_dependency` and `safety::check_migration` are now documented and commented as a **policy library exercised by evals**, not a runtime dependency/migration governor. No new command behavior added.
5. **Confirmed dead code removed** (see §5).
6. **Stale docs corrected** (see §4).

No new commands. No execution. No mission behavior change. No hook, safety, governor, or NEXT_RUN compiler/validator behavior changed.

---

## 3. Advisory banner changes

### `akar run` (`src/workflow.rs::format_workflow_report`)

The report now opens with an `ADVISORY ONLY` banner stating `akar run` prints strategy and records telemetry and does NOT: execute code, edit files, call models, run the mission — followed by "Run the task yourself in Claude Code. For a Claude-ready next-run prompt, use `akar request`." The pre-existing "advisory scaffold mode" line and the "not done by AKAR" block remain further down. No execution was added.

### `akar mission` (`src/mission.rs::format_mission_report`)

The report now opens with an `ADVISORY ONLY` banner stating `akar mission` walks the state machine in scaffold mode and does NOT: execute code, edit files, call models, run the mission — followed by "For a Claude-ready next-run prompt, use `akar request`." The state header (`Done.` / `Failed.` / `Blocked.`) follows the banner. The scaffold "Verified: scaffold mode (commands not executed)" and "Not verified: actual execution" sections remain. No execution was added.

Both banners were locked with unit-test assertions (`format_workflow_report_contains_key_sections`, `format_mission_report_contains_expected_sections`, `format_mission_report_shows_failed_when_state_failed`).

---

## 4. Docs corrected

### `docs/INSTALL.md`
- **Version string:** `Expected: akar 0.10.0` → `Expected: akar <current version>` (e.g. `akar 0.22.0`), with a note that the version always matches `Cargo.toml`.
- **Stale tense:** "No TOML/JSON config file for v0.2.x." → "No TOML/JSON config file."
- **Version-compatibility table:** extended past v0.9.x through v0.22.x, noting the template format has not changed since v1.

### `docs/OPERATING_MODEL.md`
- **"What AKAR is":** rewritten to state AKAR is **advisory-only**, "reports diff budgets" (not "enforces"), and "does not execute the task, edit project files, call models, or run missions."
- **Section E (scaffold mode):** removed the obsolete "In v0.2.x ... Real execution is planned for v0.3+" tense; now states scaffold mode is **by design (v1 architecture freeze)**, not a version lag.
- **Section F (passive runtime):** corrected the overclaim "STATE.md is updated after each session" → "STATE.md is a template the user edits; AKAR does not auto-update it after a session."
- **Section G (slash commands):** the table now lists only the six `.claude/commands/akar-*.md` files that actually exist (`akar-bootstrap`, `akar-doctor`, `akar-eval`, `akar-mission`, `akar-status`, `akar-verify`). Added an explicit note that `/akar-preflight` and `/akar-doctor-fix` are sometimes referenced in older notes but **do not exist as command files**.
- **Section G (hooks):** replaced the obsolete pre-commit-hook section with the **proven PreToolUse hook** (`templates/hooks/pre-tool-call.{sh,ps1}`), including `akar hooks --check` / `akar hooks --install` guidance and the PATH-failure-mode warning. The older `hooks/pre-commit-akar.{sh,ps1}` is noted as a reference design, not the current integration.
- **"First commands after init":** labeled `akar run` as advisory (prints strategy + records telemetry; does not execute).
- **Expected output format:** replaced the overclaiming "changed: file.rs / verified: cargo build: ok" mission example with the actual advisory banner + scaffold report shape, and noted the old shape described a future execution-capable mission, not current AKAR.

### `AKAR_MASTER_ROADMAP_v1.0_REVISED.md`
- Added a prominent **SUPERSEDED** banner at the top stating the roadmap describes an execution-capable autonomy runtime that was never built and is forbidden by the v1 architecture freeze; pointing to the freeze proposal and the v0.21 audit as the source of truth; and warning future sessions not to infer authorization to build the execution engine, model routing, or autonomy modes.
- Changed `**Status:**` from "Revised source of truth" to "SUPERSEDED by the v1 architecture freeze and the v0.21 current-reality audit. Retained as historical context only."

### `docs/architecture/PRODUCT_ROADMAP.md`
- Added a **SUPERSEDED** banner noting the "Current" version tag is stale (AKAR is at v0.22.0) and that the progression describes an execution-capable runtime that was never built; pointing to the freeze and the v0.21 audit.

### `README.md`
- **Command table:** added the missing rows `request --check`, `learn --list`, `learn --resolve`, `hooks --check`, `governor`. Labeled `run` as "Advisory scaffold only ... does not execute, edit files, call models, or run the mission" and `mission` as "Advisory/report-only scaffold ... no code executed." Labeled `request` as writing compiled NEXT_RUN.md, `request --check` as read-only, `governor` as advisory (does not write files or mutate git), `verify` as the one command that runs cargo/npm, `safety` as exiting 2 for BLOCKED, `calibrate` as display only, and `skills` as report only.
- **Example output:** `akar 0.3.0` → `akar 0.22.0`.
- **Stale version refs:** removed "AKAR v0.12.0 carries...", "AKAR v0.13.0 uses...", "AKAR v0.14.0 gives..." version prefixes (now version-neutral).
- **Docs links:** the Roadmap link now points to the architecture freeze as the source of truth, with PRODUCT_ROADMAP marked "(superseded — historical)."

---

## 5. Dead code removed

Removed only code that was confirmed dead (no runtime caller) and whose removal was isolated. Compiler and tests confirm each removal.

| Item | Location | Evidence | Action |
|---|---|---|---|
| `SafeFix::NormalizePath` | `src/safe_fix.rs` | Never constructed anywhere; match arm was a no-op returning `"ok"`. Only a unit test referenced it. | Removed variant, match arm, and `normalize_path_returns_ok` test. |
| `request_intelligence::write_next_run` | `src/request_intelligence.rs` | Shadowed by `write_governor_next_run` in `cmd_request` (v0.21 audit §7c.3); its "never overwrite" guard was moot and its output always discarded. | Removed the function entirely. `cmd_request` now writes NEXT_RUN.md exactly once via the compiled-prompt writer. |
| `RequestAdvisory::next_run_recommended` | `src/request_intelligence.rs` | Set from `matches!(mode, Resume)` but never read in runtime after the shadowed writer was removed. | Removed the field; updated `resume_mode_at_95_percent` test to assert the Resume strategy surfaces `akar request`. |
| `skill_registry::format_registry` | `src/skill_registry.rs` | `#[allow(dead_code)]`; `cmd_skills` uses `format_skill_report` instead. Test-only. | Removed function and its 3 tests. |
| `skill_registry::detect_duplicates` | `src/skill_registry.rs` | `#[allow(dead_code)]`; never called from runtime. Test-only. | Removed function and its 2 tests. |
| `diff_budget::BudgetVerdict::as_str` | `src/diff_budget.rs` | `#[allow(dead_code)]`; never called anywhere. | Removed the `impl` block. |
| Stale `#[allow(dead_code)]` on live functions | `src/foundation.rs` | `snapshot_required_playbook` and `repeated_block_playbook` are called by `loop_governor.rs` at runtime. | Removed the two misleading `#[allow(dead_code)]` annotations. |
| Unused `Path` import | `src/request_intelligence.rs` | Was only used by the removed `write_next_run`. | Removed import (and its `#[allow(unused_imports)]`). |
| Unused `event_log` import | `src/request_intelligence.rs` | Was only used by the removed `write_next_run`. | Removed from the `use` list. |

After removing `detect_duplicates`, the compiler surfaced that `SkillEntry.purpose` was write-only (its only runtime reader was `detect_duplicates`). Rather than remove the field (a broad refactor touching `collect_skills`, the test helper, and ~15 `make_skill` call sites — which the hard rules forbid), it was marked `#[allow(dead_code)]` with a comment explaining it is collected for future skill-conflict-on-purpose analysis but not currently read. This matches the v0.21 audit's recommended pattern ("gate behind `#[allow(dead_code)]` with a comment").

Test count went from 401 → 395 (six dead-code-only tests removed: 1 `NormalizePath`, 2 `detect_duplicates`, 3 `format_registry`). All remaining tests pass.

---

## 6. Drift items fixed

1. **Shadowed NEXT_RUN writer** (v0.21 §7c.3) — fixed. `cmd_request` now has a single write path. See §5.
2. **Second hardcoded diff-budget table** (v0.21 §7c.1) — fixed. The four budget caps are now `pub const BUDGET_CAP_*` in `contract.rs`; both `DiffBudget::micro/small/medium/large` and `diff_budget::budget_for_task_name` read from them. The "no second budget table" comment is now accurate. Behavior is byte-identical (same caps: 3/60, 5/200, 12/600, 30/2000).
3. **Stale `#[allow(dead_code)]` on live `foundation.rs` functions** (v0.21 §7c.2) — fixed. See §5.
4. **Stale `preflight.rs` version comment** (`needs_execution = false` "not recommended automatically in v0.1.9") — noted. The code is correct; the version string is stale. Left in place (cosmetic; touching it adds risk without honesty benefit, and the audit listed it as observational, not a delete candidate).
5. **Stale docs** (v0.21 §8) — fixed. See §4.

---

## 7. Drift items postponed

The following were identified by the v0.21 audit but **deliberately postponed** because removing them is a broad refactor that the v0.22 hard rules forbid ("Do not do broad refactors"), and the audit's own recommendation allows gating them behind `#[allow(dead_code)]` with a comment rather than deletion:

1. **Dead `contract` enum variants.** `Autonomy::{A0,A1,A2,A3,A4,A6}` (autonomy is always A5), `CostMode::{Fast,Deep,Autopilot,Emergency}` (always Balanced), `Confidence::{Low,High}` (always Medium), and the never-produced `TaskType` variants (`Research`, `Answer`, `Inspect`, `Greenfield`, `Repair`, `Release`). These are referenced across multiple `match` arms in `infer_goal`, `default_verification`, and tests. Removing them requires editing every match arm and constructor — a broad refactor. They remain gated behind the existing `#[allow(dead_code)]` on the enums. **Postponed to a future honest-cleanup release that can scope a full enum audit.**
2. **Dead `skill_registry` enum variants.** `SkillSource::ClaudeBundled`, non-Active `SkillStatus` (`Wrapped`, `Disabled`, `Replaced`, `Testing`), and `SkillRole::LibraryOnly`. These appear in live `match` arms in `build_skill_report` and `write_skill_inventory` (not just the removed `format_registry`). Removing them requires editing those match arms and is a broader refactor. **Postponed.**
3. **`doctor.rs` stub.** Still a directory-existence check. Building the real Phase 5 doctor (hook health, EVENT_LOG.jsonl parseability, template presence, `.akar/` writeability) is a feature-sized task explicitly scoped to **v0.23.0 Real Doctor** by the v0.21 audit. Not in scope for v0.22.0 (no new features).
4. **Self-fulfilling evals.** Six evals (`vague_prompt_contract`, `context_pack_build`, `design_check`, `doctor_check`, `request_pressure_compaction`, `no_all_skills_mode`) inflate the 28/28 headline. Rewriting them into meaningful behavior checks is an eval-honesty task best paired with the v0.23 doctor work. **Postponed.** The 28/28 count is unchanged; the v0.21 audit already documents which six are smoke checks.
5. **`context_pack.rs` / `design.rs` weak wiring.** Both exist primarily to satisfy a single eval. Not dead, but load-bearing only for the eval count. **Postponed** (touching them risks the eval suite and is out of scope for an honesty-cleanup release).
6. **`model_profile.rs` vestigial fields.** `gateway` always "unknown", `last_calibrated` always "never", `known_failure_patterns` always empty. **Postponed** (cosmetic; removing fields is a refactor; the v0.21 audit listed them as observational).

None of the postponed items change runtime behavior or endanger a user. They are documented here so they are not silently inherited as future work.

---

## 8. Runtime behavior impact

**No runtime behavior changed in a way that affects user-facing outcomes**, with two intentional, honesty-only exceptions:

1. **`akar run` and `akar mission` output now begins with an advisory banner.** The banner is additive text; the existing report content (state, mission, preflight, postmortem, telemetry) is unchanged. No command that previously did work now does less; no command that previously did nothing now does more.
2. **`akar request` no longer calls the shadowed `write_next_run`.** This changes no user-visible outcome: the compiled 11-section NEXT_RUN.md (written by `write_governor_next_run`) was already the file that survived, so the net content of `.akar/NEXT_RUN.md` after `akar request` is identical. The only difference is that one redundant write (whose output was always discarded) no longer happens.

Specifically unchanged:
- `akar request --check` remains read-only (exit 0 on PASS, non-zero on FAIL).
- `akar governor` still does not write NEXT_RUN.md.
- Governor decision rules, exit-code mapping, and telemetry behavior are unchanged.
- NEXT_RUN compiler and validator behavior are unchanged.
- Safety classification and hook behavior are unchanged.
- Mission state machine, preflight, postmortem, learn lifecycle, diff budget values, and skill scanning are unchanged.
- The v1 architecture freeze holds: no model API, no daemon, no cloud telemetry, no DB, no auto-apply, no skill enforcement, no settings.json mutation, no code execution in the mission path.

`akar verify` remains the only command that runs non-git subprocesses (cargo/npm), user-invoked, as before.

---

## 9. Verification

Run after all edits, before the version bump:

| Command | Result |
|---|---|
| `cargo build --release` | clean, **zero warnings** |
| `cargo test` | **395 passed; 0 failed; 0 ignored** |
| `cargo run -- --version` | `akar 0.21.0` (pre-bump; bumped to 0.22.0 after verification) |
| `cargo run -- run "test advisory banner"` | opens with `ADVISORY ONLY` banner listing the four "does NOT" items + `akar request` |
| `cargo run -- mission "test advisory banner"` | opens with `ADVISORY ONLY` banner listing the four "does NOT" items + `akar request` |
| `cargo run -- request` | writes `.akar/NEXT_RUN.md` once (compiled 11-section prompt); prints governor block |
| `cargo run -- request --check` | `NEXT_RUN check: PASS` |
| `cargo run -- governor --json --no-exit-code` | valid JSON, exit 0 |
| `cargo run -- status` | `status: HEALTHY` |
| `cargo run -- learn --list` | prints active/resolved counts |
| `cargo run -- doctor` | `doctor: OK` |
| `cargo run -- eval` | **28/28 PASS**, `overall: PASS` |
| `cargo run -- hooks --check` | `status: PASS`, both templates found |

Release build is zero-warning. Test count is 395 (down from 401 by exactly the six dead-code tests removed). Eval count is unchanged at 28/28.

---

## 10. Dogfood readiness after cleanup

AKAR v0.22.0 is **safe to dogfood in advisory mode on an external repo** with the PreToolUse hook active. The v0.21 audit's #1 dogfood risk — the `akar run` expectation gap (command name implies action, behavior is report-only) — is now mitigated: both `akar run` and `akar mission` lead with an explicit `ADVISORY ONLY` banner stating they do not execute code, edit files, call models, or run the mission. A user who types `akar run "fix the bug"` and reads the first line can no longer reasonably expect the bug to be fixed.

The remaining dogfood caveats from v0.21 stand:
- `akar doctor` still only checks directory existence (v0.23 will address this).
- The hook fails open if `akar` is not on the subprocess PATH — now documented in `OPERATING_MODEL.md` and `akar hooks` output guidance.
- Six evals are smoke checks (documented in the v0.21 audit §9c); the 28/28 headline is unchanged but the honest meaningful count is ~22.

---

## 11. Honest conclusion

AKAR v0.22.0 is **more honest than v0.21.0, with no scope added.** The advisory boundary is now unmistakable in the two commands where it mattered most (`run`, `mission`). The shadowed NEXT_RUN writer — a maintainer trap inside a live command path — is gone. The duplicate budget table — a drift risk — is centralized. The eval-only policy functions are clearly labeled as a policy library, not a runtime governor. Confirmed dead code is deleted. Stale docs (INSTALL, OPERATING_MODEL, master roadmap, PRODUCT_ROADMAP, README) now describe the actual advisory-only CLI, not an execution-capable OS that was never built.

What v0.22.0 did **not** do is equally important: it added no new commands, no execution, no auto-run, no auto-apply, no model API, no daemon, no DB, no skill enforcement, no settings.json mutation. It changed no governor decision rule, no exit-code mapping, no telemetry behavior, no NEXT_RUN compiler/validator behavior, no safety classification, no hook behavior. The v1 architecture freeze holds completely.

The dead enum variants, the stub doctor, the self-fulfilling evals, and the weakly-wired `context_pack`/`design` modules are documented as postponed — not hidden, not inherited as silent commitments. They are the honest agenda for v0.23.0 and beyond.

---

## 12. Next recommended release

**v0.23.0 — Real Doctor.** Build the Phase 5 doctor honestly: check hook template presence, EVENT_LOG.jsonl parseability, template-file presence, `.akar/` writeability. Still read-only. This is the one stub that most undercuts the "AKAR diagnoses itself" claim. Pair with rewriting the six self-fulfilling evals into meaningful behavior checks so the 28/28 headline reflects real coverage.

**v0.24.0 — Honest Enums.** A scoped release to delete the dead `contract` and `skill_registry` enum variants (A0–A4/A6, Fast/Deep/Autopilot/Emergency, Low/High confidence, unused TaskType, ClaudeBundled, non-Active SkillStatus, LibraryOnly) and trim `model_profile.rs` vestigial fields. This is the broad-refactor work v0.22.0 deliberately deferred.

**v1.0 design review** — only after v0.23 + v0.24 + the external-repo dogfood trials. The review decides whether v1.0 stays advisory-only (recommended) or authorizes a bounded execution path.

*End of report. v0.22.0 Honest Edges. No new features. No execution. Advisory-only, frozen, honest.*

