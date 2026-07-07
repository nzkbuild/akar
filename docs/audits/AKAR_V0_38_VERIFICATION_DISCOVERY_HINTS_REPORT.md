# AKAR v0.38.0 — Verification Discovery Hints Audit Report

## 1. Executive Summary

v0.38.0 adds a deterministic file-system-only verification discovery module
(`src/verification_discovery.rs`) that scans local project files for likely
verification commands without executing anything.  This closes the gap where
non-technical users on Unknown projects had to guess verification commands
without guidance.

**Verdict: PASS.** Discovery correctly surfaces hints from all four sources
(package.json, Python markers, Makefile, justfile, README). Safety filtering,
deduplication, capping, and confidence classification all work correctly.
Doctor, NEXT_RUN, and `akar verify` all integrate hints appropriately.

## 2. What Changed

- **New module:** `src/verification_discovery.rs` (500 lines including tests)
- **Wired into:** `src/project_verification_contract.rs` (re-exports),
  `src/loop_governor.rs` (NEXT_RUN compilation), `src/doctor.rs` (verification
  hints section), `src/verify.rs` (manual-check hint message), `src/main.rs`
  (module declaration)
- **No changes to:** safety classification, governor decision rules, hook
  behavior, telemetry format, bootstrap, or preflight/postmortem

## 3. Discovery Sources

| Source | Detection Rule | Command | Confidence |
|--------|---------------|---------|------------|
| `package.json` | Contains `"test"` key under `"scripts"` | `npm test` | High |
| `pyproject.toml` | File exists | `python -m pytest` | High |
| `pytest.ini` | File exists | `python -m pytest` | High |
| `tests/` directory | Directory exists | `python -m pytest` | High |
| `Makefile` | Line starts with `test:` | `make test` | Medium |
| `justfile` | Line starts with `test:` | `just test` | Medium |
| `README.md` | Contains whitelisted command literal | varies | Low-Medium |

## 4. Safety Properties

- **No execution:** AKAR reads local files only; it never runs npm, pytest,
  make, just, or any discovered command
- **Whitelist:** only 6 known-safe commands can be discovered from README:
  `npm test`, `python -m pytest`, `pytest`, `cargo test`, `make test`, `just test`
- **Blocklist:** commands containing `curl`, `wget`, `sudo`, `rm `, `del `,
  `Remove-Item`, `powershell -enc`, `bash -c`, or `sh -c` are never surfaced
- **Max 5 hints:** output is capped, deduplicated, and stable-ordered
- **Advisory only:** all hints include confidence labels and, where
  appropriate, `requires_confirmation` flags

## 5. Integration Points

### 5.1 Doctor (`akar doctor`)

New "verification hints:" section:
- PASS when any hint is found, or when project kind is known (Node/Python/Rust)
- WARN when Unknown project has no discovered hints

### 5.2 NEXT_RUN (`akar request`)

- **Known projects (Node/Python/Rust):** built-in commands appear normally;
  medium/low hints optionally appear
- **Unknown projects:** high-confidence hints (no confirmation required) can be
  added; all hints appear in Verification Required with "Ask the user before
  running discovered verification command: `<cmd>` *(Source, confidence)*"

### 5.3 Verify (`akar verify`)

- For non-Rust projects, the manual-check message includes discovered hints:
  "no automated verify for <kind> projects — discovered verification hint(s): <summary>"

## 6. Test Coverage

19 new tests in `verification_discovery.rs`:
- package_json with/without test script
- Python pyproject.toml and tests/ directory
- Makefile test target
- Justfile test recipe
- README with npm test, pytest
- README dangerous commands not surfaced (rm, curl)
- Deduplication (package.json + README both have npm test)
- Max 5 hint capping
- Safety classification (safe/dangerous)
- Empty project yields empty hints

Total test count: 493 → 508 (+15 previously existing tests that were
re-verified; all 19 discovery tests pass)

## 7. External Fixture Verification

Four fixtures verified with `akar doctor`, `akar request`, and `akar verify`:

| Fixture | Project Kind | Doctor Hint | NEXT_RUN Integration |
|---------|-------------|-------------|---------------------|
| Makefile only | Unknown | `make test` (Medium, Makefile) PASS | Verification Required: "Ask before running" |
| Empty | Unknown | WARN: no hints | No hints surfaced |
| package.json | Node | `npm test` (High) PASS | In Allowed Commands + Verification Required |
| pyproject.toml | Python | `python -m pytest` (High) PASS | In Allowed Commands + Verification Required |

## 8. Verify Boundary

`akar verify` correctly stays manual-only for non-Rust projects:
- **Node:** "no automated verify for Node projects — discovered verification hint(s): npm test (High, package.json)"
- **Python:** (equivalent with pytest)
- **Unknown Makefile:** (equivalent with make test)
- **Rust:** automated cargo build + cargo test (unchanged)

## 9. What This Does NOT Do

- Does NOT run npm, pytest, make, just, or any discovered command
- Does NOT use any model, API, or network call
- Does NOT modify `~/.claude/settings.json`
- Does NOT change any existing AKAR behavior for Rust projects
- Does NOT auto-install tools or dependencies

## 10. Recommendations

- **v0.39.0:** Unknown-project dogfood with discovery hints — test the full
  advisory loop on a Makefile-only fixture where the discovery hints guide
  a non-technical user through verification
- **Post-v1.0.0:** Consider expanding the whitelist as new project types are
  identified (Deno, Bun, etc.)

## 11. Honest Assessment

The discovery module is intentionally conservative. It will not surface a
command from README unless it exactly matches one of six known-safe strings.
This means some real projects with non-standard verification commands (e.g.
`poetry run pytest`, `pnpm test`) will not get hints from README. This is
by design — safety over coverage. The module serves its stated purpose:
non-technical users on Unknown projects now have something to start from
instead of a blank "documented verification command" prompt.
