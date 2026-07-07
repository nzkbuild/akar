# AKAR v0.33.0 — Third External Dogfood Trial Report

## 1. Baseline

- Commit: `29f0298` — fix: make AKAR doctor project-kind aware
- Version: akar 0.32.0
- `cargo test`: 493/493 PASS
- `cargo run -- eval`: 28/28 PASS
- `cargo run -- doctor`: PASS project kind: Rust (Cargo.toml)
- Working tree: clean

## 2. Dogfood fixture setup

- Path: `../akar-dogfood-v033-node-fixture`
- Language: Node.js (CommonJS)
- Marker: `package.json`
- Files: `package.json`, `src/calc.js`, `test/calc.test.js`
- Initial state: 2 passing tests (add, subtract), 1 intentionally failing test (multiply uses `add(3,3)` but asserts `9`)
- Git: init + commit with 3 files, clean tree

## 3. Fixture baseline verification

| Check | Result |
|---|---|
| `node --test test/*.test.js` | 2 pass, 1 fail (multiply) |
| Git tree | clean, 3 files committed |
| Failing test assertion | `add(3, 3)` returns 6, expected 9 |
| `package.json` scripts.test | `node --test test/*.test.js` |

## 4. Full dogfood loop

### 4.1 Init

```
akar init
```

- Bootstrap: 0 created, 0 skipped
- Warnings: templates directory not found (expected — Node fixture has no AKAR source tree)
- Doctor: reported NEXT_RUN.md and DIFF_BASELINE.json as missing (expected — fresh init)
- No errors or unexpected output

### 4.2 Hooks

```
akar hooks --install --check
```

- Source: embedded fallback
- Status: PASS
- Templates found: `pre-tool-call.sh`, `pre-tool-call.ps1`
- Correctly used embedded templates since no source-tree templates in a Node fixture

### 4.3 Doctor

```
akar doctor
```

| Check | Severity | Notes |
|---|---|---|
| project root | PASS | correct path |
| .akar/ directory | PASS | exists |
| .akar/ writable | PASS | yes |
| akar on PATH | PASS | yes |
| NEXT_RUN.md present | WARN | missing (expected — not yet generated) |
| DIFF_BASELINE.json | WARN | missing (expected — no snapshot yet) |
| LEARNING_PATCHES.md | PASS | absent |
| hook templates | PASS | valid (source: embedded) |
| Claude settings wiring | PASS | manual |
| EVENT_LOG.jsonl | PASS | absent |
| HOOK_EVENTS.jsonl | PASS | absent |
| git repository | PASS | detected |
| working tree | PASS | clean |
| git HEAD | PASS | valid |
| **project kind** | **PASS** | **Node — NEXT_RUN uses project-appropriate commands; 'akar verify' automated execution is Rust/Cargo-only** |
| NEXT_RUN.md valid | WARN | missing |

Key finding: project kind is "project kind" (not "cargo project"), severity is PASS for Node. No "Cargo.toml not found" warning. This confirms the v0.32.0 doctor fix works.

### 4.4 Status

```
akar status
```

- Status: HEALTHY
- Baseline readiness: READY
- Governor decision: SNAPSHOT_NOW (no baseline, clean tree — correct for fresh fixture)

### 4.5 Verify

```
akar verify
```

Output:
```
Verified:
  (no automated checks)
Manual checks:
  - no automated verify for Node projects — use the project-specific test command
  - check changed files match task scope
  - no secrets in output
Not verified:
  - browser click-through
  - production deployment
```

Correctly refuses to run automated verification for Node. No `npm test` was executed by AKAR.

### 4.6 Preflight

```
akar preflight --snapshot "fix one small failing test in the Node dogfood fixture"
```

- Prompt: recorded
- Task classification: Bugfix
- Risk: Low
- Autonomy: A5
- Diff budget: 1-3 files, 5-60 LOC
- Request mode: NORMAL
- Snapshot written: head `e6a8ae46ea42`

### 4.7 Request

```
akar request "fix the intentionally broken multiply test — replace add(3,3) with a real multiply function and test"
```

- Request mode: NORMAL
- Governor decision: RUN_POSTMORTEM (baseline exists, tree is dirty from init files — expected)
- NEXT_RUN.md written successfully

