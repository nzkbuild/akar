# AKAR v0.34.0 — Stable Advisory Alpha Freeze Report

## 1. Baseline

- Commit: `9786fb8` — docs: record AKAR third external dogfood trial
- Version: akar 0.33.0
- `cargo test`: 493/493 PASS
- `cargo run -- eval`: 28/28 PASS
- `cargo run -- doctor`: PASS (advisory: LEARNING_PATCHES split-rule)
- Working tree: clean

## 2. Evidence from v0.33 dogfood

The third external dogfood trial (Node.js fixture, CLI-only) confirmed:

- **Project-kind awareness holds end-to-end**: doctor reports "project kind: Node — PASS", NEXT_RUN uses `npm test` with zero Cargo commands, verify refuses automated execution for Node.
- **The CLI advisory loop is stable**: init → hooks check → doctor → status → verify → preflight → request → request --check → governor → manual fix → project test → postmortem all passed cleanly.
- **Postmortem diff measurement is reliable**: 2 files, 12 LOC, within budget, PASS.
- **No regressions**: 493/493 tests, 28/28 eval all pass in the AKAR repo.

The v0.27 (Node.js) and v0.24 (Rust) dogfood trials provide additional evidence that the CLI advisory loop works across project types.

## 3. What was NOT proven in v0.33

- **Live hook-integrated telemetry**: v0.33 was CLI-only. No Claude Code session was run with wired PreToolUse hooks in the dogfood fixture, so `HOOK_EVENTS.jsonl` was never populated.
- **Python/Unknown project dogfood**: Only Rust (v0.24) and Node (v0.27, v0.33) have been dogfooded end-to-end.
- **Multi-task sessions**: All three dogfood trials were single-task (one baseline, one fix, one postmortem).
- **Cross-platform behavior**: All dogfooding has been on Windows.

## 4. Stable advisory alpha definition

AKAR stable advisory alpha means:

| Scope | Status |
|---|---|
| CLI advisory commands | Stable and tested |
| Project-kind-aware NEXT_RUN | Stable (4 project kinds, verified) |
| Doctor/status/governor guidance | Stable |
| Preflight snapshot + diff budget | Stable |
| Postmortem diff measurement | Stable |
| Hook template install/check | Stable (embedded fallback) |
| Safety classification | Stable |
| Local-only evidence files | Stable |
| Hook integration in live sessions | NOT yet stable-alpha proven |
| Autonomous execution | NOT supported |
| Multi-agent / model routing | NOT supported |
| Multi-task session | NOT yet dogfooded |
| Python/Unknown dogfood | NOT yet dogfooded |

## 5. Supported CLI loop

The supported stable-alpha workflow (documented in full at `docs/ALPHA_USAGE.md`):

1. `akar init`
2. Decide `.akar/` gitignore/commit handling
3. `akar hooks --install` (optional)
4. Manually wire Claude Code PreToolUse (optional)
5. `akar hooks --check`
6. `akar doctor`
7. `akar status`
8. `akar preflight --snapshot "<task>"`
9. `akar request "<task>"`
10. `akar request --check`
11. Use `.akar/NEXT_RUN.md` manually with Claude Code
12. Run project tests manually
13. `akar postmortem --diff --baseline`
14. `akar learn --list`
15. Commit only intentional changes

AKAR does not execute the NEXT_RUN prompt, does not run project tests for Node/Python, does not decide gitignore choices, and refuses dirty snapshots by design.

## 6. Unsupported / not-yet-stable areas

| Area | Status | Target |
|---|---|---|
| Live hook-integrated dogfood | Not proven | v0.35.0 |
| Python external dogfood | Not proven | v0.36.0 |
| Multi-task session dogfood | Not proven | v0.37.0 |
| Cross-platform independent dogfood | Not proven | Post-v0.37.0 |
| Autonomous `akar run` execution | Not implemented | v1.0.0+ |
| Model routing / API calls | Not implemented | v1.0.1+ design |
| OpenCode/Codex adapters | Not implemented | v1.0.1+ design |
| Token cache / cost optimizer | Not implemented | v1.0.1+ design |
| Background daemon | Not implemented | Future |
| Cloud telemetry | Not implemented | Future (opt-in only) |

## 7. Privacy and local guarantees

These guarantees are stable and will not be silently removed:

- **Local-only by default**: reads/writes only within `.akar/` and `~/.claude/akar/`.
- **No model API calls**: deterministic CLI, no LLM integration.
- **No Claude settings mutation**: `~/.claude/settings.json` is never touched.
- **No source-code edits**: generates `.md` and `.json` files only.
- **No destructive git**: never runs reset/clean/stash/checkout/push.
- **No auto-apply of learning patches**: `learn --list` is read-only.
- **Hook templates always available**: embedded fallback ensures install/check always works.
- **`request --check` validates structure**: section count, minimum content, safety contract, decision consistency.
- **Postmortem measures honestly**: diff against baseline HEAD, no fudging.
- **Doctor reports project kind honestly**: "project kind" not "cargo project", PASS/WARN per actual detection.

## 8. Safety boundaries

- `akar safety` classifies commands: BLOCKED, SKIP, or ALLOW.
- Destructive commands (`rm -rf /`, `git reset --hard`, etc.) are BLOCKED.
- The PreToolUse hook (when wired) calls `akar safety` before tool execution.
- AKAR never auto-executes, never bypasses safety, and never relaxes classification without explicit code change.
- The hook templates are auditable — they are short shell/PowerShell scripts with no obfuscation.

## 9. Hook integration boundary

This is the most important boundary in stable advisory alpha:

