# Changelog

## v0.10.0 — 2026-07-05
Full Loop Proof with Auto-Hook Active. Complete AKAR baseline loop proved in one clean session with PreToolUse hook firing automatically throughout: committed clean tree → akar status READY → akar preflight --snapshot → added one row to docs/INSTALL.md version table (1 file, 1 LOC) → 8 Bash tool calls each auto-logged as ALLOW by hook → akar postmortem --diff --baseline → PASS (1 file / 1 LOC against budget 3 files / 60 LOC). Added docs/audits/AKAR_V0_10_FULL_LOOP_AUTO_HOOK_REPORT.md. No runtime behavior changed. 251 tests passing.

## v0.9.0 — 2026-07-05
First Auto-Hook Evidence. PreToolUse hook fired automatically on every Bash tool call throughout the session. Safe commands (echo, cargo build, cargo test) logged ALLOW/exit 0. `rm -rf /` logged BLOCK/exit 2 — Claude Code blocked execution before rm ran. Hook was wired manually by user into ~/.claude/settings.json. AKAR did not modify Claude Code configuration. akar.exe must be on the subprocess PATH (C:\Users\nbzkr\bin\) for classification to run. Previous attempt failed because akar was not on subprocess PATH — documented honestly. Evidence in docs/audits/AKAR_V0_9_AUTO_HOOK_EVIDENCE.md. No runtime behavior changed. 251 tests passing.

## v0.8.2 — 2026-07-05
Hook Evidence Logging. Both hook templates now append one JSONL event to `.akar/HOOK_EVENTS.jsonl` per call: timestamp, hook, tool_name, command_preview (truncated 300 chars, secrets redacted), decision (ALLOW/WARN/BLOCK/SKIP), exit_code. Creates `.akar/` if missing. Does not log full stdin JSON blob. Added `.akar/HOOK_EVENTS.jsonl` to `.gitignore`. Updated `akar hooks --check` to verify templates read stdin, write to HOOK_EVENTS.jsonl, and use exit 2. Added 4 new Rust tests. Exit behavior unchanged: non-Bash→exit 0, safe→exit 0, BLOCKED→exit 2. AKAR does not send hook telemetry anywhere. 251 tests passing.

## v0.8.1 — 2026-07-05
Hook JSON Compatibility. Fixed both hook templates to read Claude Code PreToolUse JSON from stdin (not $1/argv). Templates now extract tool_name and tool_input.command, skip non-Bash tools immediately, pass only the command string to `akar safety`, and exit 2 (not exit 1) for BLOCKED commands — exit 1 does not block in Claude Code. Added Rust hook JSON parsing module (parse_hook_event, hook_decision) with 12 new tests covering: JSON parsing, Skip/Check/Allow decisions, safety integration for cargo test and rm -rf /, exit-2 enforcement, and stdin reading. Updated format_hooks_install snippet to show correct PreToolUse/matcher/Bash shape. 247 tests passing.

## v0.8.0 — 2026-07-05
First Verified Baseline Loop. Proved the full AKAR loop on a real clean session: committed all v0.7.1 work, confirmed readiness READY, ran `akar preflight --snapshot "fix one small documentation typo"`, corrected stale version string in `docs/INSTALL.md` (1 file, 2 LOC), ran `akar safety "cargo test"` (Safe/allowed), ran `akar postmortem --diff --baseline` — verdict PASS (1 file / 2 LOC against budget of 3 files / 60 LOC). Added `docs/audits/AKAR_V0_8_FIRST_BASELINE_LOOP_REPORT.md`. No runtime behavior changed. 235 tests passing.

## v0.7.1 — 2026-07-05
Full Loop Readiness. Corrected v0.7.0 overclaim: report now states partial session evidence, not full verified loop. Added baseline loop readiness section to `akar status`: reports git repo detected, working tree clean, baseline file present, and readiness (READY/BLOCKED/UNKNOWN). Uses read-only git commands only. Full clean-baseline proof remains the next milestone. Docs updated with full baseline loop readiness instructions.

