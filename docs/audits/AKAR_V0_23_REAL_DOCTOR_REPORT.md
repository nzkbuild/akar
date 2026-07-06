# AKAR v0.23.0 — Real Doctor Report

**Date:** 2026-07-06
**Scope:** replace the stub doctor (directory-existence check only) with honest read-only environment checks that verify AKAR is ready for advisory dogfood, without modifying files or configuration. Rewrite the six self-fulfilling evals identified by the v0.21/v0.22 audit. No new commands, no execution, no auto-run, no auto-fix beyond pre-existing safe directory creation.
**Method:** read the v0.21 audit (§7b stub doctor, §9c self-fulfilling evals), the v0.22 report, and the affected modules; rewrote `doctor.rs` with a sectioned `DoctorReport`; rewired `cmd_doctor`/`cmd_status`; rewrote the six evals; rebuilt and ran the full verification matrix after each change.

---

## 1. Baseline

Confirmed in Phase 0 (no files modified before verification):

| Check | Result |
|---|---|
| `git log --oneline -5` | HEAD = `f8525f1 chore: sharpen AKAR honest edges` |
| `git status` | working tree clean; 17 commits ahead of origin/master (unpushed) |
| `cargo run -- --version` | `akar 0.22.0` |
| `cargo test` | **395 passed; 0 failed; 0 ignored** |
| `cargo run -- eval` | **28/28 PASS** |
| `cargo run -- doctor` | `doctor: OK` (stub — directory check only) |
| `cargo run -- status` | `status: HEALTHY` |
| `cargo run -- request` / `request --check` | writes NEXT_RUN.md / `NEXT_RUN check: PASS` |
| `cargo run -- hooks --check` | `status: PASS`, both templates found |

Baseline is v0.22.0. The doctor was the stub flagged by the v0.21 audit.

---

## 2. Doctor checks implemented

`akar doctor` now runs a sectioned set of read-only checks and prints a `DoctorReport`:

- **environment**
  - project root detected
  - `.akar/` directory exists (WARN if missing — bootstrap can create it)
  - `.akar/` writable if it exists (write-probe temp file, immediately deleted)
  - `akar` binary visible on PATH (best-effort via `where`/`which`, with a fallback to `current_exe` and an explicit warning that the hook fails open if `akar` is not on the subprocess PATH)
- **files**
  - `NEXT_RUN.md` present (WARN if missing)
  - `DIFF_BASELINE.json` present and parseable (WARN if missing; FAIL if present but unreadable)
  - `LEARNING_PATCHES.md` summary (total/active/resolved/active-split-rule; WARN if an active split-rule entry may force SPLIT_TASK)
- **hooks**
  - `templates/hooks/pre-tool-call.sh` and `.ps1` exist and are valid (the internal `hooks::check_hooks` equivalent — checks for `akar safety`, stdin reading, `HOOK_EVENTS.jsonl`, `exit 2`). FAIL if missing or invalid.
- **telemetry**
  - `EVENT_LOG.jsonl` structural-JSON parseability (each non-empty line must be a balanced `{...}` object). FAIL if malformed.
  - `HOOK_EVENTS.jsonl` structural-JSON parseability. FAIL if malformed.
- **git**
  - git repository detected (FAIL if not)
  - working tree clean/dirty (WARN if dirty)
  - git HEAD (informational)
  - `Cargo.toml` present (WARN if absent — `akar verify` falls back to npm)
- **next-run**
  - `NEXT_RUN.md` passes the existing `loop_governor::validate_next_run` request contract if present (FAIL if invalid; WARN if missing)
- **recommendations**
  - advisory list: `FIX:` for failures, `advisory:` for warnings

The JSONL parseability check is a **structural** validator (balanced braces/quotes, starts with `{`, ends with `}`), not a full schema parse — AKAR has no JSON dependency. It catches the corruption cases that matter for dogfood (truncated writes, concatenated lines, non-JSON content). This is documented in the doctor module doc and the report.

---

## 3. OK/WARN/FAIL semantics

- **OK** — no failed checks. Advisory dogfood can proceed.
- **WARN** — dogfood is possible but something advisory is missing: no baseline snapshot, no `NEXT_RUN.md`, dirty working tree, an active split-rule learning patch, no `Cargo.toml`, or `akar` not confirmed on PATH. No check that gates safety failed.
- **FAIL** — dogfood should stop: invalid `NEXT_RUN.md`, missing/invalid hook templates, malformed `EVENT_LOG.jsonl` or `HOOK_EVENTS.jsonl`, unreadable `DIFF_BASELINE.json`, or no git repository.

