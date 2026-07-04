# AKAR Adoption Notes

**Date:** 2026-07-03  
**Phase:** 0 — Freeze and Audit

## Repo state at audit time

- Single file: `AKAR_MASTER_ROADMAP_v1.0_REVISED.md`
- No existing Rust project, no Cargo.toml, no source code
- No existing `.claude` config in this directory
- No hooks, no slash commands, no memory files
- Not a git repository

## Toolchain confirmed

- Rust: rustc 1.94.0 (4a4ef493e 2026-03-02)
- Cargo: 1.94.0 (85eff7c80 2026-01-15)
- OS: Windows 11 Pro 10.0.26200
- Shell: PowerShell (primary)

## External Claude Code config

- Located at `~/.claude/` — **not touched**
- Global CLAUDE.md, hooks, and skills left intact per Phase 0 constraint

## Known issues at audit time

- None — clean slate

## First safe implementation target

**Phase 1: Rust CLI Scaffold**

Safe because:
- Creates new files only, no existing code to break
- No hook integration, no global config edits
- `cargo test` and `cargo run -- --version` are the verification gate
- All commands return placeholder output — no destructive operations possible

## Next safe implementation target after Phase 1

**Phase 2: Kernel Docs**

Create concise policy docs under `docs/kernel/`:
`POLICY.md`, `AUTONOMY.md`, `RISK_LEVELS.md`, `DIFF_BUDGET.md`,
`DONE_DEFINITION.md`, `SOURCE_PRIORITY.md`, `COMMAND_SAFETY.md`,
`MEMORY_SCHEMA.md`, `CONTEXT_BUDGET.md`, `TEST_INTELLIGENCE.md`,
`VERIFICATION_LADDER.md`, `DESIGN_QUALITY.md`

These are pure documentation — zero risk, high value for future phases.

---

## Phase 17 Hardening — Complete

**Date:** 2026-07-04

**Final test count:** 121 passed, 0 failed

**All commands verified:**
- `akar --version` → `akar 0.1.0`
- `akar status` → DEGRADED (expected — .akar/ and ~/.claude/akar/ not yet bootstrapped)
- `akar doctor` → 2 issues found (expected — same missing dirs)
- `akar eval` → 20/20 passed
- `akar mission "fix the login button"` → Bugfix, A5, 3 files / 60 LOC budget
- `akar verify`, `akar bootstrap`, `akar safety`, `akar skills`, `akar calibrate`, `akar hooks` — all operational

**Modules implemented:**
config, doctor, event_log, backup, safe_fix, contract, context_pack, verify, mission, design, safety, skill_registry, model_profile, eval, circuit_breaker

**Known limitations:**
- Execute phase is scaffold only — `akar mission` classifies and plans but does not run code changes
- `akar bootstrap` prints placeholder — memory template creation not yet wired to filesystem
- No release binary packaging (cargo build --release works but no installer/PATH setup)
- No Claude Code hook auto-install — hooks/ scripts must be manually copied to .git/hooks/
- `.akar/` and `~/.claude/akar/` dirs do not exist until `akar doctor --fix` is run
- No git repository initialized — `akar calibrate` reports branch as "unknown"
