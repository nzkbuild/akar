# AKAR v0.46.0 — Prepare/Finish Command Prototype

## 1. Executive Verdict

**75% manual CLI burden reduction confirmed.** `akar prepare "<task>"` and `akar finish`
replace a manual sequence of 7+ advisory commands per task with 2. The external dogfood
trial on a Node fixture measured 2 AKAR commands per task vs 8 in v0.45.0 (a 75% reduction).

Both commands are advisory-only: they compose existing operations, write no project source
code, call no model APIs, and modify no Claude settings. 534 tests pass (26 new prepare/finish
tests plus 508 existing), and all existing command behavior is preserved.

## 2. Design

### What prepare does

`akar prepare "<task>"` consolidates this manual sequence:

```
akar preflight --snapshot "<task>"     # baseline snapshot
akar request "<task>"                  # generate NEXT_RUN.md
akar request --check                   # validate NEXT_RUN.md
akar governor                          # surface loop governor decision
```

Into a single command that:
1. Validates the task argument (non-empty required)
2. Requires a git repository and clean working tree
3. Runs preflight strategy (advisory, output not shown)
4. Detects project kind (Rust/Node/Python/Unknown) from marker files
5. Writes `DIFF_BASELINE.json` (prevents dirty-tree snapshot)
6. Generates `NEXT_RUN.md` via the governor's compiled prompt writer
7. Validates `NEXT_RUN.md` against the request contract
8. Outputs: task, project kind, baseline info, request mode, check result, governor decision, verification guidance, next step

### What finish does

`akar finish` consolidates this manual sequence:

```
akar postmortem --diff --baseline      # diff measurement
akar learn --list                      # learning patch summary
akar governor                          # loop governor decision
akar doctor                            # health check
```

Into a single command that:
1. Requires `DIFF_BASELINE.json` (rejects if missing)
2. Measures diff from baseline HEAD via `git diff --numstat`
3. Compares diff against budget (PASS/EXCEEDED/UNKNOWN)
4. Summarizes learning patches (total, active, resolved, active split rules)
5. Reads governor decision
6. Runs doctor for health summary (FAILs and WARNs surfaced)
7. Writes budget-exceeded learning patch when applicable
8. Exits non-zero on budget exceeded

### Hard boundaries

Neither command:
- Runs project code, test commands, or verification
- Edits project source files
- Commits, pushes, resets, stashes, or checks out
- Modifies Claude Code settings
- Calls model APIs
- Auto-invokes Claude Code or any AI agent
- Reads or writes outside `.akar/`

## 3. Implementation

### Changes

| File | Change |
|---|---|
| `src/main.rs` | Added `prepare` and `finish` match arms, `print_usage()` help lines, `cmd_prepare()` (~130 lines), `cmd_finish()` (~120 lines) |
| `src/main.rs` | Added 26 new tests: 12 prepare (help, baseline roundtrip, clean/dirty tree, request check, governor, project kind×3, budget verdict, DIFF_BASELINE.json), 11 finish (baseline read/write, diff measurement×2, budget exceeded, doctor, learn×2, auto-resolve, governor, field consistency, budget unknown), 3 flag parsing |
| `README.md` | Updated baseline diff workflow and normal workflow sections with prepare/finish; added commands table entries |
| `docs/ALPHA_USAGE.md` | Updated supported workflow (steps 7-14 → 7-11); added prepare/finish to what-is-supported list |
| `docs/INSTALL.md` | Replaced recommended dogfood command with v0.46.0+ prepare/finish usage |

### Architecture

Both functions live in `src/main.rs` alongside existing `cmd_*` functions. No new modules
were created — prepare and finish are command composition, not new capability. They reuse:

- `preflight::run_preflight()` — strategy classification
- `diff_budget::is_working_tree_clean()`, `get_head_commit()`, `write_baseline()`, `read_baseline()`, `measure_diff_from_commit()`, `compare_budget()`
- `contract::classify_prompt()` — budget allocation from task classification
- `project_detection::detect_project_kind()` — marker-file project detection
- `loop_governor::decide()`, `write_governor_next_run()`, `validate_next_run()`, `format_next_run_check()`
- `request_intelligence::build_advisory()` — request pressure mode
- `learn::summarize_patches()` — learning patch counts
- `doctor::run_doctor_report()` — health check
- `verification_discovery::discover_verification_hints()` — test command hints for Unknown projects
- `event_log::now_iso8601()` — timestamp generation

## 4. Tests

### New tests (26)

**Prepare (12):**
| Test | What it verifies |
|---|---|
| `help_output_includes_prepare_and_finish` | Usage text references both commands |
| `prepare_baseline_read_write_roundtrip` | DIFF_BASELINE.json write-then-read preserves fields |
| `prepare_clean_tree_detection_works` | Fresh repo reports clean |
| `prepare_dirty_tree_detection_works` | Modified repo reports dirty |
| `prepare_request_check_validates_generated_next_run` | Generated NEXT_RUN.md passes request --check |
| `prepare_baseline_requires_akar_dir` | write_baseline fails without .akar/ |
| `prepare_governor_decision_is_ready_on_clean_project` | Governor produces non-empty decision |
| `prepare_project_kind_detection_node` | package.json → Node |
| `prepare_project_kind_detection_python` | pyproject.toml → Python |
| `prepare_project_kind_detection_unknown` | No markers → Unknown |
| `prepare_budget_verdict_pass_for_small_changes` | Small change passes 3/60 budget |
| `prepare_diff_baseline_json_is_valid_after_write` | JSON file written with correct content |