## v0.7.0 — 2026-07-05
Verified Session Report. Added `docs/audits/AKAR_V0_7_VERIFIED_SESSION_REPORT.md` documenting a real end-to-end AKAR loop attempt with honest findings: snapshot refusal on dirty tree works, safety gate blocks rm -rf /, diff measurement reports EXCEEDED correctly, 229 tests pass, 28/28 evals pass. Full baseline loop was not run (session built the feature itself). Hook was manually invoked, not auto-installed. AKAR stayed advisory-only throughout. No runtime behavior changed. 229 tests passing.

## v0.6.2 — 2026-07-05
Diff Baseline Snapshot. Added `akar preflight --snapshot "<task>"`: checks working tree is clean, reads HEAD commit, writes `.akar/DIFF_BASELINE.json` with timestamp, prompt, HEAD, task type, and diff budget. Added `akar postmortem --diff --baseline`: reads saved baseline, measures diff from baseline HEAD to current working tree, compares against saved budget, prints PASS/EXCEEDED/UNKNOWN, appends learning patch on EXCEEDED. Added `DIFF_BASELINE.json` to `.gitignore`. Refactored learning patch writing into shared helper. 229 tests passing.

## v0.6.1 — 2026-07-05
Explicit diff budget selection. Added `--task <type>` argument to `akar postmortem --diff`. Supported types: bugfix, feature, refactor, security, migration, dependency, frontend, docs, test, config, unknown (with aliases). Default remains Bugfix with explicit hint. Invalid task exits non-zero with valid options listed. Learning note includes task type and full rule: "Next prompt must reduce scope or split the task." Uses existing contract.rs budget tiers — no second budget table. Updated README. 221 tests passing.

## v0.6.0 — 2026-07-05
Diff Budget Measurement. Added `src/diff_budget.rs`: measures actual git working tree diff using `git diff HEAD --numstat` and `--name-only`, parses file count and LOC, compares against preflight diff budget. Added `akar postmortem --diff`: prints measured files/LOC vs budget, reports PASS/EXCEEDED/UNKNOWN. Appends learning patch to `.akar/LEARNING_PATCHES.md` when EXCEEDED (rule: "Next prompt must reduce scope or split the task"). Does not enforce, block, or revert changes. Updated README and docs. 213 tests passing.

## v0.5.2 — 2026-07-05
Loop Engineering Doctrine. Added `docs/architecture/AKAR_LOOP_ENGINEERING.md` defining the development loop: freeze current truth, choose one narrow release target, write deterministic instructions, require verification, audit diffs, record baseline, generate next prompt from evidence. Documents Prompt Rules, Release Loop, and Human Audit sections. Added loop engineering paragraph to README. No runtime behavior changed. 203 tests passing.

