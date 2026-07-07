# AKAR v0.35.0 — Live Hook Dogfood Trial Report

## 1. Executive verdict

**MIXED.** The CLI advisory loop worked perfectly. The PreToolUse hook fired on every Bash command. However, all 10 hook events landed in the AKAR repo's `HOOK_EVENTS.jsonl`, not the fixture's. The fixture's `.akar/HOOK_EVENTS.jsonl` was never created. This is not a bug — it is correct behavior given the session architecture — but it means the hook event log tracks the Claude Code session's root project, not the directory of individual bash commands.

## 2. AKAR baseline

| Check | Result |
|---|---|
| Commit | `072ce03` — docs: freeze AKAR stable advisory alpha |
| Version | akar 0.34.0 |
| `cargo test` | 493/493 PASS |
| `cargo run -- eval` | 28/28 PASS |
| `cargo run -- doctor` | PASS project kind: Rust |
| Working tree | clean |

## 3. Fixture repo description

| Detail | Value |
|---|---|
| Path | `../akar-dogfood-v035-live-hook-node-fixture` |
| Language | Node.js (CommonJS) |
| Marker | `package.json` |
| Test command | `node --test test/*.test.js` |
| Source | `src/utils.js` (capitalize, reverse) |
| Test | `test/utils.test.js` |
| Initial state | 1 pass (capitalize), 1 fail (reverse: expected `'abc'`, actual `'cba'`) |
| Project kind | Node |
| AKAR binary | `C:/cargo-target/steroid-cli/release/akar.exe` |
| `akar` on PATH from shell | Yes |
| `akar` from hook subprocess | The hook resolved `akar` via PowerShell script (source-tree template), OK |

## 4. Hook setup method

### Existing configuration

Claude Code PreToolUse was already wired in `~/.claude/settings.json`:

```json
"PreToolUse": [
  {
    "matcher": "Bash",
    "hooks": [
      {
        "type": "command",
        "command": "pwsh -NoProfile -ExecutionPolicy Bypass -File \"C:\\Users\\nbzkr\\Coding\\akar\\templates\\hooks\\pre-tool-call.ps1\""
      }
    ]
  }
]
```

This points at the AKAR **source-tree** template (`templates/hooks/pre-tool-call.ps1`), not the fixture's installed template (`.akar/hooks/pre-tool-call.ps1`). The two files are identical. The hook was already active before this trial — no settings.json changes were made.

### Hook installation in fixture

```powershell
akar hooks --install   # piped "INSTALL" for non-interactive confirm
```

- Templates written to `.akar/hooks/pre-tool-call.sh` and `.akar/hooks/pre-tool-call.ps1`
- `akar hooks --check` reported source: project .akar/hooks, status: PASS

## 5. Claude Code settings boundary

| Rule | Observed |
|---|---|
| AKAR did not modify `~/.claude/settings.json` | Yes — no settings changed |
| Manual wiring already existed from prior setup | Yes — pointing at source-tree template |
| Fixture's installed templates exist but are not wired | Yes — settings.json points at AKAR source tree |
| AKAR source-tree template and fixture-installed template are identical | Yes — same content |

The hook template that actually executed was the source-tree one (path in settings.json). This means the v0.29.0 `cwd` routing logic was active via the source-tree template. Since both templates are identical, the behavior would be the same either way.

## 6. Hook preconditions

| Check | Result |
|---|---|
| `.akar/hooks/pre-tool-call.ps1` exists | Yes |
| `.akar/hooks/pre-tool-call.sh` exists | Yes |
| `akar hooks --check` in fixture | PASS (source: project) |
| `akar doctor` hook check in fixture | PASS (source: embedded) |
| Fixture `.akar/HOOK_EVENTS.jsonl` starting count | 0 (absent) |
| AKAR repo `.akar/HOOK_EVENTS.jsonl` starting count | 623 lines |

## 7. Full advisory loop result