### 4.8 Request check

```
akar request --check
```

NEXT_RUN check: PASS
- sections: PASS
- minimum content: PASS
- safety contract: PASS
- decision consistency: PASS

### 4.9 Governor

```
akar governor --json --no-exit-code
```

Decision: RUN_POSTMORTEM. Reason: tree dirty from init files. Correct decision for the state.

## 5. NEXT_RUN.md inspection

The compiled prompt was inspected for project-aware quality:

| Section | Content verified | Result |
|---|---|---|
| Current State | Shows "project kind: Node" | PASS |
| Current State | Shows requested task text | PASS |
| Governor Decision | RUN_POSTMORTEM with suggested prompt | PASS |
| Evidence Used | Complete evidence inventory | PASS |
| Allowed Commands | Includes `npm test` | PASS |
| Allowed Commands | No `cargo build`, `cargo test`, or any Cargo commands | PASS |
| Forbidden Commands | Lists destructive commands | PASS |
| Stop Conditions | Includes "Stop if `npm test` fails" | PASS |
| Stop Conditions | No Cargo-related stop conditions | PASS |
| Verification Required | Includes `npm test` | PASS |
| Verification Required | No `cargo build` or `cargo test` | PASS |
| Final Response Format | Standard 10-item format present | PASS |
| Request contract | 11 sections, safety, consistency | PASS (via `request --check`) |

No Cargo commands appeared anywhere in NEXT_RUN.md. The project-kind detection correctly guided all generated sections.

## 6. Manual task execution

Task: Fix the intentionally broken multiply test.

Changes:
- `src/calc.js`: Added `multiply` function, exported it
- `test/calc.test.js`: Import `multiply`, fix test assertion from `add(3,3)` to `multiply(3,3)`, fix test name

Result:
```
node --test test/*.test.js
✔ add adds two numbers
✔ subtract subtracts two numbers
✔ multiply multiplies two numbers
ℹ tests 3, pass 3, fail 0
```

## 7. Post-task verification

### 7.1 Project test

| Test | Result |
|---|---|
| `node --test test/*.test.js` | 3/3 PASS |

### 7.2 Postmortem

```
akar postmortem --diff --baseline
```

| Field | Value |
|---|---|
| Baseline task | Bugfix |
| Baseline budget | 3 files, 60 LOC |
| Actual files | 2 |
| Actual changes | 8 added, 4 deleted (12 total LOC) |
| Status | **PASS** |

Within budget. No diff budget violation.

### 7.3 Learn

```
akar learn --list
```

- 0 entries, 0 active, 0 resolved (no learning patches recorded — expected for a clean one-task session)

### 7.4 Governor

```
akar governor --json --no-exit-code
```

Decision: RUN_POSTMORTEM (tree dirty with uncommitted changes — expected)

### 7.5 Doctor

```
akar doctor
```

- All checks PASS except: working tree dirty (expected — uncommitted fix)
- project kind: Node — PASS (correct)
- No "cargo project" or "Cargo.toml not found" anywhere

### 7.6 Status

```
akar status
```

- Status: HEALTHY
- Readiness: BLOCKED (tree dirty — correct)
- Guidance: instructs to commit, not force-discard

## 8. Hook evidence

No HOOK_EVENTS.jsonl was recorded — the hooks were not wired into a live Claude Code session for this trial. This is expected: the dogfood fixture was tested by running AKAR CLI commands directly, not through a PreToolUse hook. The hook templates were verified as present and valid via `akar hooks --check`.

For a full hook trial, the user would need to:
1. Run `akar hooks --install` in the fixture
2. Manually register the hook in `~/.claude/settings.json` PreToolUse
3. Run a Claude Code session in the fixture directory

This is a known scope boundary of AKAR's design: AKAR provides the templates and verification but never edits Claude Code settings. The hook pipeline itself was verified end-to-end in previous dogfood trials (v0.28.0, v0.30.0).

## 9. Project-kind awareness verification

