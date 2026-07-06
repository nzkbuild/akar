# AKAR v0.27.0 — Second External Dogfood Trial Report

## 1. Executive verdict

AKAR's advisory loop (init → hooks → snapshot → task-threaded request → fix → postmortem) completed successfully end to end on a fresh non-Rust external repo, and the v0.26.0 task-threading feature worked exactly as designed. Two new pieces of real friction were found: `akar init` leaves `.akar/` untracked with no `.gitignore`, which makes the very next command (`preflight --snapshot`) refuse on a "dirty" tree that is dirty only because of AKAR's own output; and the PreToolUse hook resolves its log directory from the Claude Code session's cwd, not the target repo, so hook events for commands run against an external repo are written into the wrong project's `.akar/HOOK_EVENTS.jsonl`. Neither is a safety failure — both are usability gaps that should be fixed before a third trial.

## 2. AKAR baseline

Phase 0, run from `C:\Users\nbzkr\Coding\akar` before any fixture changes:

```
git log --oneline -5   → 5554899 (HEAD), a470ef9, a3c2af8, 7ef56e2, f8525f1
git status              → working tree clean, 21 commits ahead of origin/master
cargo run -- --version  → akar 0.26.0
cargo test              → test result: ok. 439 passed; 0 failed
cargo run -- doctor      → doctor: OK/WARN sections all PASS except one pre-existing advisory
                           (LEARNING_PATCHES.md active split-rule entry)
cargo run -- status      → status: HEALTHY, doctor OK, bootstrap OK, readiness READY
cargo run -- request "dogfood baseline check"
                          → wrote .akar/NEXT_RUN.md, decision SPLIT_TASK (pre-existing active
                            split-rule entry in this repo's own LEARNING_PATCHES.md)
cargo run -- request --check → NEXT_RUN check: PASS
cargo run -- eval        → overall: PASS (28/28)
cargo run -- hooks --check → status: PASS, source: source-tree
```

No AKAR source files were touched during Phase 0.

## 3. Fixture repo description

- Path: `C:\Users\nbzkr\Coding\akar-dogfood-js-fixture` (sibling to the AKAR repo, outside it)
- Language: Node.js (no framework, no external npm dependencies)
- Test command: `npm test` (runs `node test.js`, plain `assert`-based, no test runner installed)
- Files: `package.json`, `math.js` (one function, `add(a, b)` deliberately implemented as `a * b`), `test.js` (one assertion, `add(2, 3) === 5`)
- Initial failing test: confirmed by running `node test.js` directly before AKAR touched the repo — `AssertionError: add(2, 3) should equal 5` (`6 !== 5`), exit code 1
- Intended task: "fix one small failing test in the dogfood JS fixture"
- Git state: initialized fresh, one commit with the buggy fixture, clean tree at that point
- No secrets, no real user data, no network calls in the test path
- Hooks were installable from the release binary — see §5

## 4. Setup path result

`akar init` ran successfully and created `.akar/` with `NEXT_RUN.md`-adjacent scaffolding, but printed `warnings: - templates directory not found` and doctor immediately reported the expected missing-baseline/missing-NEXT_RUN warnings for a fresh repo. This warning is accurate (there is no Rust source tree here for AKAR to find bundled templates in) but is not obviously actionable to a first-time user — it doesn't say whether that's expected for a non-Rust project or a misconfiguration.

## 5. Hook template install/check result

- `akar hooks --check` before install: PASS, `source: embedded` — confirms the v0.25.0 embedded-fallback fix still works on a completely fresh external repo with no AKAR source tree nearby.
- `akar hooks --install` first attempt: prompted `Type INSTALL to confirm, or anything else to cancel` and piping no input cancelled it (`cancelled — no changes made`) — this is correct confirm-before-write behavior, not a bug, but it means non-interactive/scripted dogfood runs need `echo INSTALL | akar hooks --install`.
- Retried with `echo INSTALL | akar hooks --install`: succeeded, copied `pre-tool-call.sh` and `pre-tool-call.ps1` into `.akar/hooks/`, printed the manual `~/.claude/settings.json` wiring example, and explicitly stated it will NOT edit `~/.claude/settings.json`.
- `akar hooks --check` after install: PASS, `source: project .akar/hooks` — correctly picked up the newly installed project-local templates over the embedded fallback.

## 6. Doctor/status/governor before task

Immediately after `hooks --install`, `akar doctor` returned WARN with `[WARN] NEXT_RUN.md present — missing`, `[WARN] DIFF_BASELINE.json — missing`, `[PASS] LEARNING_PATCHES.md: absent`, hooks section all PASS, telemetry both absent (accurate — no events yet), git working tree clean, `[WARN] cargo project: no Cargo.toml`. `akar status` reported `HEALTHY` with governor decision `SNAPSHOT_NOW` (no baseline, clean tree) — consistent and correct for a fresh repo with no baseline yet.

