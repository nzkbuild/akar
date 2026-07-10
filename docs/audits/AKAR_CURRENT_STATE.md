# AKAR Current State — 2026-07-11

Consolidated snapshot for future prompts. See the source audit docs for full detail:
- `AKAR_V0_52_CLAUDE_MD_STALE_CONTEXT_REVISION.md`
- `AKAR_V0_53_ZERO_RELAY_SETUP_FOUNDATION.md`
- `AKAR_V0_53_EXTERNAL_DOGFOOD.md`
- `AKAR_V0_54_ZERO_RELAY_AUTO_CONTEXT_HOOK.md`
- `AKAR_V0_54_EXTERNAL_DOGFOOD.md`
- `AKAR_V0_55_EXTERNAL_DOGFOOD.md`
- `AKAR_CAPABILITY_AUDIT_v0.56.0.md` (this release)

## Baseline

| Check | Value |
|---|---|
| Commit | `a7b12a8` — docs: record AKAR v0.56 external benchmark results |
| Version | `akar 0.56.0` |
| `cargo test` | 642 passed, 1 failed (pre-existing: HOOK_EVENTS.jsonl line 972) |
| `cargo build --release` | Clean |

## What v0.56 Delivers (capability benchmark + audit)

1. **Architecture audit** — confirmed capability.rs KEEP decision (12 sections, single file, strong intra-module cohesion).
2. **28 security/hostile-metadata audit tests** — prompt injection containment, code fence safety, control characters, malformed unicode, shell metacharacter safety, fake system message isolation, inventory pressure (0/1/30/100/1000 caps), scoring manipulation resistance, MCP secret safety, broken source tolerance. All 28 pass.
3. **Dedup fix** — `select_capabilities` now deduplicates by capability ID (HashSet-based, 1-line change at line 793).
4. **Benchmark fixture** — `docs/audits/benchmarks/redirect-validator/` with 3 known defects (every→some logic bug, empty allow-list bypass, CRLF injection), two-stage verification (`npm test` + `npm run audit:test`), clean git baseline.
5. **Benchmark matrix** — 4-clone design (Haiku/Sonnet × AKAR enabled/disabled), 10-dimension scoring rubric, ready for external execution.
6. **Total: 64 capability tests (43 original + 28 new + dedup fix), all passing.**

## What v0.55 Delivered (host capability awareness)

1. **`akar capabilities [--json]`** — discovers 30 capabilities across 4 categories:
   repo commands (4), user skills (17), plugins (3), AKAR built-ins (6). Read-only.
2. **Deterministic selection** — keyword + project-kind scoring picks up to 5 most
   relevant capabilities per task. No model calls.
3. **Task operating profile** — Leverage/Limits/Risks/Strategy, atomic phase plan,
   two-stage verification (Stage 1 functional, Stage 2 audit). Tailored by task type.
4. **Enhanced auto-context** — hook handler injects capabilities + profile + verification
   alongside existing task/budget/NEXT_RUN info. Hard context budget (1,200 + 600 char caps).
5. **Secret redaction** — MCP server discovery reveals only names and scopes. Never
   exposes command arguments, tokens, or credentials.
6. **Host-agnostic architecture** — `Capability` data model with host-specific adapters
   (Repository, ClaudeCode, Akar). Shared internal task-contract logic reused between
   manual prepare flow and hook flow.

## Dogfood Verdict: 5/5 PASS

| Fixture | Type | Verdict |
|---|---|---|
| Fixture 1: Capability discovery (list) | Automated CLI | PASS |
| Fixture 2: Capability discovery (JSON) | Automated CLI | PASS |
| Fixture 3: Hook with capabilities | Automated CLI | PASS |
| Fixture 4: Dirty tree hook | Automated CLI | PASS |
| Fixture 5: Status shows caps | Automated CLI | PASS |

v0.55 automated dogfood is complete. No external fresh Claude Code trial was
requested — the capability discovery and enhanced context injection are validated
via external fixtures.

## Zero-Relay Delivery Chain (v0.48 → v0.55)

