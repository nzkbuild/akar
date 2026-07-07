# AKAR v0.31.0 — Project Detection Unification Report

## 1. Baseline

- Commit: `50cb00c` — feat: add AKAR project-aware verification contract
- Version: v0.30.0
- `cargo test`: 479 passed, 0 failed
- `cargo run -- eval`: 28/28 PASS
- Working tree: clean

## 2. Issue addressed

v0.30.0 introduced project-aware verification (`src/project_verification_contract.rs`) with its own `detect_project_kind()` function. `src/verify.rs` had its own independent `detect_recipe()` with separate marker-file detection logic. `src/doctor.rs` and `src/main.rs` each had their own inline `Cargo.toml` checks. Four places duplicated project-detection knowledge.

## 3. Duplicate detection audit

| File | Function/line | Detection | Duplication |
|---|---|---|---|
| `src/project_verification_contract.rs` | `detect_project_kind()` | Rust/Node/Python/Unknown | Canonical from v0.30.0 |
| `src/verify.rs` | `detect_recipe()` (lines 96, 109) | Rust/Node (no Python) | Partial duplicate |
| `src/doctor.rs` | Line 581 inline | Cargo.toml only | Duplicate |
| `src/main.rs` | `cargo_lock_dirty_advisory` (line 911) | Cargo.toml only | Duplicate |
| `src/loop_governor.rs` | `compile_next_run_prompt` | Uses `pvc::detect_project_kind()` | Already unified |
| `src/preflight.rs` | `run_preflight` | Uses `verify::detect_recipe()` | Consumer, not detector |

## 4. Shared detector design

Created `src/project_detection.rs` as the single canonical detector:

- `ProjectKind` enum: Rust, Node, Python, Unknown
- `ProjectDetection` struct: kind, root path, matched marker file, human label, detection reason
- `detect_project_kind(root: &Path) -> ProjectKind`: lightweight detection
- `detect_project(root: &Path) -> ProjectDetection`: rich detection with marker file + reason

## 5. Detection priority and marker rules

Unchanged from v0.30.0:

- Priority: Rust > Node > Python > Unknown
- Markers: `Cargo.toml` (Rust), `package.json` (Node), `pyproject.toml`/`setup.py`/`requirements.txt` (Python), none (Unknown)
- No file parsing, no network access, no package manager invocation

## 6. Verification contract integration

`project_verification_contract.rs` now:
- Re-exports `ProjectKind` from `project_detection`
- Delegates `detect_project_kind()` to `project_detection`
- Contains no marker-file logic of its own
- Retains all build/test/prefix/stop-conditions/guidance data functions

Detection tests moved from `project_verification_contract.rs` (9 tests) to `project_detection.rs` (13 tests — the same 9 plus 4 new rich-detection tests).

## 7. verify.rs boundary

- `detect_recipe()` now uses `crate::project_detection::detect_project_kind()` instead of inline marker checks
- Only `ProjectKind::Rust` triggers automated commands (cargo build, cargo test)
- Non-Rust projects get manual-only recipe with honest label: "no automated verify for {kind} projects — use the project-specific test command"
- `format_results()` now includes `manual_checks` in output
- No new executable recipes for Node or Python; `akar verify` will not run npm or pytest

## 8. NEXT_RUN compatibility

- NEXT_RUN behavior unchanged from v0.30.0
- Node/Python/Unknown projects still get project-appropriate Allowed Commands, Stop Conditions, and Verification Required sections
- `akar request --check` passes for all project kinds
- Rust projects retain `cargo run --` prefix; non-Rust retain `akar` prefix

## 9. Tests added/updated

- `project_detection.rs`: 13 tests (9 detection + 4 rich-detection)
- `project_verification_contract.rs`: removed 9 duplicate detection tests, retained 17 command/prefix/stop-condition tests
- `verify.rs`: tests adapted to new output format (includes manual_checks)
- Total: 483 tests (479 → 483, +4 new detection tests)

## 10. Verification

### Internal

| Command | Result |
|---|---|
| `cargo build --release` | PASS |
| `cargo test` | 483 passed, 0 failed |
| `cargo run -- --version` | akar 0.30.0 (pre-bump) |
| `cargo run -- request "dogfood verification"` | NORMAL mode, NEXT_RUN written |
| `cargo run -- request --check` | PASS |
| `cargo run -- governor --json --no-exit-code` | SPLIT_TASK (expected) |
| `cargo run -- doctor` | WARN (dirty tree) |
| `cargo run -- status` | HEALTHY |
| `cargo run -- hooks --check` | PASS |
| `cargo run -- eval` | 28/28 PASS |
| `cargo run -- verify` | cargo build: PASS, cargo test: PASS |

### External fixtures

| Fixture | project kind | verify behavior |
|---|---|---|
| Node (`package.json`) | Node | No automated checks; manual: "no automated verify for Node projects" |
| Python (`pyproject.toml`) | Python | No automated checks; manual: "no automated verify for Python projects" |
| Unknown (no markers) | Unknown | No automated checks; manual: "no automated verify for Unknown projects" |

## 11. Files changed

- `src/project_detection.rs` — new file (shared detector)
- `src/project_verification_contract.rs` — removed duplicate detection, delegates to shared detector
- `src/verify.rs` — uses shared ProjectKind, non-Rust projects get manual-only recipe, includes manual_checks in output
- `src/doctor.rs` — uses `detect_project_kind()` instead of inline Cargo.toml check
- `src/main.rs` — uses `detect_project_kind()` instead of inline Cargo.toml check; added `mod project_detection`
- `README.md` — updated project-aware verification section
- `docs/INSTALL.md` — clarified `akar verify` limitation for non-Rust projects
- `docs/audits/AKAR_V0_31_PROJECT_DETECTION_UNIFICATION_REPORT.md` — this report

## 12. Honest conclusion

AKAR now has exactly one place that decides project kind. The shared detector is exercised by 13 tests covering all marker combinations and priority rules. The `akar verify` command is honest about its limitations — it advertises non-Rust as unsupported rather than silently doing nothing or trying to run commands that may not exist. No runtime behavior changed for Rust projects, and NEXT_RUN output is identical to v0.30.0 for all project kinds.

The `ProjectDetection` struct and `detect_project()` function exist in the detector module as richer alternatives for future use but are not yet consumed by other modules — they compile without dead-code warnings only because they're in the test path. This is intentional headroom, not dead weight.

## 13. Next recommended release

v0.32.0: Doctor project-kind awareness. The doctor currently only checks "cargo project" (Cargo.toml present vs absent). With the shared detector, the doctor could report the actual project kind and give project-appropriate recommendations instead of a Cargo-only warning.