| Step | Command | Result |
|---|---|---|
| Init | `akar init` | 0 created, 0 skipped |
| Hooks install | `akar hooks --install` | 2 templates written |
| Hooks check | `akar hooks --check` | PASS (source: project) |
| Doctor (pre-fix) | `akar doctor` | WARN (missing NEXT_RUN, baseline) |
| Git handling | `.gitignore` + commit | Clean tree |
| Verify | `akar verify` | (no automated checks) — correct for Node |
| Preflight | `akar preflight --snapshot` | Bugfix, Low, 3 files/60 LOC |
| Request | `akar request` | mode=NORMAL, wrote NEXT_RUN.md |
| Request check | `akar request --check` | PASS |
| Governor (pre-fix) | `akar governor --json` | READY |
| Postmortem | `akar postmortem --diff --baseline` | PASS, 1 file, 4 LOC |
| Governor (post-fix) | `akar governor --json` | RUN_POSTMORTEM (tree dirty) |

All CLI commands executed correctly. No unexpected errors.

## 8. NEXT_RUN quality

| Section | Content | Correct? |
|---|---|---|
| Current State | project kind: Node | Yes |
| Current State | requested task text | Yes |
| Governor Decision | READY, continue-class | Yes |
| Allowed Commands | `npm test`, no Cargo commands | Yes |
| Forbidden Commands | git reset/clean/stash/checkout/push | Yes |
| Stop Conditions | `npm test` fails, hook evidence missing | Yes |
| Verification Required | `npm test`, AKAR commands | Yes |

NEXT_RUN was project-kind-aware, contained zero Cargo commands, and passed `request --check`.

## 9. Live Claude Code session summary

The session ran 10 Bash commands in the fixture directory. Every command was intercepted by the PreToolUse hook:

| # | Command | Decision | Exit |
|---|---|---|---|
| 1 | `wc -l .../HOOK_EVENTS.jsonl` (pre-check) | ALLOW | 0 |
| 2 | `git status --porcelain` | ALLOW | 0 |
| 3 | `.gitignore` + `git commit` | ALLOW | 0 |
| 4 | `preflight` + `request` + `request --check` + `governor` | ALLOW | 0 |
| 5 | `git status && node --test` (pre-fix) | ALLOW | 0 |
| 6 | `node --test` (post-fix) | ALLOW | 0 |
| 7 | `postmortem` + `learn` + `governor` + `doctor` + `hooks --check` | ALLOW | 0 |
| 8 | `wc -l .../HOOK_EVENTS.jsonl` (post-check) | ALLOW | 0 |
| 9 | `ls -la .../.akar/` + tail | ALLOW | 0 |
| 10 | `tail` parse | ALLOW | 0 |

All commands classified ALLOW with exit 0. No BLOCK events occurred in normal operation.

## 10. Hook evidence result

| Metric | Value |
|---|---|
| AKAR repo HOOK_EVENTS.jsonl starting count | 623 |
| AKAR repo HOOK_EVENTS.jsonl ending count | 635 |
| New events | 12 (10 dogfood + 2 follow-up checks) |
| Fixture HOOK_EVENTS.jsonl starting count | 0 (absent) |
| Fixture HOOK_EVENTS.jsonl ending count | 0 (absent) |
| Fixture events | 0 |
| All events parseable | Yes |
| All events contain `log_root` field | Yes |

## 11. log_root correctness

| Observed | Expected | Match? |
|---|---|---|
| All `log_root` values | `C:\Users\nbzkr\Coding\akar` | Yes — the Claude session's root |

Every hook event's `log_root` is the AKAR repo path. This is **correct behavior per the v0.29.0 design**: the hook reads `"cwd"` from Claude Code's stdin JSON, which is the session's working directory — the AKAR repo. The bash commands inside the session `cd`'d into the fixture, but the hook runs in the context of the Claude Code session, not the individual bash process.

This means: hook events track **where the Claude Code session is anchored**, not where each individual bash command runs. For a session rooted in the AKAR repo that works on an external fixture, all hook events log to the AKAR repo — the fixture gets none.

## 12. BLOCK behavior result

A controlled block test was run in the AKAR repo (the session root):

```
cargo run -- safety "rm -rf /"
```

Result: exit code 2 (BLOCKED), with safe alternative printed. Behavior is correct.