1. v0.48 designed the AI-facing delivery mechanism
2. v0.49 simulated it manually
3. v0.50 attempted fresh-session test but couldn't (manual relay in release spec)
4. v0.51 proved the v0.48 snippet works but found stale-context vulnerability
5. v0.52 fixed stale-context with revised compare-and-reject snippet
6. v0.53 made the snippet managed via `akar init --claude`
7. v0.54 removes the manual `akar prepare` step via auto-context hook
8. **v0.55 adds host capability awareness — the hook injects grounded, task-relevant capabilities**

The desired flow now includes capability intelligence:
```
akar init --claude --hooks --yes   (one-time setup)
→ user opens Claude Code and types a normal task
→ UserPromptSubmit hook fires
→ AKAR discovers 30 capabilities, selects top 5 for this task
→ compact context injected: task, budget, capabilities, profile, verification
→ CLAUDE.md snippet triggers NEXT_RUN.md read
→ Claude works with grounded capability awareness
```

The only remaining manual step is `akar finish` at session end.

## Known Caveats

1. `doctor::ok_when_everything_present_and_valid` fails — HOOK_EVENTS.jsonl line 972 malformation (pre-existing)
2. 2 dead-code warnings: `ProjectDetection` struct and `detect_project` never used (pre-existing)
3. 8 dead-code warnings in capability.rs: format helpers, unused enums/fields (used by CLI/doctor)
4. settings.local.json merge produces working but not pretty-printed JSON (functional, backed up)
5. Live multi-host support not implemented
6. MCP/skill/plugin routing not implemented
7. Token/request reduction — benchmark matrix and runner script ready; automated execution failed (--print mode unreliable); requires interactive sessions (see `AKAR_V0_56_CAPABILITY_AWARE_BENCHMARK_RUNTIME_AUDIT.md`)

## Code Map

### Modules (32 `mod` declarations in src/main.rs, alphabetical)
- `capability` — host capability awareness (v0.55→v0.56, ~2,400 lines, 64 tests)
- `claude_snippet` — CLAUDE.md snippet detection + idempotent apply (v0.53, 349 lines, 12 tests)
- `hook_handler` — UserPromptSubmit hook handler with capability injection (v0.54→v0.55, 511 lines, 16 tests)
- `path_health` — PATH version detection + safe repair (v0.53, 445 lines, 8 tests)
- `hooks` — PreToolUse hook template management
- `init` — `run_init(skip, claude, hooks, yes)`
- `doctor` — `DoctorReport` with claude_snippet + path_health + claude_hooks sections
- `main` — CLI entry point, 32 mod declarations, `capabilities` subcommand

### Key patterns (unchanged)
- Manual CLI parsing (no clap)
- Embedded templates via `include_str!`
- std-only zero-dependency approach (manual JSON, no serde)
- `#[cfg(test)] mod tests` within source files
- Host-agnostic data model + host-specific discovery adapters (v0.55)

### Capability module architecture (src/capability.rs)
- `Capability` struct: id, name, category, host, scope, confidence, risk, invocation_hint
- `CapabilityInventory`: all discovered capabilities with metadata
- `discover_all()`: orchestrates Repository → ClaudeCode → Akar discovery
- `select_capabilities()`: deterministic keyword + confidence + scope scoring
- `build_task_profile()`: Leverage/Limits/Risks/Strategy + phase plan + verification
- `build_enhanced_context()`: full auto-context with caps + profile + footer
- Hard budget: `CAPABILITY_CONTEXT_HARD_CAP = 1200`, `PROFILE_CONTEXT_BUDGET = 600`

## Next Recommended Release

**v0.57.0: Benchmark Execution or Post-Session Automation** — two candidate directions:
(A) Execute the 4-clone benchmark matrix with actual Claude Code model runs to get
empirical scores; (B) Automate `akar finish` via PreToolUse hooks to close the last
manual step in the prepare↔finish cycle. The benchmark infrastructure is ready and
the scoring rubric is defined — the next release can be a dogfood-only release that
reports actual measurements.
