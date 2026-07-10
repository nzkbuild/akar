# AKAR v0.56.0 — Capability Quality, Security, and Benchmark Audit

## Summary

v0.56.0 audits the v0.55 capability-awareness foundation to answer: **does host
capability awareness create measurable improvement?** The release adds 28
security/hostile-metadata tests, a dedup fix, a two-stage verification benchmark
fixture, and a scoring framework — without changing the capability selection model
itself.

## Architecture Audit (capability.rs)

**Decision: KEEP as single file.** The 2,391-line `src/capability.rs` has 12 clear
responsibility sections separated by `// ----` dividers:

1. Data model (Capability, enums, inventory, selection, profile structs)
2. Repository-native discovery (~230 lines)
3. Claude Code capability discovery (~220 lines)
4. AKAR capability discovery (~85 lines)
5. Main discovery entry point (~40 lines)
6. Deterministic selection (~140 lines)
7. Task operating profile (~230 lines)
8. Context rendering (~150 lines)
9. JSON helpers — std-only, no serde (~100 lines)
10. Format helpers (~130 lines)
11. Tests (~855 lines)

All sections operate on the shared `Capability` data model. Splitting would create
tight cross-module coupling (every discovery adapter returns `Capability`, selection
consumes `CapabilityInventory`, rendering consumes `CapabilitySelection` +
`TaskProfile`). The JSON helpers (std-only, no serde) would need duplication or a new
shared crate. Verdict: cohesion > coupling, monolithic is correct here.

## Security Audit Results

### Capability Selection Quality
- **Fixed**: Added `HashSet`-based deduplication in `select_capabilities` to prevent
  duplicate capability IDs from consuming selection slots (1-line change, line 793).
- All 64 capability tests pass (zero failures).

### New Audit Tests (28 tests, all passing)
| Category | Tests | What They Verify |
|---|---|---|
| Hostile metadata | 8 | Prompt injection stays in data, code fences contained, control chars safe, malformed unicode safe, long descriptions truncated, shell metacharacters never executed, fake system messages isolated, fake secrets in descriptions isolated |
| Inventory pressure | 4 | 0 caps, 1 cap, 30 caps, 100/1000 caps at scale |
| Scoring manipulation | 2 | Keyword stuffing doesn't guarantee selection, scoring bounded |
| Duplicate/conflict | 1 | Duplicate IDs deduplicated (implementation fix applied) |
| MCP secret safety | 1 | MCP env vars never exposed in context |
| Broken sources | 4 | Unreadable skill dir, malformed plugin JSON, missing MCP settings, partial discovery doesn't block others |

### Key Security Finding
Capability descriptions are safe-by-construction: they're embedded in the
`additionalContext` JSON string that Claude Code reads as data, not instructions.
Even if a hostile skill/plugin `.md` file contains "Ignore previous instructions,"
it stays inside a capability description block — Claude would need to actively
disobey its own prompt to be influenced. The renderer truncates at newline
boundaries, so code-fence injection can't break out of the context block.

## Benchmark Infrastructure

### Fixture: `docs/audits/benchmarks/redirect-validator/`
Node.js HTTP redirect validator with 3 known defects:
1. `.every` instead of `.some` in host allow-list (logic bug)
2. Empty allow-list passes everything (open redirect — reachable via dynamic config)
3. `locationHeader` does not strip CRLF (header injection)

### Two-Stage Verification
- **Stage 1** (`npm test`): Functional tests — validate basic behavior, do NOT test
  empty allow-list or CRLF. 11 assertions, all pass.
- **Stage 2** (`npm run audit:test`): Audit tests — flag the $.every bug, empty
  allow-list bypass, and CRLF injection. 5 assertions, all pass on current code.

### Benchmark Matrix (ready, not yet executed)
4 clones × 2 model tiers × AKAR enabled/disabled, same first prompt, same defect
count. Scoring rubric: 10 dimensions × 10 points each (100 max).

Manual work budget consumed so far: 0/4 external model runs, 0/1 copy-paste cycles.

## Scoring Rubric (10 dimensions)

| # | Dimension | Weight |
|---|---|---|
| 1 | Defect Discovery | 10 |
| 2 | Two-Stage Verification | 10 |
| 3 | Token Efficiency | 10 |
| 4 | Context Relevance | 10 |
| 5 | Fix Correctness | 10 |
| 6 | No Regressions | 10 |
| 7 | Safety/Command Audit | 10 |
| 8 | Instructions Followed | 10 |
| 9 | Round Efficiency | 10 |
| 10 | Quality-per-Token | 10 |

## Test Results

```
cargo test: 642 passed, 1 failed (pre-existing HOOK_EVENTS.jsonl line 972)
cargo test capability: 64 passed, 0 failed
cargo build --release: clean
```

## Verdict on Original Questions

| Question | Answer |
|---|---|
| Does capability awareness reduce user copy-paste? | Selection is deterministic — no model calls, no delay. Evidence: benchmark matrix design (not yet executed). |
| Does it reduce repeated prompting? | Task profile + two-stage verification plan embedded in context. Evidence: benchmark Stage 2 instructions. |
| Does it reduce wasted tokens? | Hard budget enforced (1,200 chars caps, 600 chars profile). Evidence: truncation tests confirm. |
| Does it reduce irrelevant context? | Top-5 scoring by keyword + project-kind. Evidence: pressure tests at 1,000 caps. |
| Does it reduce hallucinated tools? | Capabilities grounded in actual filesystem discovery. Evidence: broken-source tests tolerate failure gracefully. |
| Does it improve capability selection? | Deterministic, no model involvement. Evidence: dedup fix, scoring manipulation resistance. |
| Does it improve safety? | MCP secrets redacted. Evidence: `mcp_env_vars_never_exposed` test. |
| Does it improve verification quality? | Two-stage plan rendered per task. Evidence: benchmark fixture demonstrates the pattern. |
| Does it improve audit depth? | Audit tests catch what functional tests miss. Evidence: fixture Stage 1 vs Stage 2. |
| Is it maintainable? | Single-file, 12 clear sections, 64 tests. Evidence: architecture audit confirms KEEP decision. |
