# AKAR Current State — 2026-07-10

Consolidated snapshot for future prompts. See the source audit docs for full detail:
- `AKAR_V0_52_CLAUDE_MD_STALE_CONTEXT_REVISION.md`
- `AKAR_V0_53_ZERO_RELAY_SETUP_FOUNDATION.md`
- `AKAR_V0_53_EXTERNAL_DOGFOOD.md`
- `AKAR_V0_54_ZERO_RELAY_AUTO_CONTEXT_HOOK.md`
- `AKAR_V0_54_EXTERNAL_DOGFOOD.md` (this dogfood)

## Baseline

| Check | Value |
|---|---|
| Commit | `34bb34e` — feat: add AKAR Claude Code auto-context hook prototype |
| Version | `akar 0.54.0` |
| Working tree | dirty (fmt whitespace drift on 26 unrelated src files) |
| `cargo test` | 578 passed, 1 failed (pre-existing: `doctor::ok_when_everything_present_and_valid`) |
| `cargo eval` | 27/28 PASS (pre-existing: `doctor_check`) |
| `cargo build --release` | Clean (2 pre-existing dead-code warnings) |

## What v0.54 Delivers (3 capabilities + 3 regression fixes)

1. **Auto-context hook handler** — `akar hook user-prompt-submit` reads Claude Code
   UserPromptSubmit JSON from stdin, evaluates tree cleanliness, generates
   NEXT_RUN.md + DIFF_BASELINE.json on clean trees, injects STOP on dirty trees.
   Returns compact context in `hookSpecificOutput.additionalContext` envelope.
2. **Hook setup** — `akar init --hooks --yes` creates/merges project-local
   `.claude/settings.local.json` with UserPromptSubmit hook. Idempotent, preserves
   user hooks, backs up before overwrite.
3. **Doctor/status visibility** — doctor has "claude code hooks:" section, status
   shows `hook:` line.

Three v0.54 regressions found and fixed during dogfood:
- Double-comma in JSON merge output → bracket-counting logic
- Missing `.akar/` directory creation → `create_dir_all` before writes
- Governor ran after baseline dirtying → run `decide()` before `write_baseline()`

## Dogfood Verdict: 4/4 Automated PASS, 1 Pending

| Fixture | Type | Verdict |
|---|---|---|
| Fixture 1: No existing Claude config | Automated CLI | PASS |
| Fixture 2: Existing config + user hook preservation | Automated CLI | PASS |
| Fixture 3: Clean tree hook simulation | Automated CLI | PASS |
| Fixture 4: Dirty tree hook simulation | Automated CLI | PASS |
| Trial: Fresh Claude Code auto-context | Fresh Claude Code session | PENDING |

Hook setup creates valid JSON, preserves user hooks, idempotent. Clean tree
generates NEXT_RUN.md with correct governor decision (SNAPSHOT_NOW). Dirty tree
injects STOP, creates no files. Fresh Claude Code trial is instrumented and
ready but cannot be executed from within a Claude Code session (same meta-testing
limitation as v0.49/v0.50).

## Zero-Relay Delivery Chain (v0.48 → v0.54)

1. v0.48 designed the AI-facing delivery mechanism
2. v0.49 simulated it manually
3. v0.50 attempted fresh-session test but couldn't (manual relay in release spec)
4. v0.51 proved the v0.48 snippet works but found stale-context vulnerability
5. v0.52 fixed stale-context with revised compare-and-reject snippet
6. v0.53 made the snippet managed via `akar init --claude`
7. **v0.54 removes the manual `akar prepare` step via auto-context hook**

The desired flow is now built:
```
akar init --claude --hooks --yes   (one-time setup)
→ user opens Claude Code and types a normal task
→ UserPromptSubmit hook fires
→ AKAR auto-prepares context
→ compact context injected into Claude's system prompt
→ CLAUDE.md snippet triggers NEXT_RUN.md read
→ Claude works without the user mentioning AKAR
```

