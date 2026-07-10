# AKAR Current State — 2026-07-10

Consolidated snapshot for future prompts. See the three source audit docs for full detail:
- `AKAR_V0_52_CLAUDE_MD_STALE_CONTEXT_REVISION.md`
- `AKAR_V0_53_ZERO_RELAY_SETUP_FOUNDATION.md`
- `AKAR_V0_53_EXTERNAL_DOGFOOD.md`

## Baseline

| Check | Value |
|---|---|
| Commit | `29fe6aa` — docs: finalize AKAR v0.53 external dogfood results |
| Version | `akar 0.53.0` |
| Working tree | clean |
| `cargo test` | 562 passed, 1 failed (pre-existing: `doctor::ok_when_everything_present_and_valid`) |
| `cargo eval` | 27/28 PASS (pre-existing: `doctor_check`) |
| `cargo build --release` | Clean (2 pre-existing dead-code warnings) |

## What v0.53 Delivers (3 capabilities, all dogfood-proven)

1. **Managed CLAUDE.md snippet** — `akar init --claude --yes` inserts the exact v0.52
   compare-and-reject snippet into `<project>/CLAUDE.md`. Idempotent, preserves user
   content, backs up before overwrite. Requires confirmation (`--yes` or "INSTALL").
2. **PATH version health** — `check_path_health()` detects running binary vs PATH `akar`
   version mismatch. Surfaces in doctor, status, and init. Offers safe repair.
3. **Doctor/status visibility** — doctor has "claude.md snippet:" and "path health:"
   sections. Status shows `claude.md:` and `path akar:` lines.

## Dogfood Verdict: 6/6 PASS

| Fixture | Type | Verdict |
|---|---|---|
| Fixture 1: No CLAUDE.md | Automated CLI | PASS |
| Fixture 2: Existing CLAUDE.md + user content | Automated CLI | PASS |
| Fixture 3: Old AKAR block replacement | Automated CLI | PASS |
| Fixture 4: PATH health visibility | Automated CLI | PASS |
| Trial A: Matching-task delivery | Fresh Claude Code session | PASS |
| Trial B: Stale-context rejection | Fresh Claude Code session | PASS |

The managed snippet produces identical fresh-session behavior to the v0.52 hand-copied
snippet. Matching-task delivery works. Stale-context rejection boundary holds.
Zero manual relay in both trials.

## Known Caveats (pre-existing, unchanged by v0.53)

1. `doctor::ok_when_everything_present_and_valid` fails — HOOK_EVENTS.jsonl line 972 malformation
2. `doctor_check` eval fails — same root cause
3. 2 dead-code warnings: `ProjectDetection` struct and `detect_project` never constructed/used

## Code Map for v0.54 Implementation

### Modules (30 `mod` declarations in src/main.rs, alphabetical)
- `claude_snippet` — CLAUDE.md snippet detection + idempotent apply (v0.53, 349 lines, 12 tests)
- `path_health` — PATH version detection + safe repair (v0.53, 445 lines, 8 tests)
- `hooks` — PreToolUse hook template management, `EMBEDDED_HOOK_SH`, `EMBEDDED_HOOK_PS1` via `include_str!`
- `init` — `run_init(skip, claude, yes)`, `InitResult` with claude_snippet + path_health fields
- `doctor` — `DoctorReport` with claude_snippet + path_health sections
- `main` — CLI entry point, manual flag parsing (`has_yes_flag`, `parse_flag_u64`, `parse_flag_str`)
- `config` — `Config::discover()`, project root / akar dir / global dir resolution

### Key patterns to follow
- Manual CLI parsing (no clap): `has_yes_flag(args: &[String]) -> bool` — check for `--hooks` similarly
- Embedded templates via `include_str!` compiled into binary
- `crate::backup::backup_file()` for backup-before-overwrite
- `confirm_action(prompt: &str) -> bool` in init.rs reads stdin for "INSTALL"
- `#[allow(dead_code)]` on public API struct fields consumed via format consumers
- std-only zero-dependency approach
- `#[cfg(test)] mod tests` within source files, temp dirs in `std::env::temp_dir()`

### Hook infrastructure (existing, for reference)
- `src/hooks.rs`: `HookTemplateSource` enum, `check_hooks()`, `install_hooks()`, `discover_hook_templates()`
- `templates/hooks/pre-tool-call.sh` and `pre-tool-call.ps1` — PreToolUse hooks (read JSON from stdin)
- Claude Code settings: hooks go in `~/.claude/settings.json` under `hooks.PreToolUse` (existing pattern uses global settings; v0.54 uses project-local `.claude/settings.local.json`)

### v0.54 implementation shape (not yet started)
- New file: `src/hook_handler.rs` — reads UserPromptSubmit JSON from stdin, generates/updates `.akar/NEXT_RUN.md`, returns `hookSpecificOutput.additionalContext`
- New template: `templates/hooks/user-prompt-submit.ps1` — PowerShell UserPromptSubmit hook (or handler built into binary)
- Modified: `src/main.rs` — add `mod hook_handler`, parse `hook user-prompt-submit` subcommand, add `has_hooks_flag`, modify init arm for `--hooks`
- Modified: `src/init.rs` — add `hooks: bool` parameter, add `run_hook_setup()` function
- Modified: `src/doctor.rs` — add hook config check section
- Project-local config: `.claude/settings.local.json` (preferred over global `~/.claude/settings.json`)
- Safety: dirty tree → inject stop/finish instead of preparing new task