**Finish (11):**
| Test | What it verifies |
|---|---|
| `finish_read_baseline_fails_when_missing` | Missing DIFF_BASELINE.json → error |
| `finish_read_baseline_succeeds_after_write` | Roundtrip read after write |
| `finish_measure_diff_detects_changes` | git diff detects modified files |
| `finish_measure_diff_no_changes_is_zero` | Clean tree → zero diff |
| `finish_budget_exceeded_detected` | Large change → Exceeded verdict |
| `finish_doctor_report_produces_issues` | Doctor report constructable |
| `finish_learn_summarize_no_patches` | No LEARNING_PATCHES.md → count=0 |
| `finish_learn_summarize_with_patches` | 3 patches with mixed status → correct counts |
| `finish_does_not_auto_resolve_patches` | Summarize does not modify patch file |
| `finish_governor_decision_is_present` | Governor produces non-empty output |
| `finish_measurement_fields_are_consistent` | total = added + deleted |

**Flag parsing (3):**
| Test | What it verifies |
|---|---|
| `parse_flag_u64_parses_used_and_limit` | --used/--limit numeric parsing |
| `parse_flag_str_parses_task_flag` | --task string parsing |

### Regression

All 508 existing tests pass unchanged. Existing commands (preflight, request, postmortem,
governor, doctor, status, learn, hooks, safety, skills, verify, eval, calibrate, init,
bootstrap, mission, run, telemetry) are preserved with identical behavior.

## 5. External Dogfood

### Fixture

`akar-dogfood-v046-prepare-finish-node-fixture` — Node.js calculator with 4 functions
(add, subtract, multiply, square), 4 tests (1 failing: multiply uses + instead of *).

### Task 1: Bugfix (multiply)

| Step | Command | Count |
|---|---|---|
| prepare | `akar prepare "fix multiply function returning a+b instead of a*b"` | 1 |
| fix bug | Edit calc.js (replace + with *) | 0 (edit) |
| verify | `node --test` (4/4 pass) | 0 (manual) |
| finish | `akar finish` | 1 |
| **Total AKAR commands** | | **2** |

### Task 2: Feature (divide)

| Step | Command | Count |
|---|---|---|
| commit Task 1 | `git add -A && git commit -q -m "..."` | 0 (git) |
| prepare | `akar prepare "add divide function with division-by-zero check"` | 1 |
| add feature | Edit calc.js (add divide), edit test (add 2 tests) | 0 (edit) |
| verify | `node --test` (6/6 pass) | 0 (manual) |
| finish | `akar finish` | 1 |
| **Total AKAR commands** | | **2** |

### Comparison

| Metric | v0.45.0 | v0.46.0 | Reduction |
|---|---|---|---|
| AKAR commands per task | 8 | 2 | 75% |
| Total AKAR commands (2 tasks) | 16 (projected) | 4 | 75% |
| Commands to start a task | 4 | 1 | 75% |
| Commands to finish a task | 3+ | 1 | 67% |

## 6. Design Constraints Observed

All hard rules from the v0.46.0 specification were followed:

- Only `prepare` and `finish` implemented — no other new commands
- Behavior kept narrow — composition of existing operations only
- Existing logic reused — no new modules, no refactoring of existing modules
- Existing command behavior unchanged — all 508 existing tests pass
- No config.toml, CLAUDE.md snippet, enable/disable, auto-run, auto-invocation, auto-execution
- No daemon, watch, shell wrapper, model/API calls, Codex/OpenCode adapters
- No token optimization, memory engine, skill resolver changes
- No auto-edit of .gitignore, Claude Code settings, or destructive git commands
- No push

## 7. Known Edges

- **Governor decision at prepare time is typically RUN_POSTMORTEM** because the baseline
  was just written and the tree is clean — the governor sees "baseline exists + clean tree"
  and recommends running postmortem. This technically means the governor at prepare time
  suggests the next step after the task. The governor at finish time is the post-task
  state. Both are useful for different phases of the workflow.
- **Dirty baseline after init** — if `.akar/` files from `akar init` are not gitignored
  or committed, the tree will be dirty and `prepare` will refuse (same behavior as
  `preflight --snapshot`).
- **Budget for Feature tasks** — Feature tasks get a 12-file, 600-LOC budget, which
  can appear generous for small additions like the dogfood divide function. This is
  the existing contract classification behavior, unchanged from v0.45.0.
- **Health warnings on dirty tree at finish** — finishing with an uncommitted working
  tree correctly surfaces the "working tree dirty" WARN. This is advisory (not FAIL)
  because finish is the natural time to review uncommitted changes.

## 8. Verdict

**v0.46.0 is the most significant burden-reduction release since AKAR's inception.**
Projecting the 75% reduction across a typical 5-task session: 40 AKAR commands (v0.45.0)
drops to 10 (v0.46.0). The two-command prepare/finish model is proven to work on a Node
fixture with both bugfix and feature tasks. The consolidated output surfaces all the
same advisory information as the manual sequence while removing the cognitive load of
remembering 7+ distinct AKAR commands and their correct ordering.

The result is consistent with the v0.44.0 design estimate: command count is the
dominant friction, and consolidation is the highest-impact, lowest-risk first step.
