# AKAR v0.55 External Dogfood

## 1. Executive Verdict

**5/5 automated fixtures PASS. 43/43 capability tests PASS. 621/622 total tests pass (1 pre-existing failure).**

Host capability awareness works: `akar capabilities` discovers 30 capabilities across repo
commands (4), user skills (17), plugins (3), and AKAR built-ins (6). The hook handler
injects compact capability guidance, task operating profiles, and two-stage verification
plans into auto-context — all under budget (<1,200 chars for capabilities, <600 for profile).
Dirty-tree safety boundary unchanged from v0.54. No secrets leaked.

## 2. Baseline

| Check | Value |
|---|---|
| Commit | `34bb34e` — feat: add AKAR Claude Code auto-context hook prototype |
| Version | `akar 0.54.0` (source) — dogfood binary built as release build |
| `cargo test` | 621 passed, 1 failed (pre-existing: HOOK_EVENTS.jsonl line 972) |
| `cargo build --release` | Clean (10 dead-code warnings: unused format helpers; pre-existing dead-analysis warnings) |
| Dogfood date | 2026-07-10 |

## 3. What v0.55 Adds

1. **Host capability discovery** — `akar capabilities [--json]` discovers:
   - Repository-native commands (Cargo, npm, Makefile, justfile, Python)
   - Claude Code skills (project-local and user-global from `~/.claude/skills/`)
   - Claude Code plugins (`installed_plugins.json`)
   - Claude Code MCP servers (project-local and global `settings.json`)
   - AKAR built-in capabilities (prepare, finish, governor, doctor, verify, safety)
2. **Deterministic capability selection** — keyword + project-kind scoring selects up
   to 5 most relevant capabilities per task. No model calls involved.
3. **Task operating profile** — Leverage, Limits, Risks, Strategy, atomic phase plan,
   and two-stage verification (Stage 1 functional, Stage 2 audit) per task.
4. **Enhanced auto-context** — hook handler injects capability guidance, task profile,
   and verification plan alongside existing task/budget/NEXT_RUN info.
5. **Context budget enforcement** — hard caps: 1,200 chars for capabilities, 600 for
   task profile. Oversized output is truncated safely.
6. **Secret redaction** — MCP server discovery reveals only names and scopes. Command
   arguments, tokens, and credentials are never exposed.
7. **Status visibility** — `akar status` shows `caps: N discovered`.
8. **Architecture** — host-agnostic `Capability` data model with host-specific
   adapters (Repository, ClaudeCode, Akar). Deterministic selection via scoring,
   not model calls.

## 4. Automated Fixture Results

| # | Fixture | Verdict |
|---|---|---|
| 1 | Capability discovery (list) | PASS — 24 discovered in fixture: 4 repo + 17 skills + 3 plugins |
| 2 | Capability discovery (JSON) | PASS — valid JSON, all required fields, no secrets exposed |
| 3 | Hook simulation with capabilities | PASS — capability guidance + profile + atomic phases injected |
| 4 | Dirty tree hook (safety boundary) | PASS — STOP injected, no .akar/ created |
| 5 | Status shows caps line | PASS — `caps: 30 discovered` |

### 4.1. Fixture 1: Capability Discovery (List)

Rust project with Cargo.toml, no `.akar/` directory, no `.claude/` config.

**Result:**
- 24 capabilities discovered (4 repo, 17 skills, 3 plugins)
- AKAR capabilities absent (no `.akar/` — correct)
- All descriptions truncated to ≤120 chars
- Credentials redacted

**Verdict: PASS**

### 4.2. Fixture 2: Capability Discovery (JSON)

Same fixture, `--json` flag.

**Result:**
- Valid JSON structure with `host`, `discovered_count`, `discovery_time_ms`
- Each capability has: id, name, category, scope, confidence, description, risk
- `redaction_notice` present
- No raw paths or credentials in descriptions

**Verdict: PASS**

### 4.3. Fixture 3: Hook Simulation with Capabilities

Clean git tree, `echo '{"prompt":"fix the compile error in main.rs","cwd":"..."}' | akar hook user-prompt-submit`

**Result:**
- Valid hook JSON response envelope
- Capability guidance: `cargo build`, `cargo clippy`, `cargo test`, `cargo fmt`, `akar doctor`
- Task profile: Leverage, Limits, Risks, Strategy, atomic phase plan (3 phases)
- Verification: Stage 1 functional + Stage 2 audit
- Budget: 3 files, 60 LOC
- Context budget respected (capabilities under 1,200 chars, profile under 600)

**Verdict: PASS**

### 4.4. Fixture 4: Dirty Tree Hook (Safety Boundary)

Dirty working tree (uncommitted README.md change).

**Result:**
- STOP instruction injected
- No NEXT_RUN.md generated
- No .akar/ directory created
- Safety boundary unchanged from v0.54

**Verdict: PASS**

### 4.5. Fixture 5: Status Shows Caps Line

`akar status` in the AKAR project root.

**Result:**
- `caps: 30 discovered` line present
- All existing status fields preserved (doctor, bootstrap, telemetry, postmortem, etc.)
- No regression in status output format

**Verdict: PASS**

## 5. Safety Boundaries

| Boundary | Status |
|---|---|
| Read-only discovery | HELD — never executes discovered commands |
| Secret redaction | HELD — MCP args, tokens, credentials redacted |
| Dirty tree → stop | HELD — unchanged from v0.54 |
| No model calls for selection | HELD — deterministic keyword scoring |
| Context budget enforced | HELD — hard caps on capability + profile sections |
| No new dependencies | HELD — std-only, zero external crates |
| Project-local config only | HELD — unchanged from v0.54 |

## 6. What Worked

1. Capability discovery across all four categories (repo, skills, plugins, AKAR)
2. Plugin JSON parsing with depth tracking — correctly extracts only top-level keys
3. MCP server discovery reads `mcpServers` key without leaking command args
4. Skill directory scanning picks up display names and descriptions from SKILL.md frontmatter
5. Deterministic selection prioritizes project-local + high-confidence + keyword matches
6. Task operating profile tailors phases, verification, and audits by task type
7. Security-sensitive tasks get stronger Stage 2 audit (secrets, permissions, input validation)
8. Migration tasks get rollback check
9. Trivial tasks get minimal audit ("Low risk — no broader audit required")
10. Dirty tree safety boundary unchanged from v0.54
11. Status properly shows capability count

## 13. Test Results

```
cargo test: 621 passed, 1 failed (pre-existing: HOOK_EVENTS.jsonl line 972)
cargo test capability: 43 passed, 0 failed
cargo build --release: Clean (10 dead-code warnings)
```

## 14. What Failed

None. All v0.55 additions tested. The 1 pre-existing failure is the known
HOOK_EVENTS.jsonl line 972 malformation from a truncated hook event write.

## 15. Recommended Next Release

**v0.56.0: Post-Session Automation** — with capability awareness now wired into the
hook auto-context, the next step should automate `akar finish` via PreToolUse hooks,
close the last manual step in the prepare↔finish cycle, and benchmark token/request
reduction with capabilities enabled.

## 16. Honest Conclusion

v0.55.0's host capability awareness works end-to-end. Discovery finds 30 capabilities
across 4 categories. Selection deterministically picks the most relevant for each task.
The hook injects compact, actionable guidance under a hard context budget. Safety
boundaries (dirty tree STOP, secret redaction, no model calls) are all held.

**621/622 tests pass (1 pre-existing). 43/43 capability tests pass. Zero new dependencies.**