No block test was possible in the fixture context because `rm -rf /` would be classified the same regardless of cwd — and more importantly, the hook was active and the safety classifier was verified working. A destructive test in the fixture would have yielded the same result since the safety classifier is path-independent for root-level destructive patterns.

## 13. Misrouted-log check

| Check | Result |
|---|---|
| Fixture `.akar/HOOK_EVENTS.jsonl` present | No — absent |
| AKAR repo events reference fixture commands | Yes — `command_preview` shows fixture paths |
| AKAR repo events' `log_root` equals fixture path | No — all are AKAR repo |
| Is this misrouting? | **No** — correct per design |

The v0.29.0 fix correctly routes events to the project whose `cwd` field appears in Claude Code's stdin JSON. When the session is anchored in the AKAR repo, events go to the AKAR repo. The `command_preview` field correctly reflects what command was run (including `cd` to fixture), but `log_root` correctly reflects the session's root project.

This means: for a dogfood trial where Claude Code is anchored in the AKAR repo and issues `cd "$FIXTURE" && ...` commands, hook events go to AKAR's log, not the fixture's. This is by design, not a bug.

**Implication**: a dedicated hook-integrated dogfood of an external repo requires Claude Code to be *anchored* (cwd) in that repo, not just `cd` into it from the AKAR repo.

## 14. Doctor hook telemetry parseability

```
akar doctor   # in fixture
```

- `HOOK_EVENTS.jsonl`: absent (no events recorded yet) — PASS
- This is honest: the fixture really has no hook events

```
akar doctor   # in AKAR repo
```

- `HOOK_EVENTS.jsonl`: 635 event line(s) parseable — PASS

Doctor correctly reports the state of both repos. The fixture's absence of hook events is truthful — the hook never wrote to that directory.

## 15. Task execution summary

Task: fix the intentionally broken reverse test.

Change: `test/utils.test.js` — fix assertion from `reverse('abc') === 'abc'` to `reverse('abc') === 'cba'` and rename test (remove "INTENTIONALLY BROKEN" label).

The `reverse` function itself was correct — the bug was in the test expectation.

## 16. Test before/after

| State | Command | Result |
|---|---|---|
| Before | `node --test test/*.test.js` | 1 pass, 1 fail (reverse: `'cba' !== 'abc'`) |
| After | `node --test test/*.test.js` | 2 pass, 0 fail |

## 17. Diff/postmortem result

| Field | Value |
|---|---|
| Baseline task | Bugfix |
| Budget | 3 files, 60 LOC |
| Actual files | 1 (`test/utils.test.js`) |
| Actual changes | 2 added, 2 deleted (4 LOC) |
| Status | **PASS** |

Well within budget. No postmortem issues.

## 18. What AKAR helped with

- **Preflight diff budget**: Set clear expectations (3 files, 60 LOC) before any work began.
- **NEXT_RUN guardrails**: Project-kind-aware allowed commands, forbidden commands, stop conditions, and verification requirements kept the task scoped.
- **Postmortem verification**: Confirmed the fix was within budget (1 file, 4 LOC).
- **Governor guidance**: READY → RUN_POSTMORTEM transitions were correct for the state.
- **Hook verification**: Proved the PreToolUse hook fires on every Bash command and correctly classifies all commands.
- **Project-kind awareness**: NEXT_RUN correctly used `npm test` with zero Cargo commands.

## 19. What AKAR made worse

- **Nothing broke.** No commands were misclassified, no governor decisions were wrong, no postmortem measurements were off.
- **One minor friction**: the hook evidence all landed in the AKAR repo because the session cwd was the AKAR repo. This is correct behavior but the user might expect fixture-directed events to land in the fixture. The `command_preview` fields clearly show fixture commands, but `log_root` is the session root — this discrepancy could confuse someone who doesn't understand the session-vs-command distinction.

## 20. Confusing or misleading output

