# AKAR v0.30.0 — Project-Aware Verification Contract Report

## 1. Baseline

Starting point: AKAR v0.29.0, 453 tests passing, clean release build, clean working tree. The v0.27.0 second external dogfood trial reported that NEXT_RUN's Allowed Commands and Verification Required sections were hardcoded to Cargo commands (`cargo build --release`, `cargo test`, `cargo run -- ...`) regardless of the project being dogfooded. This release replaces that static list with project-aware commands derived from a single canonical verification contract.

## 2. Dogfood issue addressed

Second external dogfood trial, next recommended release: "The last remaining v0.27.0 finding — NEXT_RUN's hardcoded Rust/cargo command lists on non-Rust projects — should be addressed next." A Node or Python repo bootstrapped with AKAR would see `cargo build --release` and `cargo test` in its NEXT_RUN prompt, which is confusing and wrong.

## 3. Hardcoded command audit

Before v0.30.0, three functions in `src/loop_governor.rs` returned hardcoded Cargo commands:

- `allowed_commands()` (line 978): 12 always-on commands, 8 cargo-specific
- `verification_commands()` (line 1061): 8 always-on commands, all cargo-specific
- `stop_conditions()` (line 1029): "Stop if `cargo test` fails." hardcoded

`src/contract.rs`: `default_verification()` returned cargo-only commands for all task types.

`src/verify.rs`: `detect_recipe()` already detected Cargo.toml vs package.json — a reusable pattern that was not wired into NEXT_RUN generation.

## 4. Verification contract design

New module `src/project_verification_contract.rs` is the single source of truth:

- `ProjectKind` enum: Rust, Node, Python, Unknown
- `detect_project_kind()`: priority Rust > Node > Python > Unknown
- `build_commands()`: only Rust returns `cargo build --release`
- `test_commands()`: Rust → `cargo test`, Node → `npm test`, Python → `python -m pytest`, Unknown → empty
- `akar_prefix()`: Rust → `cargo run --`, non-Rust → `akar`
- `akar_cli_commands()`: `--version`, `status`, `governor --json --no-exit-code`, `doctor`, `eval`, `hooks --check` with appropriate prefix
- `project_allowed_commands()`: build + test commands
- `project_verification_commands()`: same as project_allowed_commands
- `project_stop_conditions()`: project-specific "Stop if X test fails"
- `unknown_verification_guidance()`: human-readable guidance for unknown projects

No duplicate table exists — this is the only source.

## 5. Project detection behavior

Detection rules are marker-file existence only. No file parsing, no network access, no package manager invocation.

| Marker(s) | Detection |
|---|---|
| `Cargo.toml` exists | Rust |
| `package.json` exists (no Cargo.toml) | Node |
| `pyproject.toml` / `setup.py` / `requirements.txt` exists (no Cargo.toml or package.json) | Python |
| None of the above | Unknown |

Priority: Rust > Node > Python > Unknown. A repo with both Cargo.toml and package.json is treated as Rust.

## 6. NEXT_RUN behavior by project kind

### Rust (AKAR repo itself)
- Allowed Commands: `cargo build --release`, `cargo test`, `cargo run -- <cli commands>`
- Verification Required: same
- Stop Conditions: "Stop if `cargo test` fails."
- AKAR CLI prefix: `cargo run --`

### Node
- Allowed Commands: `npm test`, `akar <cli commands>`
- Verification Required: `npm test`, `akar <cli commands>`
- No `cargo build --release` or `cargo test`
- Stop Conditions: "Stop if `npm test` fails."
- AKAR CLI prefix: `akar`

### Python
- Allowed Commands: `python -m pytest`, `akar <cli commands>`
- Verification Required: `python -m pytest`, `akar <cli commands>`
- No `cargo build --release` or `cargo test`
- Stop Conditions: "Stop if `python -m pytest` fails."
- AKAR CLI prefix: `akar`

### Unknown
- Allowed Commands: `akar <cli commands>` (no build/test commands)
- Verification Required: `akar <cli commands>` plus guidance lines:
  - "Run the project's documented verification command."
  - "Inspect README or project scripts before choosing a test command."
- No `cargo build --release`, `cargo test`, `npm test`, or `python -m pytest`
- Stop Conditions: "Stop if verification fails (run the project's documented verification command)."
- AKAR CLI prefix: `akar`

Common across all kinds: Current State now includes `- project kind: <label>`.

## 7. Validator compatibility

`akar request --check` passes for all four project kinds. The validator checks structure, safety contract, and decision consistency — not exact project command text. No validator changes were required; the validator never required Cargo commands.

## 8. Governor behavior preserved

`akar governor` does not write NEXT_RUN. Governor decision rules are unchanged. Governor exit codes are unchanged. Governor telemetry is unchanged.

## 9. Tests added

`src/project_verification_contract.rs` — 26 new tests:
- 3 detection: Rust from Cargo.toml, Node from package.json, Unknown from empty
- 3 Python detection: pyproject.toml, setup.py, requirements.txt
- 3 priority: Rust > Node, Rust > Python, Node > Python
- 2 build commands: Rust has cargo build, Node is empty
- 4 test commands: Rust/cargo, Node/npm, Python/pytest, Unknown/empty
- 3 akar prefix: Rust uses `cargo run --`, non-Rust uses `akar`
- 4 project allowed: Rust has cargo build+test, Node has npm only (no cargo), Python has pytest only (no cargo), Unknown is empty
- 4 stop conditions: each project kind returns appropriate stop condition text
- 1 unknown guidance: includes "documented verification"

Total: 453 → 479 (+26).

## 10. External fixture verification

Three temporary repos were created and verified:

### Node fixture (package.json)
- NEXT_RUN includes `npm test` in Allowed Commands and Verification Required
- NEXT_RUN does NOT include `cargo build --release` or `cargo test`
- `project kind: Node`, AKAR prefix: `akar`
- `request --check`: PASS

### Python fixture (pyproject.toml)
- NEXT_RUN includes `python -m pytest` in Allowed Commands and Verification Required
- NEXT_RUN does NOT include `cargo build --release` or `cargo test`
- `project kind: Python`, AKAR prefix: `akar`
- `request --check`: PASS

### Unknown fixture (no marker files)
- NEXT_RUN contains no `cargo build --release`, `cargo test`, `npm test`, or `python -m pytest` as required commands
- Verification Required includes guidance: "Run the project's documented verification command."
- `project kind: Unknown`, AKAR prefix: `akar`
- `request --check`: PASS

## 11. Honest conclusion

The fix replaces three hardcoded Cargo functions in `loop_governor.rs` with calls into a single project-detection contract. The contract module is self-contained, has no external dependencies, and the detection rules are marker-file presence only — no parsing, no network, no side effects. AKAR's own NEXT_RUN is unchanged (it's a Rust project). No governor behavior changed. The validator required no modifications. The diff is focused: one new module, three functions refactored in loop_governor, two docs updated.

## 12. Next recommended release

The remaining v0.27.0 dogfood findings are now fully addressed. AKAR v0.31.0 should focus on what the v0.29.0 report identified: improving `akar verify` to use the same project detection contract for recipe building (currently `verify.rs` has its own independent Rust/Node detection that could be unified).

A broader v0.31.0 target could be introducing a `ProjectDetection` struct that both `verify.rs` and `project_verification_contract.rs` consume, eliminating the duplicated detection logic.