The overall status is `Fail` if any check is `Fail`; else `Warn` if any check is `Warn`; else `Ok`.

`akar status` now uses the report status: HEALTHY unless the doctor is `Fail` (Warn is advisory and no longer flips status to DEGRADED).

---

## 4. Read-only guarantees

`run_doctor_report` and `run_checks` never:

- create `.akar/` or any directory,
- write or rewrite `NEXT_RUN.md` (it validates an existing file only),
- resolve learning patches,
- install hooks or modify `~/.claude/settings.json`,
- mutate git,
- delete or truncate logs,
- auto-fix malformed files.

The only non-read-only path is `akar doctor --fix`, which is limited to pre-existing safe directory creation (see §5). The write-probe for `.akar/` writability creates a temp file and deletes it immediately; it never leaves a file behind.

Tests `doctor_does_not_create_files` and `doctor_does_not_modify_next_run` lock these guarantees.

---

## 5. doctor --fix behavior

`akar doctor --fix` is intentionally limited. It can apply only the pre-existing safe directory creation (via `safe_fix::SafeFix::CreateMissingDir`) — for example, creating a missing `.akar/` directory.

It does **not**:

- modify Claude Code settings (`~/.claude/settings.json`),
- install hooks,
- mutate git,
- rewrite `NEXT_RUN.md`,
- resolve learning patches,
- delete or truncate logs,
- auto-fix malformed files.

Dogfood-critical checks (invalid `NEXT_RUN.md`, malformed telemetry logs, missing hook templates, no git repo) have no `FixHint` — `doctor --fix` prints `skip: no auto-fix for: ... (requires human action)` for each, and the report's recommendations state what to do. The `--fix` summary line ends with `(no Claude settings/hooks/git changed)`.

The `FixHint::CreateFromTemplate` variant was removed (the doctor no longer offers auto-creating hook templates — missing templates are a FAIL requiring human action). `SafeFix::CreateMissingTemplate` is retained (tested, correct) but marked `#[allow(dead_code)]` with an explanatory comment, since no runtime path constructs it as of v0.23.

---

## 6. Eval honesty changes

Rewrote the six self-fulfilling evals flagged by the v0.21/v0.22 audit:

| Eval | Before (self-fulfilling) | After (meaningful) |
|---|---|---|
| `vague_prompt_contract` | `passed = true` (classify always returns a contract) | Asserts a vague prompt yields a **complete** contract: non-zero diff budget, a recognized `TaskType`, A5 autonomy. |
| `doctor_check` | `passed = true` (no panic) | Asserts the doctor runs real checks, has all six sections, has a valid OK/WARN/FAIL status, and does **not FAIL** on the real repo. |
| `context_pack_build` | `passed = total_files >= 0` (always true) | Asserts `total_files == files.len()` and every listed file exists on disk. |
| `design_check` | `passed = true` (no panic) | Asserts `has_design_dna` matches actual `DESIGN_DNA.md` presence and issues are consistent. |
| `request_pressure_compaction` | `passed = "compact" != "stop"` (tautology) | Asserts `build_advisory` at 70% pressure returns `PressureMode::Compact` with a compaction-mentioning strategy. |
| `no_all_skills_mode` | passed because nonexistent dir returns empty (trivial) | Relabeled `no_all_skills_mode_smoke` with an honest detail string stating it is a regression smoke check. |

The eval count stays 28/28. The eval module doc now notes that `_smoke`-labelled evals are regression smoke checks, not behavior proofs. No eval meaning was inflated — five became real behavior checks and one was honestly relabeled.

---

## 7. Tests added/updated

Added 16 doctor tests (395 → 411 total):

- `missing_akar_dir_is_warn_not_fail`
- `doctor_does_not_create_files`
- `doctor_does_not_modify_next_run`
- `missing_next_run_is_warn_not_fail`
- `invalid_next_run_is_fail`
- `valid_next_run_passes`
- `missing_baseline_is_warn_not_fail`
- `malformed_event_log_is_fail`
- `malformed_hook_events_is_fail`
- `valid_event_log_passes`
- `missing_hook_template_is_fail`
- `dirty_git_tree_is_reported_as_warn`
- `no_git_repo_is_fail`
- `ok_when_everything_present_and_valid`
- `validate_json_object_accepts_valid` / `validate_json_object_rejects_malformed`
- `format_doctor_report_has_sections`

Updated `run_checks_returns_issues_when_dirs_missing` for the new semantics (removed the obsolete `run_checks_returns_no_issues_when_dirs_exist` — the new doctor legitimately produces WARNs for missing NEXT_RUN/baseline even when dirs exist, so that assertion no longer holds).

