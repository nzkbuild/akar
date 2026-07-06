# AKAR v0.29.0 — Hook Event Target Root Report

## 1. Baseline

Starting point: AKAR v0.28.0, 448 tests passing, clean release build, clean working tree. The v0.27.0 second external dogfood trial reported that PreToolUse hook events for commands run against an external repo were logged to the AKAR repo's own `.akar/HOOK_EVENTS.jsonl` instead of the external repo's, because the hook scripts resolved their log directory from the process cwd (`$(pwd)` / `Get-Location`) — which is the Claude Code session's working directory, not the target repo. This release fixes that by extracting the `"cwd"` field that Claude Code provides in the hook stdin JSON and using it as the log root.

## 2. Dogfood issue addressed

From the v0.27.0 report §11 and §13: "The installed PreToolUse hook resolves its log directory via `Get-Location` of the hook process (the Claude Code session's own cwd), not the working directory of the command actually being classified, so hook events for commands run against an external repo were written into the wrong project's `.akar/HOOK_EVENTS.jsonl`. This is real, reportable friction."

## 3. Solution: extract `cwd` from hook stdin JSON

Claude Code's PreToolUse hook JSON includes a top-level `"cwd"` field (confirmed via the official docs at https://code.claude.com/docs/en/hooks). Both hook templates now extract this field and prefer it over the process cwd:

- **bash**: `CWD_FROM_JSON` extracted via `grep -o '"cwd"[[:space:]]*:[[:space:]]*"[^"]*"'`, validated with `[ -d "$CWD_FROM_JSON" ]`, becomes `LOG_ROOT`.
- **PowerShell**: `$cwdFromJson` extracted via `$json -match '"cwd"\s*:\s*"([^"]+)"'`, validated with `Test-Path -PathType Container`, becomes `$logRoot`.

Fallback: if `"cwd"` is absent from the JSON or the resolved path does not exist as a directory, the process cwd is used (same behavior as before v0.29.0).

## 4. `log_root` field in each event

Every JSONL event line now includes a `"log_root"` field with the chosen path (JSON-escaped). This makes the target project explicit even when inspecting the log from another context — the field answers "which project does this event belong to?"

## 5. Hook behavior unchanged

- Non-Bash tools still log SKIP and exit 0.
- BLOCK still exits 2.
- ALLOW still exits 0.
- `akar safety` classification unchanged — `rm -rf /` is still BLOCKED, `cargo test` is still ALLOWed, regardless of whether `"cwd"` is present in the JSON.

## 6. Rust-side changes

`src/hooks.rs`:
- `HookEvent` gained a `cwd: String` field.
- `parse_hook_event` now calls `extract_json_str_value(json, "cwd")` in addition to the existing `tool_name` and `command` extractions.
- Embedded template tests (`embedded_bash_template_is_nonempty`, `embedded_powershell_template_is_nonempty`) now also assert that the template contains `"cwd"` and `log_root`.

No other Rust source files were touched.

## 7. Tests added/updated

`src/hooks.rs` (+5):
- `parse_hook_event_extracts_cwd_when_present` — JSON with `"cwd":"/home/user/my-project"` yields `e.cwd == "/home/user/my-project"`.
- `parse_hook_event_cwd_empty_when_absent` — JSON without `"cwd"` yields `e.cwd == ""`.
- `parse_hook_event_cwd_does_not_affect_safety_rm_rf` — `rm -rf /` is still BLOCKED when `"cwd"` is present.
- `parse_hook_event_cwd_does_not_affect_safety_cargo_test` — `cargo test` is still ALLOWed when `"cwd"` is present.
- `parse_hook_event_non_bash_skip_unaffected_by_cwd` — Non-Bash Skip decision is unchanged with `"cwd"`.

Total: 448 → 453 tests (+5). No existing tests were modified (beyond the 2 embedded-template tests gaining additional assertions for presence of `"cwd"` and `log_root`).

## 8. Verification

```
cargo build --release        → Finished `release` profile [optimized] target(s)
cargo test                   → test result: ok. 453 passed; 0 failed
akar --version                → akar 0.29.0
akar doctor                   → doctor: WARN (pre-existing advisories in this repo, unrelated)
akar status                   → status: HEALTHY
akar request "dogfood verification" → wrote .akar/NEXT_RUN.md
akar request --check          → NEXT_RUN check: PASS
akar governor --json --no-exit-code → valid JSON, all fields present
akar hooks --check             → status: PASS, source: source-tree
akar eval                      → overall: PASS (28/28)
```

## 9. Honest conclusion

The fix is minimal and surgical: two hook templates gained `"cwd"` extraction (one grep pattern in bash, one regex match in PowerShell), and the Rust-side parser mirror gained the same field for testability. Hook safety behavior is completely unchanged — this release only changes *where* events are logged, not *whether* a command is blocked. The `log_root` field in each event line provides auditability regardless of which context you're reading the log from.

## 10. Next recommended release

The last remaining v0.27.0 finding — NEXT_RUN's hardcoded Rust/cargo command lists on non-Rust projects — should be addressed next, potentially via reusing the existing `preflight` task-type classifier to generate project-appropriate Allowed Commands and Verification Required sections.