- AKAR provides hook templates (`pre-tool-call.sh`, `pre-tool-call.ps1`) that are embedded in the binary.
- `akar hooks --install` writes them to `.akar/hooks/`.
- `akar hooks --check` verifies they exist and are parseable.
- AKAR prints the manual wiring instructions (`akar hooks`).
- AKAR **never** edits `~/.claude/settings.json`.
- The hook pipeline was verified in earlier trials (v0.28.0, v0.30.0) within the AKAR repo, but **not in a live external-repo Claude Code session during v0.33**.

This means: the hook templates are correct and installable, but the full path from "Claude Code fires PreToolUse → hook calls `akar safety` → event logged to the correct project's `HOOK_EVENTS.jsonl`" has not been independently dogfooded on an external repo.

## 10. Verification boundary

- `akar verify` runs `cargo build` + `cargo test` for Rust projects only.
- For Node, Python, and Unknown projects, it reports "(no automated checks)" and directs the user to manual verification.
- This boundary is intentional and will not change without a dedicated release that dogfoods each new automated recipe independently.

## 11. Known limitations

Documented in full at `docs/ALPHA_USAGE.md`. Key items:

- Live Claude Code hook telemetry not dogfooded in v0.33
- Hook wiring always manual (by design, but a friction point)
- Hook behavior depends on user PATH/session setup
- Python/Unknown projects need full dogfood
- Cross-platform behavior needs broader validation
- Multi-task sessions need dogfood
- `akar verify` is Rust/Cargo only
- `akar run`/`akar mission` are scaffolds, not engines
- Cost/token optimization is future work
- Multi-agent support is future work

## 12. Why no feature was added

v0.34.0 is a documentation/audit release. The v0.33 dogfood report's next-release recommendation suggested a feature release, but the better decision is a freeze first:

1. **Three releases in a row added project-kind awareness** (v0.30.0 contract, v0.31.0 unification, v0.32.0 doctor fix). Adding more features before documenting the current baseline would accumulate scope without a clear stability contract.
2. **The v0.33 dogfood exposed a gap**: live hook telemetry was not tested. A freeze makes that gap explicit and scopes it as v0.35.0 work.
3. **A freeze before features is honest**: it tells users exactly what they can rely on today, rather than letting them infer stability from the absence of a warning.

No src/ code was modified. No runtime behavior changed. No internals were refactored.

## 13. Why this is not v1.0.0

v1.0.0 requires:

- **Live hook-integrated dogfood proof** (not just CLI): a Claude Code session with wired PreToolUse, on an external repo, with hook events recorded and verified.
- **Python external dogfood proof**: same full loop as v0.33 Node, but on a Python fixture.
- **Multi-task session proof**: multiple sequential preflight → fix → postmortem cycles in one session.
- **Cross-platform validation**: at minimum, the CLI loop verified on macOS or Linux.

None of these have been completed. Claiming v1.0.0 without them would overstate AKAR's readiness.

## 14. Recommended next releases

| Version | Scope | Rationale |
|---|---|---|
| v0.35.0 | Live Hook Dogfood Trial | Fill the biggest gap: prove PreToolUse hook telemetry works in a real Claude Code session on an external repo |
| v0.36.0 | Python External Dogfood Trial | Extend project-kind proof to Python (Node and Rust already done) |
| v0.37.0 | Multi-task Session Dogfood Trial | Prove the loop holds across multiple sequential tasks |
| v1.0.0 | Release Candidate review | After all three dogfood gaps are closed, review for v1.0.0 |
| v1.0.1+ | Design work | Multi-agent support, token optimization, OpenCode/Codex adapters — deferred until the single-agent root is proven stable |

Multi-agent support, token optimization, and platform adapters are explicitly deferred to post-v1.0.0 design work. The priority is proving the single-agent advisory loop is solid end-to-end before adding scope.

## 15. Verification

| Command | Result |
|---|---|
| `cargo build --release` | PASS |
| `cargo test` | 493/493 PASS |
| `cargo run -- --version` | akar 0.33.0 (pre-bump) |
| `cargo run -- doctor` | PASS (advisory: LEARNING_PATCHES split-rule) |
| `cargo run -- status` | HEALTHY, SPLIT_TASK (from existing patch) |
| `cargo run -- request` | wrote NEXT_RUN.md |
| `cargo run -- request --check` | PASS |
| `cargo run -- governor --json --no-exit-code` | SPLIT_TASK |
| `cargo run -- hooks --check` | PASS (source-tree) |
| `cargo run -- eval` | 28/28 PASS |

## 16. Files changed

| File | Change |
|---|---|
| `docs/ALPHA_USAGE.md` | NEW — full stable advisory alpha definition |
| `docs/audits/AKAR_V0_34_STABLE_ADVISORY_ALPHA_FREEZE_REPORT.md` | NEW — this report |
| `README.md` | Added stable advisory alpha section near top |
| `docs/INSTALL.md` | Added alpha note, hook wiring clarification, .akar/ handling note |
| `CHANGELOG.md` | Added v0.34.0 entry |
| `Cargo.toml` | Bumped to v0.34.0 |

No src/ files modified. No runtime behavior changed.

## 17. Honest conclusion

AKAR v0.34.0 is a docs-only freeze that draws a clear line around what's proven and what's not. The CLI advisory loop is stable across Rust and Node project types. Project-kind awareness, diff budget measurement, governor decisions, and NEXT_RUN compilation all work correctly. The hook templates are installable and checkable with an always-available embedded fallback.

The largest gap is live hook-integrated telemetry: v0.33 was CLI-only, and no external-repo Claude Code session with wired PreToolUse has been dogfooded yet. This is the most important next step — v0.35.0.

After that, Python dogfood (v0.36.0) and multi-task session dogfood (v0.37.0) close the remaining gaps before v1.0.0 review. Multi-agent, token optimization, and platform adapters are deferred until the single-agent loop is proven solid.
