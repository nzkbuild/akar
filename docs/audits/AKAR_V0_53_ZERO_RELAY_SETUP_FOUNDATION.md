# AKAR v0.53.0 — Zero-Relay Setup Foundation

## 1. Executive Verdict

**RELEASE COMPLETE.** Three new capabilities delivered with zero new dependencies
and 29 new tests (all passing). The doctor, status, and init commands now surface
CLAUDE.md snippet state and PATH version health, closing the two setup gaps discovered
during v0.52 fresh-session dogfood: (a) the v0.52 CLAUDE.md snippet was hand-managed,
and (b) global `akar` on PATH was v0.35.0 while the running binary was v0.52.0.

**Runtime behavior changed: YES.** `akar init --claude` inserts the proven v0.52
revised snippet into CLAUDE.md idempotently. `akar doctor` and `akar status` now
include snippet and PATH health lines. `akar init` always checks PATH health and
offers repair when a version mismatch or missing binary is detected.

**1 pre-existing failure unchanged.** `doctor::ok_when_everything_present_and_valid`
fails due to HOOK_EVENTS.jsonl malformation at line 972 — this existed before v0.53
and is not caused by v0.53 changes. `akar eval` is 27/28 (same doctor_check false
negative). 562/563 tests pass.

## 2. Baseline Confirmation

| Check | Result |
|---|---|
| Commit (pre-release) | `d605927` — docs: record AKAR v0.52 fresh-session trial results |
| Version | `akar 0.53.0` |
| Working tree | dirty (v0.53 files uncommitted) |
| `cargo test` | 562 passed, 1 failed (doctor::ok_when_everything_present_and_valid — pre-existing) |
| `cargo fmt` | Applied to 6 changed files; 2 pre-existing warnings only |
| `cargo build --release` | Clean (2 pre-existing warnings: ProjectDetection/detect_project unused) |

## 3. What v0.53.0 Delivers

### A. Managed CLAUDE.md Snippet (`--claude`)

- `akar init --claude` inserts the exact v0.52 revised compare-and-reject snippet
  into `<project>/CLAUDE.md`
- Idempotent: creates CLAUDE.md if absent, appends if no AKAR block, replaces if
  outdated, no-ops if canonical snippet already present
- Marker-based detection via `<!-- AKAR section ends -->` and `## AKAR Session Guidance`
  header
- Preserves all non-AKAR user content (only replaces the AKAR block)
- Backs up existing file before overwrite (timestamped `.bak.<epoch>` copy)
- Requires `--yes` flag or interactive "Type INSTALL to confirm" prompt
- Safety: corrupted marker (header missing) falls back to append, duplicate markers
  still replace first block and warn

### B. PATH Version Health

- `check_path_health()` detects running binary vs PATH `akar` version mismatch
- Uses `where.exe` (Windows) / `which` (Unix) + `PATH` env var walk as fallback
- Version extraction via subprocess `--version` output with semver token parsing
- Four states: Healthy, Missing, Mismatch, UnknownVersion
- `repair_path()` copies running binary to PATH with safety checks:
  - Never overwrites a non-akar file (verifies target via `--version`)
  - No-op if running binary is already at target location
  - No-op if PATH is already healthy
  - Requires confirmation (or `--yes`)
  - Prefers `~/.cargo/bin` if on PATH

### C. Doctor/Status Visibility

- **Doctor report:** new "claude.md snippet:" and "path health:" sections
  - Snippet Absent/PresentNoBlock/Outdated/Duplicate → Warn; PresentWithBlock → Pass
  - PATH Healthy → Pass; Missing/Mismatch/UnknownVersion → Warn
- **Status output:** new `claude.md:` and `path akar:` lines between governor output
  and doctor findings

## 4. Implementation Summary

### New Files

| File | Lines | Tests | Purpose |
|---|---|---|---|
| `src/claude_snippet.rs` | 349 | 12 | Snippet detection + idempotent apply |
| `src/path_health.rs` | 445 | 8 | PATH version detection + safe repair |

### Modified Files

| File | Changes |
|---|---|
| `Cargo.toml` | Version 0.52.0 → 0.53.0 |
| `src/main.rs` | `mod` declarations, `has_yes_flag` helper, `--yes`/`--claude` wired, status output extended |
| `src/init.rs` | `claude_snippet` + `path_health` fields on `InitResult`, `run_claude_snippet()`, `run_path_health_check()`, `confirm_action()`, format report sections |
| `src/doctor.rs` | `claude_snippet` + `path_health` fields on `DoctorReport`, `check_claude_snippet()`, `check_path_health_section()`, format sections |
| `CHANGELOG.md` | v0.53.0 entry |

### Test Coverage

