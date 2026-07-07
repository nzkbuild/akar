# AKAR v0.32.0 — Doctor Project-Kind Awareness Report

## 1. Baseline

- Commit: `c661816` — refactor: unify AKAR project detection
- Version: v0.31.0
- `cargo test`: 483 passed, 0 failed
- `cargo run -- eval`: 28/28 PASS
- Working tree: clean

## 2. Doctor Reality Check issue addressed

The v0.31.0 Doctor Reality Check (external fixture inspection, no source changes) found that `akar doctor` and `akar init` correctly reported the project kind textually but:
- The check was still named "cargo project" — implying Cargo was the expected norm
- Node and Python projects received WARN because Cargo.toml was absent, even though detection worked correctly
- This was misleading for non-Rust users

## 3. Old misleading behavior

| Project kind | Doctor label | Severity | Message |
|---|---|---|---|
| Rust | `cargo project` | PASS | "Cargo.toml found" |
| Node | `cargo project` | WARN | "no Cargo.toml — project kind is \"Node\"; ..." |
| Python | `cargo project` | WARN | "no Cargo.toml — project kind is \"Python\"; ..." |
| Unknown | `cargo project` | WARN | "no Cargo.toml — project kind is \"Unknown\"; ..." |

`akar init` echoed the same misleading framing in its "doctor: issues remain" section.

## 4. New project-kind behavior

| Project kind | Doctor label | Severity | Message |
|---|---|---|---|
| Rust | `project kind` | PASS | "Rust (Cargo.toml) — 'akar verify' runs cargo build + cargo test" |
| Node | `project kind` | PASS | "Node — NEXT_RUN uses project-appropriate commands; 'akar verify' automated execution is Rust/Cargo-only" |
| Python | `project kind` | PASS | "Python — NEXT_RUN uses project-appropriate commands; 'akar verify' automated execution is Rust/Cargo-only" |
| Unknown | `project kind` | WARN | "Unknown — no Rust, Node, or Python markers found; NEXT_RUN will use documented-verification guidance" |

## 5. Severity rules

- **PASS**: Rust, Node, Python — a known project kind was detected. The shared `project_detection` module did its job.
- **WARN**: Unknown — no supported markers found. NEXT_RUN will use documented-verification fallback, and the user may need to review README/project scripts.

## 6. Init output behavior

`akar init` now shows project-kind issues only for Unknown. Node and Python projects no longer get a "no Cargo.toml" warning in the init summary.

## 7. Verify boundary preserved

- `akar verify` runs `cargo build` + `cargo test` for Rust projects only (unchanged)
- Node/Python/Unknown: manual-only recipe with honest label (unchanged from v0.31.0)
- No new executable recipes; no npm or pytest invocation

## 8. Tests added/updated

10 new doctor tests:
- `project_kind_rust_is_pass_and_labeled_project_kind`
- `project_kind_node_is_pass_and_labeled_project_kind`
- `project_kind_python_is_pass_and_labeled_project_kind`
- `project_kind_unknown_is_warn_and_labeled_project_kind`
- `node_doctor_output_does_not_contain_cargo_project`
- `python_doctor_output_does_not_contain_cargo_project`
- `unknown_doctor_output_does_not_contain_cargo_project`
- `node_doctor_output_does_not_warn_about_missing_cargo_toml`
- `python_doctor_output_does_not_warn_about_missing_cargo_toml`
- `unknown_doctor_output_does_not_say_cargo_required`

Test count: 483 → 493.

## 9. External fixture verification

| Fixture | doctor label | severity | "cargo project" seen? | verify runs npm/pytest? |
|---|---|---|---|---|
| Node (`package.json`) | `project kind` | PASS | No | No |
| Python (`pyproject.toml`) | `project kind` | PASS | No | No |
| Unknown (no markers) | `project kind` | WARN | No | No |

## 10. Verification

| Command | Result |
|---|---|
| `cargo build --release` | PASS |
| `cargo test` | 493/493 PASS |
| `cargo run -- --version` | akar 0.32.0 |
| `cargo run -- doctor` | PASS project kind: Rust (Cargo.toml) |
| `cargo run -- eval` | 28/28 PASS |
| `cargo run -- request --check` | PASS |

## 11. Honest conclusion

The doctor no longer assumes every project should have a Cargo.toml. Node and Python projects get PASS for project-kind detection — reflecting that the shared detector worked correctly. Only Unknown projects get WARN, which is honest: AKAR cannot confidently infer the right test command. The "cargo project" check name is gone from all user-facing output.

The fix is narrow: one match block in `doctor.rs`, 10 tests, and two doc updates. No behavior outside doctor/init was changed.

## 12. Next recommended release

v0.33.0: Third external dogfood trial on a non-Rust project, confirming the full AKAR loop works end-to-end with project-kind awareness.