## 7. NEXT_RUN task-threading quality

First `akar preflight --snapshot "fix one small failing test in the dogfood JS fixture"` attempt was refused: `preflight --snapshot: working tree is dirty` because `.akar/` itself (containing the freshly installed hooks and generated files) was untracked with no `.gitignore` — see §13 for the friction detail and the fix applied (adding `.akar/` to `.gitignore` and committing it, exactly as a real user would).

After that, `preflight --snapshot` succeeded (task classified Bugfix, Low risk, 1-3 files / 5-60 LOC budget, verification correctly identified as `npm run build` / `npm test` — preflight's task classifier handled the non-Rust project correctly). `akar request "fix one small failing test in the dogfood JS fixture"` then reported governor decision READY (baseline present, tree clean) and wrote `.akar/NEXT_RUN.md`.

Inspecting the compiled `.akar/NEXT_RUN.md`:
- `## Current State` contained `- requested task: fix one small failing test in the dogfood JS fixture` — present, correct.
- `## Objective` contained `Continue the scoped task without broadening the work.` followed by `- Task: fix one small failing test in the dogfood JS fixture` — present, correct, and the continue-class wording (not the stop-class "blocker primary" wording) was used correctly for the READY decision.
- All safety sections (Hard Rules, Allowed Commands, Forbidden Commands, Stop Conditions, Verification Required) were present and intact.
- `akar request --check` returned `NEXT_RUN check: PASS` on all four sub-checks (sections, minimum content, safety contract, decision consistency).

One quality gap found: `## Allowed Commands` and `## Verification Required` are hardcoded to Rust/cargo commands (`cargo build --release`, `cargo test`, `cargo run -- ...`) even though this is a Node.js project and `preflight` itself correctly detected `npm test`/`npm run build` as the verification commands for the same task. NEXT_RUN's compiled sections do not read from the same task-classification logic that `preflight` uses, so a Node.js/JS/Python dogfood user gets an Objective that mentions their real task but a Verification Required list that names the wrong toolchain entirely.

## 8. Task execution summary

Performed the smallest possible fix: `math.js` line 2, `return a * b;` → `return a + b;`. One line changed. No other files touched. No destructive commands run. AKAR was not modified from inside the fixture.

## 9. Test before/after

- Before: `node test.js` → `AssertionError [ERR_ASSERTION]: add(2, 3) should equal 5` (`6 !== 5`), process exit 1.
- After: `npm test` → `PASS: add(2, 3) === 5`, process exit 0.

## 10. Diff/postmortem result

`git diff --stat` after the fix: `math.js | 2 +-`, 1 file changed, 1 insertion, 1 deletion. `akar postmortem --diff --baseline` reported `status: PASS` with `actual: 1 files, 1 added, 1 deleted (2 total changed LOC)` against the `3 files, 60 LOC` Bugfix budget — correctly well within budget. `akar learn --list` reported 0 total/active/resolved entries (no learning patch file present, correct for a clean successful fix). After the fix, `akar governor --json --no-exit-code` correctly returned `RUN_POSTMORTEM` (baseline present, tree dirty again post-fix) until the postmortem was run.

## 11. Hook evidence result

The Claude Code PreToolUse hook is active in this session (`~/.claude/settings.json` has a `PreToolUse` → `Bash` → `pre-tool-call.ps1` entry pointing at the AKAR source tree's template). Checking `.akar/HOOK_EVENTS.jsonl` inside the fixture found it absent. Checking AKAR's own `.akar/HOOK_EVENTS.jsonl` instead found 454 parseable lines including entries whose `command_preview` explicitly referenced the fixture path (e.g. `cd ".../akar-dogfood-js-fixture" && npm test`). Root cause: the installed hook script (`templates/hooks/pre-tool-call.ps1`) resolves its log directory via `Join-Path (Get-Location) ".akar"`, and `Get-Location` inside the Claude Code hook process resolves to the Claude Code session's own working directory (the AKAR repo), not the working directory of the Bash command actually being classified. So every hook event during this entire dogfood trial — including ones for commands run against the external fixture — was logged into AKAR's own `.akar/HOOK_EVENTS.jsonl`, not the fixture's. `akar doctor` in the AKAR repo confirmed those 454 lines are structurally parseable (`[PASS] HOOK_EVENTS.jsonl: 454 event line(s) parseable`). This is real, reportable friction, not a safety issue — no destructive command was misclassified, but hook evidence collected against an external repo does not end up where a user would look for it (that repo's own `.akar/HOOK_EVENTS.jsonl`).

## 12. What AKAR helped with

- `preflight` correctly classified the JS Bugfix task's risk (Low), budget (1-3 files, 5-60 LOC), and real verification commands (`npm test`/`npm run build`), demonstrating the classifier is not Rust-specific even though NEXT_RUN's static command lists are.
- The baseline/snapshot → postmortem loop correctly measured the actual 1-file/2-LOC diff against budget and returned PASS, giving an honest, evidence-based confirmation that the fix stayed in scope.
- Task threading did exactly what v0.26.0 designed it to do: the compiled prompt named the real task in both Current State and Objective without ever touching or weakening the safety sections.
- `hooks --check`'s embedded fallback meant hook templates were available and installable on a repo with zero AKAR source tree nearby, with zero Claude-settings mutation.

## 13. What AKAR made worse

- `akar init` on a fresh repo leaves `.akar/` completely untracked with no `.gitignore` guidance, so the very first `preflight --snapshot` call after setup refuses with "working tree is dirty" — but the only dirty content is AKAR's own generated output. A first-time user following the documented setup sequence (`init` → `hooks --install` → `preflight --snapshot`) hits a refusal caused entirely by AKAR itself, with no advisory pointing at the fix (add `.akar/` to `.gitignore`). This is the same category of friction as the v0.24 Cargo.lock finding but for AKAR's own directory.
- The PreToolUse hook logs to the wrong project's `HOOK_EVENTS.jsonl` when the Claude Code session's cwd differs from the target repo being dogfooded, silently. There is no warning anywhere in the loop that this is happening.

## 14. Confusing or misleading output

- `akar init`'s `warnings: - templates directory not found` gives no indication of whether this matters for a non-Rust project — a user can't tell if it's benign or something to fix.
- NEXT_RUN's `## Allowed Commands`/`## Verification Required` naming `cargo build --release`/`cargo test` on a project with no Cargo.toml at all is actively misleading — a Claude session handed this prompt could reasonably try to run cargo commands that will simply fail with "no such file" in a Node.js repo.

## 15. Manual rescue required

One manual rescue was required: adding `.akar/` to `.gitignore` and committing it, to get past the dirty-tree refusal in §13. This is exactly the kind of manual step AKAR's own advisory model expects a human to perform (AKAR explicitly does not decide what goes in `.gitignore` on the user's behalf) — but it was undocumented, so it took a live investigation (checking `git status --porcelain`) to diagnose rather than being called out by AKAR itself the way the v0.26.0 Cargo.lock advisory calls out that specific case.

