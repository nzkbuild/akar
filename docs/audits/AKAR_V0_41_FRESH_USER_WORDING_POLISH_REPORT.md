# AKAR v0.41.0 — Fresh-User Wording Polish Audit Report

## 1. Executive Verdict

**PASS.** Three cosmetic wording changes address first-run friction that had
persisted across multiple dogfood trials. No new AKAR capabilities, no behavior
changes, no hook modifications, no governor changes.

## 2. AKAR Baseline

- **Version:** 0.40.0
- **Commit:** 6120fff
- **Tests:** 508 passed, 0 failed
- **Evals:** 28/28 PASS
- **Working tree:** clean

## 3. Why This Matters

Five external dogfood trials (v0.24, v0.27, v0.33, v0.39, v0.40) all reported
the same three friction points:

1. "templates directory not found" on init — sounds like failure even when
   embedded fallback works fine.
2. `.akar/` creation dirties the tree with no guidance — users are left to
   discover this on their own.
3. Doctor WARNs on missing NEXT_RUN/baseline on fresh projects — alarms new
   users who think something is actually broken.

These are cosmetic but they're the first thing a new user sees. Fixing them
removes the "am I doing something wrong?" feeling from first-run experience.

## 4. Change 1: Bootstrap Template Discovery Wording

**Before:** "templates directory not found"
**After:** "source template directory not present in this repo; this is normal
for installed AKAR. Embedded hook templates remain available via 'akar hooks
--install'."

**File:** `src/bootstrap.rs:77` — the warning string in `find_templates_dir`
fallback path.

**Tests updated:** 2 (format_bootstrap_report_distinguishes_..., 
format_bootstrap_report_shows_warnings_when_present)

## 5. Change 2: Init .akar/ Dirty-Tree Guidance

**New output section** after bootstrap/doctor and before "next steps" in
`format_init_report`:

```
.akar/ notice:
  .akar/ contains local AKAR runtime state. Inspect 'git status'.
  Intentionally add .akar/ to .gitignore or commit only files you
  want tracked. AKAR will not decide for you.
  Do not use destructive cleanup blindly.
```

**File:** `src/init.rs:157-162` — inserted before "next steps:" section.

**Tests updated:** 1 (format_fresh_init — asserts ".akar/ notice" appears)

**Hard rules honored:** No auto-write to .gitignore, no auto-commit, no
auto-delete of .akar/, no destructive cleanup.

## 6. Change 3: Fresh-Project Doctor Severity

**Three check sites changed from WARN to PASS:**

| Check | Before | After |
|-------|--------|-------|
| `files: NEXT_RUN.md present` | WARN "missing — run 'akar request'..." | PASS "not generated yet — run 'akar request \"<task>\"' when ready" |
| `files: DIFF_BASELINE.json` | WARN "missing — run 'akar preflight..." | PASS "no snapshot yet — run 'akar preflight --snapshot \"<task>\"' before a measured session" |
| `next-run: NEXT_RUN.md valid` | WARN "missing — run 'akar request'..." | PASS "not generated yet — run 'akar request \"<task>\"' when ready" |

**File:** `src/doctor.rs` — `check_files` (lines 358-361, 378-381) and
`check_next_run` (lines 616-619).

**Tests updated:** 2 renamed and re-asserted:
- `missing_next_run_is_warn_not_fail` → `missing_next_run_is_pass_not_warn`
- `missing_baseline_is_warn_not_fail` → `missing_baseline_is_pass_not_warn`

**Severity rules preserved:**
- Invalid NEXT_RUN.md → still FAIL (invalid_next_run_is_fail unchanged)
- Malformed DIFF_BASELINE.json → still FAIL
- Unknown project kind → still WARN (project_kind_unknown_is_warn_and_labeled_project_kind unchanged)
- Dirty working tree → still WARN (dirty_git_tree_is_reported_as_warn unchanged)
- No git repo → still FAIL (no_git_repo_is_fail unchanged)

## 7. Fresh-Repo Verification

### Fresh temp directory (no git, no project markers)

```
doctor: FAIL (git repo not detected)
  files: NEXT_RUN.md present → PASS "not generated yet..."
  files: DIFF_BASELINE.json → PASS "no snapshot yet..."
  next-run: NEXT_RUN.md valid → PASS "not generated yet..."
  git: project kind → WARN "Unknown"
  verification hints → WARN "no confident verification command discovered"
```

The FAIL comes from no git repo (correct — unchanged behavior). The three
changed checks all show PASS with fresh-project wording. The WARNs for
Unknown project kind and verification hints remain (correct).

### Init output (fresh git repo)

```
init: shell=PowerShell
bootstrap: 0 created, 0 skipped
warnings:
  - source template directory not present in this repo; this is normal...
doctor: issues remain
  - project kind — Unknown...
  - verification hints — no confident verification command discovered
.akar/ notice:
  .akar/ contains local AKAR runtime state. Inspect 'git status'.
  ...
next steps:
  akar status ...
```

New wording confirmed. `.akar/ notice` section appears. Template warning is
explanatory, not alarming.

## 8. AKAR Self-Repo Verification

- `cargo test`: 508 passed, 0 failed
- `cargo build --release`: success (2 pre-existing warnings only)
- `akar --version`: "akar 0.41.0"
- `akar eval`: 28/28 PASS
- `akar doctor`: WARN (LEARNING_PATCHES split-rule, dirty tree — both expected)
- `akar status`: HEALTHY
- `akar request --check`: PASS (4/4 checks)
- `akar hooks --check`: PASS (source-tree)
- `akar learn --list`: 8 entries (1 active — expected)

## 9. What This Release Does NOT Change

Per hard rules: no new product features, no refactored internals, no hook
behavior changes, no hook template changes, no safety classification changes,
no governor decision rule changes, no governor exit code changes, no governor
telemetry changes, no project detection rule changes, no verification discovery
rule changes, no NEXT_RUN compiler behavior changes (except wording), no
`request --check` contract changes, no `akar verify` broadening, no mission
execution, no auto-run, no auto-apply, no Claude Code settings modifications,
no model/API calls, no OpenCode/Codex/multi-agent support, no token
optimization/cache features.

## 10. Honest Assessment

Three cosmetic frictions that appeared in every external dogfood report since
v0.24 are now addressed. The changes are limited to output strings and
severity tags — zero risk of runtime regression. The biggest subjective
improvement is the doctor fresh-project framing: a new user who runs `akar init`
then `akar doctor` now sees PASS for NEXT_RUN and baseline checks instead of
WARN, with wording that tells them what to do next rather than implying
something is broken.

The `.akar/ notice` in init output does not prevent the dirty-tree issue — the
user still has to act on it — but it makes the expectation explicit so they're
not surprised by a dirty tree on the next `akar status`.

## 11. Next Recommended Release

**v0.42.0 or v1.0.0 RC.** With all five dogfooded project-kind lanes (Rust,
Node, Python, Unknown single-task, Node multi-task) and first-run wording
polished, the advisory loop is ready for release-candidate review.