| Behavior | Expected | Actual |
|---|---|---|
| Doctor check name | "project kind" (not "cargo project") | "project kind" |
| Doctor severity for Node | PASS (not WARN) | PASS |
| Doctor message | "Node — NEXT_RUN uses project-appropriate commands" | Correct |
| Verify automated execution | None for Node | (no automated checks) |
| Verify manual checks | Present | 3 manual checks |
| NEXT_RUN Allowed Commands | `npm test`, no Cargo | `npm test`, no Cargo |
| NEXT_RUN Verification | `npm test`, no Cargo | `npm test`, no Cargo |
| NEXT_RUN Stop Conditions | `npm test` failure, no Cargo | Correct |

All project-kind behaviors verified. No regression from v0.31.0 or v0.32.0.

## 10. Verify boundary still intact

`akar verify` still does not run `npm test` or `pytest` for non-Rust projects. The Node fixture confirms this: verify output showed "(no automated checks)" and directed the user to manual verification. The boundary from v0.31.0 is preserved.

## 11. Governor behavior

| State | Decision | Reason | Correct? |
|---|---|---|---|
| Post-init, clean tree, no baseline | SNAPSHOT_NOW | No baseline + clean tree | Yes |
| Post-prefight, dirty tree (init files) | RUN_POSTMORTEM | Baseline exists + dirty tree | Yes |
| Post-fix, dirty tree (uncommitted) | RUN_POSTMORTEM | Baseline exists + dirty tree | Yes |

All governor decisions were appropriate for the state. No unexpected STOP or BLOCK decisions.

## 12. No regression checks

| Command | Result |
|---|---|
| `cargo build --release` | PASS |
| `cargo test` (AKAR repo) | 493/493 PASS |
| `cargo run -- eval` | 28/28 PASS |
| `cargo run -- doctor` (AKAR repo) | PASS project kind: Rust |
| `cargo run -- request --check` (AKAR repo) | PASS |
| `akar request` (Node fixture) | Generated project-aware NEXT_RUN |
| `akar request --check` (Node fixture) | PASS |

## 13. Issues and rough edges

1. **Init creates dirty tree before preflight**: `akar init` creates `.akar/` files (NEXT_RUN.md, etc.) which make the working tree dirty. The governor immediately reports RUN_POSTMORTEM after preflight because `.akar/` files are untracked. This is a known pattern — users must either commit `.akar/` or add it to `.gitignore`. The doctor correctly reports this as "working tree: dirty — commit or review changes before a measured session."

2. **No hook event telemetry**: Since the trial was CLI-only without a live Claude Code session, `HOOK_EVENTS.jsonl` remained empty. This is expected and not a bug, but it means hook pipeline testing requires a real Claude Code session.

3. **Dead-code warnings**: `project_detection.rs` still has `ProjectDetection` struct and `detect_project` function marked unused — they exist as intentional headroom for future work. No functional impact.

## 14. What was not tested

- Live Claude Code session with wired PreToolUse hooks
- Hook execution under tool-call safety validation
- `akar postmortem` after multiple sequential tasks
- `akar learn` with actual learning patches
- Python or Unknown project fixtures
- Cross-platform (macOS/Linux) behavior
- `akar run` (full workflow in one command)

## 15. Honest conclusion

The v0.33.0 dogfood trial confirms that AKAR's project-kind awareness works end-to-end on a non-Rust project. The doctor correctly reports "project kind: Node — PASS". NEXT_RUN.md uses `npm test` in Allowed Commands, Verification Required, and Stop Conditions without any Cargo commands. `akar verify` correctly refuses automated execution for Node. The postmortem diff measurement works correctly.

No regressions were found in the AKAR repo itself (493 tests, 28 eval cases all pass). The v0.31.0 project detection unification and v0.32.0 doctor awareness fix both hold up under external testing.

The dogfood loop is smooth: init → hooks --check → doctor → status → preflight → request → request --check → manual task → project test → postmortem. Each step provides clear, actionable output. The governor makes appropriate decisions at every state transition.

## 16. Next recommended release

v0.34.0: A focused feature release. Candidate areas:
- Thread the verbatim task description from `akar request "<task>"` into NEXT_RUN.md Objective section (user-visible quality of life)
- Or: `akar run "<task>"` end-to-end one-command dogfood (init → preflight → request in one shot)
- Or: consume `ProjectDetection` and `detect_project` in another module to resolve the dead-code warnings