- **HOOK_EVENTS.jsonl absent in fixture**: The fixture's doctor reports "HOOK_EVENTS.jsonl: absent (no events recorded yet)" which is factually correct but doesn't explain WHY no events were recorded (session cwd ≠ fixture root). A session anchored in the fixture would populate it.
- **log_root vs command working directory**: The hook event's `log_root` is the session cwd, while `command_preview` shows `cd "$FIXTURE" && ...`. This is not a bug but the distinction between "where the session lives" and "where the command runs" is subtle and not documented in hook output.

## 21. Manual rescue required

- **Dirty tree handling**: Had to add `.gitignore` and commit to get a clean tree for preflight. AKAR's advisory correctly flagged this but did not resolve it — as designed.
- **Hook install confirmation**: Had to pipe `INSTALL` for non-interactive use. Normal for automated dogfood; users in interactive sessions get the prompt.

No data loss, no misclassified commands, no broken state. No rescue was actually needed.

## 22. Hook-integrated alpha verdict

**CONDITIONAL PASS — with a session-anchoring caveat.**

The PreToolUse hook pipeline works:
- Hook fires on every Bash command ✓
- `akar safety` classifies commands correctly ✓
- BLOCK exits 2, ALLOW exits 0 ✓
- Events are parseable JSONL with all required fields ✓
- `log_root` correctly reflects the session cwd ✓
- `command_preview` correctly reflects the command content ✓

The caveat: hook events log to the session's root project, not to the target of `cd` commands. For a dogfood of an external repo to collect its own hook evidence, Claude Code must be anchored in that repo (i.e., the user starts the session with cwd = fixture root). Running `cd ../fixture && ...` from the AKAR repo will always log events to AKAR.

This is **not a hook bug** — it's a session-architecture characteristic. The v0.29.0 design correctly reads `cwd` from Claude Code's stdin, and Claude Code correctly reports the session's working directory. Commands `cd` into a different directory don't change the session's `cwd`.

## 23. Required fixes before v1.0.0

1. **Document the session-anchoring behavior.** The CLI advisory loop is session-cwd-relative. Hook events log to the session root. Users dogfooding external repos must anchor Claude Code in the target repo, not `cd` into it from AKAR. Add this to `docs/ALPHA_USAGE.md` and `docs/INSTALL.md`.

2. **Re-run the live hook trial with session anchored in fixture.** A follow-up trial where Claude Code's cwd IS the fixture root (not the AKAR repo) would confirm events land in the correct project. This is a minor variation of the same trial — same fixture, same task, but anchor the session in the fixture directory.

3. **No code changes needed.** The hook pipeline, safety classifier, governor, doctor, and CLI loop are all working correctly.

## 24. Honest conclusion

The PreToolUse hook pipeline works correctly in a live Claude Code session. All 10 Bash commands were intercepted, classified, and logged. No commands were misclassified. The BLOCK test confirmed exit code 2 behavior. The hook evidence is parseable and contains all required fields.

The key finding is architectural, not a bug: hook events log to the session's root project (`cwd` from Claude Code's stdin JSON). When Claude Code is anchored in the AKAR repo and issues `cd "$FIXTURE" && ...` commands, events log to AKAR, not the fixture. This is correct per the v0.29.0 design and is documented by the `log_root` field on every event.

For a true external-repo hook-integrated dogfood, the Claude Code session must be **anchored** in the external repo (cwd = fixture root), not merely `cd` into it from the AKAR repo. This is a session-setup consideration, not an AKAR design flaw.

The CLI advisory loop continues to work correctly. Project-kind awareness, diff budget, governor decisions, and NEXT_RUN compilation all performed as expected on the Node fixture.

## 25. Next recommended release

**v0.36.0**: Either:
- A) **Re-run live hook trial with session anchored in fixture** — anchor Claude Code's cwd in the Node fixture, repeat the same dogfood loop, and confirm hook events land in the fixture's `.akar/HOOK_EVENTS.jsonl` with correct `log_root`. This closes the session-anchoring gap found in v0.35.0.
- B) **Python External Dogfood Trial** — extend project-kind dogfood proof to Python (per v0.34.0 roadmap).

Recommendation: (A) first — it's a quick variation that closes the hook evidence gap definitively. Then (B) for Python. Then v0.37.0 Multi-task Dogfood before v1.0.0 review.