Updated the six eval tests indirectly via the rewritten eval blocks; `run_evals_returns_28_results` still passes.

Removed dead code surfaced by the rewrite: `Severity::Info`, `FixHint::CreateFromTemplate`; marked `SafeFix::CreateMissingTemplate` and `config::Config::validate` `#[allow(dead_code)]` with comments.

---

## 8. Verification

Run after all edits, before the version bump:

| Command | Result |
|---|---|
| `cargo build --release` | clean, **zero warnings** |
| `cargo test` | **411 passed; 0 failed; 0 ignored** |
| `cargo run -- --version` | `akar 0.22.0` (pre-bump; bumped to 0.23.0 after verification) |
| `cargo run -- doctor` | `doctor: WARN` (dirty tree + active split-rule; sectioned report with environment/files/hooks/telemetry/git/next-run/recommendations) |
| `cargo run -- doctor --fix` | `0 fixed, 0 failed, 2 skipped (no Claude settings/hooks/git changed)` |
| `cargo run -- status` | `status: HEALTHY` (Warn does not flip to DEGRADED) |
| `cargo run -- request` | writes `.akar/NEXT_RUN.md` once |
| `cargo run -- request --check` | `NEXT_RUN check: PASS` |
| `cargo run -- governor --json --no-exit-code` | valid JSON, exit 0 |
| `cargo run -- learn --list` | prints active/resolved counts |
| `cargo run -- eval` | **28/28 PASS**, `overall: PASS` |
| `cargo run -- hooks --check` | `status: PASS`, both templates found |

Release build is zero-warning. Test count is 411 (+16 doctor tests). Eval count is unchanged at 28/28.

---

## 9. Dogfood readiness after doctor

The doctor is no longer the stub that "most undercuts the 'AKAR diagnoses itself' claim" (v0.21 audit §7b). `akar doctor` now genuinely diagnoses the environment: it detects a missing `.akar/`, an unwritable `.akar/`, an invalid `NEXT_RUN.md`, malformed telemetry logs, missing hook templates, a non-repo, and a dirty tree — and tells the user exactly what to do via the recommendations section, without modifying anything.

The remaining dogfood caveats from v0.21/v0.22:
- The JSONL parseability check is structural, not a full schema parse. It catches truncation/concatenation/non-JSON but not schema-level corruption. Honest and documented.
- `akar`-on-PATH detection is best-effort (`where`/`which` with a `current_exe` fallback). The doctor warns explicitly when it cannot confirm PATH visibility, because the hook fails open in that case.
- The six self-fulfilling evals are now meaningful (five real behavior checks, one honest smoke label).

---

## 10. Honest conclusion

AKAR v0.23.0 delivers the **Real Doctor** the v0.21 audit recommended. The doctor is now an honest read-only diagnostic: it checks the environment, runtime files, hook templates, telemetry parseability, git state, and next-run validity, and reports OK/WARN/FAIL with actionable recommendations — without ever modifying files, settings, hooks, or git. `doctor --fix` is deliberately narrow (safe directory creation only) and explicitly refuses to auto-fix dogfood-critical checks, which require human action.

The six self-fulfilling evals are gone. Five are now real behavior checks that assert actual contracts (complete classification, consistent context packs, design-DNA reflection, real pressure-mode output, real doctor sections). One is honestly relabeled as a smoke check. The 28/28 headline is now a more honest reflection of coverage.

What v0.23.0 did **not** do: no new commands, no execution, no auto-run, no auto-apply beyond pre-existing safe directory creation, no model API, no daemon, no DB, no skill enforcement, no settings.json mutation, no changes to governor decision rules/exit codes/telemetry, no changes to NEXT_RUN compiler/validator, no changes to safety classification or hook behavior. The v1 architecture freeze holds completely.

---

## 11. Next recommended release

**v0.24.0 — Honest Enums.** A scoped release to delete the dead `contract` and `skill_registry` enum variants (A0–A4/A6, Fast/Deep/Autopilot/Emergency, Low/High confidence, unused TaskType, ClaudeBundled, non-Active SkillStatus, LibraryOnly) and trim `model_profile` vestigial fields. This is the broad-refactor work v0.22 deliberately deferred. Pair with reviewing `context_pack`/`design` wiring now that their evals are meaningful.

**v1.0 design review** — only after v0.24 + the external-repo dogfood trials (now that the doctor can actually diagnose external-repo readiness). The review decides whether v1.0 stays advisory-only (recommended) or authorizes a bounded execution path.

*End of report. v0.23.0 Real Doctor. No new features. No execution. Advisory-only, frozen, honest.*