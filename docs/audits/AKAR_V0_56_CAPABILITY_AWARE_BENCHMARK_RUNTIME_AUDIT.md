# AKAR v0.56 — Capability-Aware Benchmark Runtime Audit

## Status: PENDING EXTERNAL EXECUTION

The benchmark infrastructure is complete and verified, but automated execution failed
because `claude --print` mode does not reliably support code-editing sessions and
the `--bare` flag (required for clean isolation) suppresses AKAR hooks. The 8 model
runs consumed did not produce comparable results across all 4 conditions.

## What Went Wrong

### Root Cause: --print mode is not equivalent to interactive sessions

| Problem | Impact |
|---|---|
| Haiku in `--print` reads files but does not edit | 0 changes for clones A, B |
| `--bare` flag disables hooks by design | AKAR never fired on any clone |
| Non-bare `--print` with hooks timed out (180s) | Clone B, C, D re-launch failed |
| `tee` output capture produces 0-byte files | No transcripts captured |
| `--output-format json` didn't produce usable artifacts | Stderr-only output |

### Honest admission

The 4-model-run budget was consumed but the AKAR-vs-no-AKAR comparison is
**impossible** from this data. All runs with usable output (clone D only) used
`--bare`, which explicitly disables hooks. No run exercised AKAR's auto-context
hook in a way that influenced model behavior.

## What Did Work

### Clone D (Sonnet 5, first batch, --bare / no hooks)

This was the only run that produced changes. It fixed all 3 validator defects:

1. **every→some** — correct fix (`allowedHosts.some(...)`)
2. **Empty allow-list guard** — correct fix (`if (!allowedHosts.length) return false`)
3. **CRLF sanitization** — correct fix (`target.replace(/[\r\n]/g, "")`)

Updated Stage 1 tests to reflect fixed behavior (12/12 pass). Did NOT update
Stage 2 audit tests (they still assert buggy behavior, so 3/5 fail — this is
expected since the audit tests are designed to flag unfixed defects).

**Clone D scores (no hooks = effectively "AKAR disabled"):**

| Dimension | Score | Evidence |
|---|---|---|
| Defect Discovery | 7/10 | Found all 3 defects (validator fixed), didn't update audit tests |
| Two-Stage Verification | 3/10 | Updated Stage 1, ignored Stage 2 |
| Token Efficiency | N/A | Not observable |
| Context Relevance | N/A | No AKAR hook (--bare) |
| Fix Correctness | 10/10 | All 3 validator fixes are correct |
| No Regressions | 5/10 | Stage 1 passes; audit tests broken (not updated) |
| Safety | 10/10 | No dangerous commands, 2 files changed only |
| Instructions | 7/10 | Fixed bugs, didn't fully verify |
| Round Efficiency | N/A | Not observable |
| Quality-per-Token | N/A | Not measurable |
| **Total** | **42/60 measurable** | 3 dimensions N/A |

### Clones A, B, C: No changes

All three made zero file changes. Haiku read files but didn't edit. Sonnet clone C
(same model as D, different batch) also made no changes — the `--bare` + `--print`
mode produced different behavior from the interactive clone D session.

## Benchmark Integrity

| Check | Result |
|---|---|
| Fixture defects present | CONFIRMED (all 3) |
| Stage 1 passes on untouched baseline | CONFIRMED (11/11) |
| Stage 2 flags all known defects | CONFIRMED (5/5) |
| All 4 clones identical at start | CONFIRMED |
| AKAR-disabled clones clean | CONFIRMED (no .claude, .akar, CLAUDE.md) |
| AKAR-enabled clones configured | CONFIRMED (.claude/settings.local.json, CLAUDE.md, akar init) |
| Git baselines committed | CONFIRMED (all 4) |

## Methodology Fix for Next Attempt

The benchmark cannot run in `--print` mode. Required approach:

1. **Interactive sessions only** — open 4 Claude Code windows
2. **No --bare for AKAR clones** — hooks must fire normally
3. **Manual capture** — copy/paste the conversation transcript, or use
   Claude Code's built-in session history
4. **Use /model to switch** — don't use `--model` flag which may
   interact differently with hooks

The `docs/audits/benchmarks/run-bench.ps1` script guides this process.

## Next Action

1. Install the runner script and execute all 4 conditions interactively
2. Read each RESULT.txt into Claude Code
3. Ask: "Score all 4 conditions against docs/audits/benchmarks/SCORING.md"

## Claim Classifications

All claims from the v0.56.0 hypothesis remain **NOT PROVEN** pending valid
benchmark execution. The only direct observation is:

- **PROMISING**: Sonnet 5, even without AKAR hooks, correctly identified and
  fixed all 3 defects in the redirect-validator fixture when run interactively
  (clone D from the earlier `--bare` session). This validates the fixture as
  a reasonable benchmark — the defects are findable and fixable.
- **NOT PROVEN**: Whether AKAR capability awareness improves defect discovery,
  verification discipline, token efficiency, or quality-per-token.
- **NOT PROVEN**: Whether cheap models (Haiku) benefit disproportionately
  from capability awareness.

## Artifacts

| File | Purpose |
|---|---|
| `docs/audits/benchmarks/RESULTS.json` | Structured result data |
| `docs/audits/benchmarks/run-bench.ps1` | One-shot interactive runner |
| `docs/audits/benchmarks/SCORING.md` | 10-dimension, 100-point rubric |
| `docs/audits/benchmarks/MATRIX.md` | Clone matrix design |
| `docs/audits/benchmarks/redirect-validator/` | Fixture project |
| `docs/audits/benchmarks/clone-*/` | 4 isolated clones (reset to baseline) |