The only remaining manual step is `akar finish` at session end.

## Known Caveats

1. `doctor::ok_when_everything_present_and_valid` fails — HOOK_EVENTS.jsonl line 972 malformation (pre-existing)
2. `doctor_check` eval fails — same root cause (pre-existing)
3. 2 dead-code warnings: `ProjectDetection` struct and `detect_project` never constructed/used (pre-existing)
4. Fresh Claude Code auto-context trial PENDING — fixture is instrumented, requires external session
5. settings.local.json merge produces working but not pretty-printed JSON (functional, backed up)
6. 26 unrelated src files have fmt whitespace drift (not staged)

## Code Map

### Modules (31 `mod` declarations in src/main.rs, alphabetical)
- `claude_snippet` — CLAUDE.md snippet detection + idempotent apply (v0.53, 349 lines, 12 tests)
- `hook_handler` — UserPromptSubmit hook handler (v0.54, 499 lines, 16 tests)
- `path_health` — PATH version detection + safe repair (v0.53, 445 lines, 8 tests)
- `hooks` — PreToolUse hook template management, `EMBEDDED_HOOK_SH`, `EMBEDDED_HOOK_PS1` via `include_str!`
- `init` — `run_init(skip, claude, hooks, yes)`, `InitResult` with claude_snippet + hook_setup + path_health fields
- `doctor` — `DoctorReport` with claude_snippet + path_health + claude_hooks sections
- `main` — CLI entry point, manual flag parsing (`has_yes_flag`, `has_hooks_flag`, `parse_flag_u64`, `parse_flag_str`)

### Key patterns to follow
- Manual CLI parsing (no clap): check for flags by iterating `args`
- Embedded templates via `include_str!` compiled into binary
- `crate::backup::backup_file()` for backup-before-overwrite
- `confirm_action(prompt: &str) -> bool` in init.rs reads stdin for "INSTALL"
- `#[allow(dead_code)]` on public API struct fields consumed via format consumers
- std-only zero-dependency approach (manual JSON, no serde)
- `#[cfg(test)] mod tests` within source files, temp dirs in `std::env::temp_dir()`

### Hook handler architecture (src/hook_handler.rs)
- `run_user_prompt_submit_hook()` — public entry point
- `parse_hook_input()` — stdin JSON → HookInput { prompt, cwd }
- `evaluate()` — checks tree cleanliness → HookOutcome variant
- `generate_next_run()` — runs governor (before writes!), creates .akar/, writes baseline + NEXT_RUN
- `context_for_ready()` / `context_for_dirty_tree()` / `context_for_no_repo()` — builds compact context
- `print_response()` — wraps context in `{"hookSpecificOutput":{"hookSpecificOutput":{"additionalContext":"..."}}}` envelope
- `config_for_cwd()` — builds Config from hook cwd (needed because Config::discover uses current_dir)

### Hook setup architecture (src/init.rs: setup_claude_hooks)
- `setup_claude_hooks()` — string-based JSON merge with bracket counting for existing UserPromptSubmit arrays
- Creates `.claude/` directory if needed, backs up before overwrite, checks for AKAR marker for idempotency
- Hook command: `pwsh -NoProfile -Command "akar hook user-prompt-submit"` (Windows) or plain `akar hook user-prompt-submit` (Unix)

### v0.54 files changed since `34bb34e` (dogfood regression fixes, uncommitted)
- `src/hook_handler.rs` — `create_dir_all` + governor-before-baseline fix
- `src/init.rs` — bracket-counting merge fix for double-comma

## Next Recommended Release

**v0.55.0: External Fresh Claude Code Trial Results** — run the fresh Claude Code
trial against the instrumented fixture, record results. If the trial passes, this
is a pure audit release (no code changes beyond this dogfood's regression fixes).
If the trial fails, v0.55 may need hook format adjustments.