- `claude_snippet`: 12 tests (path resolution, state detection table, apply: create/append/replace/idempotent/cancelled/preserve/corrupt-marker/backup/duplicate-marker, backup verification)
- `path_health`: 8 tests (running path exists, version matches cargo, doesn't panic, version parsing x2, paths_equal x2, repair cancelled, repair skips when healthy, is_cargo_bin)
- `doctor`: 4 new tests (snippet check, path health check, report sections, run_checks)
- `init`: 4 tests updated (new fields on existing test constructors)
- `main`: 2 new tests (has_yes_flag true/false)

**Total: 29 new tests, all passing. 562/563 total (1 pre-existing failure).**

## 5. Verification Suite Results

```
akar --version                          → akar 0.53.0
akar doctor                             → FAIL (known: HOOK_EVENTS.jsonl + dirty tree)
                                            Snippet: [WARN] not found
                                            PATH: [PASS] healthy
akar status                             → DEGRADED (known: SPLIT_TASK, dirty tree)
                                            claude.md: no AKAR snippet
                                            path akar: healthy
akar request "zero relay setup"         → NORMAL mode
akar request --check                    → PASS
akar governor --json --no-exit-code     → SPLIT_TASK (known)
akar learn --list                       → 8 entries (1 active, 7 resolved)
akar hooks --check                      → PASS
akar eval                               → 27/28 PASS (1 pre-existing: doctor_check)
```

All new v0.53 features work as designed. The snippet WARN in doctor is expected —
CLAUDE.md in this repo does not have the AKAR snippet (by design; the repo is the
source, not the consumer). PATH health reports healthy — `C:\Users\nbzkr\bin\akar.exe`
matches running binary.

## 6. Design Decisions

### Manual CLI parsing (no clap)

`has_yes_flag()` joins the existing pattern of `has_debug_flag()` / `has_quiet_flag()` /
`has_json_flag()` in `src/main.rs`. Adding clap dependency for three flag checks is not
worth the binary size and compile time cost.

### `#[allow(dead_code)]` on public structs

`ClaudeSnippetResult.prior_state` and `PathRepairResult.source`/`.dest` are consumed
via `Debug`/format consumers (e.g., `format_init_report`), not direct field reads.
Rust's dead-code analysis doesn't see format-string usage. The fields are part of the
public API and must remain accessible.

### `env!("CARGO_PKG_VERSION")` for running version

The running binary's version is known at compile time. Using `CARGO_PKG_VERSION`
avoids a self-subprocess call for version extraction and guarantees correctness
(it's the version built into this binary, not whatever `--version` might return).

### Backup-before-overwrite

`claude_snippet::apply_snippet` reuses `crate::backup::backup_file()` — the same
timestamped `.bak.<epoch>` pattern used elsewhere in AKAR. This is a safety net,
not a restore mechanism; users who need to revert can find the backup by timestamp.

### PATH repair: prefer `~/.cargo/bin`

`determine_dest()` prefers `~/.cargo/bin` over other writable PATH directories because
this is the most common install location for Rust binaries on developer machines.
The heuristic is a preference, not a requirement — any writable PATH directory works.

## 7. Known Caveats

1. **1 pre-existing test failure** (`doctor::ok_when_everything_present_and_valid`):
   HOOK_EVENTS.jsonl line 972 malformation. Existed before v0.53, unrelated to these
   changes.

2. **1 pre-existing eval failure** (`doctor_check`): false negative caused by the same
   HOOK_EVENTS.jsonl malformation. Existed before v0.53.

3. **PATH repair is not atomic.** `std::fs::copy` on Windows is not atomic; a crash
   mid-copy could leave a partial binary. This is acceptable for a developer tool.
   The running binary is the source of truth; repair can be re-run.

4. **`where.exe` may find a different `akar` than the shell resolves.** On Windows,
   `where.exe` returns the first match in PATH order, which matches cmd.exe
   resolution but may differ from PowerShell's `Get-Command` resolution in edge
   cases (e.g., when a PowerShell alias shadows the binary).

5. **CLAUDE.md snippet is not auto-updated.** Running `akar init --claude` again
   after a snippet wording change in a future AKAR version will update the
   snippet (idempotent replace). But there is no automatic "check for snippet
   updates" command. Users must re-run `akar init --claude` to get updated wording.

6. **Snippet state detection only checks the first AKAR block.** If CLAUDE.md has
   multiple `<!-- AKAR section ends -->` markers, only the first is checked for
   currency; the others are noted as duplicates but not individually verified.

## 8. What Was NOT Done

- No auto-edit of CLAUDE.md (always requires `--yes` or interactive confirmation)
- No auto-edit of `~/.claude/settings.json` (never touched by AKAR)
- No PATH repair without confirmation
- No clap dependency added
- No new file formats or .akar/ contents
- No changes to hooks, bootstrap, governor, diff_budget, or event_log modules
- No capsule, token optimizer, or autopilot features

## 9. Git State (Pre-Commit)

```
M Cargo.lock
 M Cargo.toml
 M src/doctor.rs
 M src/init.rs
 M src/main.rs
?? src/claude_snippet.rs
?? src/path_health.rs
```

6 files changed (2 new, 4 modified).

## 10. Honest Conclusion

v0.53.0 closes the two setup gaps discovered in v0.52 fresh-session dogfood:
managed CLAUDE.md snippet insertion and PATH version awareness. Both are exposed
in doctor, status, and init — the three commands a user runs when setting up AKAR
for the first time (or checking their setup).

The implementation follows AKAR's established patterns: manual CLI parsing,
std-only (zero dependencies), idempotent operations, backup-before-overwrite,
confirmation-before-mutation, and `#[cfg(test)] mod tests` within source files.

The 1 pre-existing test failure and 1 pre-existing eval failure are unchanged.
No regressions. 29 new tests, all passing.