## 16. Alpha readiness verdict

AKAR is usable as an advisory alpha loop on a fresh external non-Rust repo: every command in the documented sequence produced correct, safety-preserving output, and the trial completed from init through a real fix to a passing postmortem with no destructive action and no Claude Code configuration changes. It is not yet frictionless — the two findings in §13 are real day-one blockers/confusions for a brand-new user on any non-Rust or freshly-initialized repo, and should be fixed before recommending AKAR for unattended external dogfood.

## 17. Required fixes before next dogfood

1. `akar init` (or `akar doctor`) should proactively advise creating a `.gitignore` entry for `.akar/` (or a subset of it) on a fresh repo, the same way v0.26.0 added a targeted Cargo.lock advisory — without auto-creating the `.gitignore` itself.
2. The PreToolUse hook template should resolve its log directory from the tool call's actual working directory (if available from the hook input) rather than `Get-Location` of the hook process, or the discrepancy should be documented so users know hook evidence for external repos lands in the Claude Code session's own project, not the target repo.
3. NEXT_RUN's compiled `## Allowed Commands`/`## Verification Required` sections should use the same task/project-type classification `preflight` already has (Rust vs. npm vs. other) instead of hardcoding cargo commands unconditionally.

## 18. Honest conclusion

This was a clean run of the intended loop with no safety incidents and a real, correctly-scoped fix. Task threading works. The advisory discipline (budget, snapshot, postmortem) works. But this trial surfaced two friction points that would confuse or block a genuinely new external user before they get anywhere near their actual task, both stemming from the same root pattern the v0.24 and v0.26 findings already identified: AKAR's own generated state and hook logging make assumptions (tracked-vs-untracked, cwd-is-the-target-repo) that don't hold on every setup, and AKAR doesn't yet call those assumptions out to the user the way it already does for Cargo.lock.

## 19. Next recommended release

v0.28.0 should fix the `.akar/`-untracked dirty-tree friction (§17 item 1) as a targeted advisory, mirroring the v0.26.0 Cargo.lock advisory pattern exactly. The hook cwd-resolution issue (§17 item 2) and the NEXT_RUN command-list Rust-only hardcoding (§17 item 3) are good candidates for the release after that, since both require slightly more invasive changes (hook template behavior, NEXT_RUN compiler task-type awareness) than a single advisory line.