## v0.5.1 — 2026-07-05
Safety gate fix. Classified destructive filesystem wipe commands as BLOCKED: `rm -rf /`, `rm -rf /*`, `sudo rm -rf /`, `sudo rm -rf /*`, `rm -fr /`, `rm -fr /*`, `del /s /q C:\`, `Remove-Item -Recurse -Force C:\`, `Remove-Item -Recurse -Force /`. Normal dev commands (cargo, git, npm) unaffected. Added 10 new safety tests. 203 tests passing.

## v0.5.0 — 2026-07-05
Real hook integration. Added `templates/hooks/pre-tool-call.sh` and `templates/hooks/pre-tool-call.ps1` — both call `akar safety` and exit non-zero only for BLOCKED commands. Added `hooks.rs` module with `--check` (verifies templates exist and contain `akar safety` call) and `--install` (copies templates into `.akar/hooks/` after user types INSTALL, creates backups before overwrite). AKAR does not modify `~/.claude/settings.json` or install hooks automatically. Updated README and docs. 194 tests passing.

## v0.4.1 — 2026-07-05
Cleanup leftovers. Deleted `circuit_breaker.rs` (unused in production paths). Removed module declaration from `main.rs`. Fixed `OPERATING_MODEL.md` circuit breaker section (was claiming behavior that never existed). Updated `AKAR_ADOPTION_NOTES.md` module list. Added clarifying comment to `context_pack.rs` that it builds a path tier list only and does not read file contents. No behavior added or changed. 190 tests passing.

## v0.4.0 — 2026-07-05
Honest Scaffold release. Fixed `akar run` UX to clearly state advisory/scaffold mode with no code execution. Removed dead model drift code (SessionFingerprint, detect_drift, calibrate_from_prompt, git helpers). Simplified design.rs to DNA-check only (removed frontend file scanner). Deprecated event-count request pressure inference — pressure mode now requires explicit used/limit counts. Fixed preflight skill conflict noise — scans project-local skills only instead of all 200+ global skills. Fixed README and docs to replace overclaims ("enforces diff budget" → "reports a diff budget", "installs hooks" → "prints hook instructions", "proposes learning" → "writes generic learning notes"). Added LEARNING_PATCHES.md and NEXT_RUN.md to .gitignore. Added audit documents and architecture freeze proposal. 190 tests passing.

## v0.3.0 — 2026-07-04
Added `akar init` onboarding command (bootstrap + doctor + next-steps guide, shell detection, --claude flag). Expanded INSTALL.md with runtime layout, configuration precedence, migration, and version compatibility sections. Expanded OPERATING_MODEL.md with onboarding (D), lifecycle (E), passive runtime (F), Claude integration (G), and health & recovery (H) sections. Fixed .gitignore to commit .akar/*.md templates while ignoring EVENT_LOG.jsonl and generated files. Updated docs/README.md with full doc index.

## v0.2.2 — 2026-07-04
Public GitHub polish: repo cleanup, README rewrite, CONTRIBUTING, SECURITY, CHANGELOG. Removed tracked runtime artifacts (.akar/, reports/). No new runtime features.

## v0.2.1 — 2026-07-04
Install guide, operating model, evaluation plan docs. Updated README with full command table and docs links.

## v0.2.0 — 2026-07-04
First stable runtime. Added `akar run` (stable workflow command). Upgraded `akar status` to show doctor/bootstrap/telemetry/postmortem/skills/request in one view. 28/28 evals passing.

## v0.1.9 — 2026-07-04
Mission preflight wiring. `akar preflight "<task>"` combines contract classification, request pressure, skill intelligence, and verification into one pre-execution strategy report.

## v0.1.8 — 2026-07-04
Skill intelligence runtime wiring. `akar skills` scans global + project commands, detects role conflicts (methodology + execution controller combo), writes SKILL_INVENTORY.md.

## v0.1.7 — 2026-07-04
Request Intelligence v0. `akar request` shows pressure mode (Normal/Saver/Compact/Boundary/Resume) from explicit counts or local telemetry signals.

## v0.1.6 — 2026-07-04
Learning Patch v0. `akar learn` reads postmortem evidence and proposes LP-NNNN patches for degraded/failed/unknown outcomes.

## v0.1.5 — 2026-07-04
Postmortem wiring. `akar postmortem` reads EVENT_LOG.jsonl and classifies latest outcome as clean/degraded/failed/unknown.

## v0.1.4 — 2026-07-04
Runtime telemetry wiring. `akar telemetry` shows event log summary. Mission flow appends compact events to `.akar/EVENT_LOG.jsonl`.

## v0.1.3 — 2026-07-04
Runtime hygiene. Zero warnings. .gitignore policy. Release checklist.

## v0.1.2 — 2026-07-04
Real `akar bootstrap`. Creates `.akar/` and copies 9 memory templates idempotently. `akar doctor` returns OK after bootstrap.

## v0.1.1 — 2026-07-04
Architecture refinement. AKAR OS framing, RFC system, Skill Intelligence docs, 25 evals.

## v0.1.0 — 2026-07-03
Initial Rust CLI scaffold. 15 modules, 121 tests, 20 evals passing. Core commands: status, doctor, bootstrap, verify, eval, mission, safety, skills, calibrate, hooks.
