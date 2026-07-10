# AKAR v0.56 — Capability-Aware Benchmark Final Report

## Status: COMPLETE (with invalid AKAR comparison)

The benchmark was executed interactively across all 4 conditions, but the AKAR
UserPromptSubmit hook did not fire in either AKAR-enabled clone. The A-vs-B and
C-vs-D comparisons are therefore **not valid AKAR-vs-no-AKAR comparisons**.

Full scored report: `docs/audits/benchmarks/BENCHMARK_REPORT.md`

## What Went Wrong (Revisited)

### Phase 1: Automated execution (8 model runs — FAILED)

| Problem | Impact |
|---|---|
| `--bare` flag disables hooks by design | AKAR never fired on any clone |
| Haiku in `--print` reads files but does not edit | 0 changes for clones A, B |
| Non-bare `--print` with hooks timed out (180s) | Clone B, C, D re-launch failed |
| `tee` output capture produces 0-byte files | No transcripts captured |

### Phase 2: Interactive execution (4 model runs — PARTIAL SUCCESS)

The user ran all 4 conditions interactively. The models performed (Sonnet fixed all 3 defects, Haiku fixed 0-1), but:

**AKAR UserPromptSubmit hook did NOT fire in either enabled clone (B, D).**

- No `NEXT_RUN.md` generated in clone B or clone D
- No UserPromptSubmit events in `HOOK_EVENTS.jsonl` (only PreToolUse events)
- Both AKAR-enabled clones behaved identically to their AKAR-disabled counterparts

### Root cause hypothesis

The hook configuration was correct (`.claude/settings.local.json` + `CLAUDE.md` present), but:
1. **Path resolution**: The user opened each clone as a separate directory. Claude Code may resolve settings relative to the launch directory differently.
2. **PowerShell execution policy**: Non-interactive pwsh may block script execution.
3. **PATH**: `akar` binary may not be on PATH in the hook execution context.
4. **Empty matcher**: The `""` matcher in settings.local.json may not match in certain contexts.

## What Did Work

### Benchmark fixture validated

| Check | Result |
|---|---|
| Fixture defects present in baseline | CONFIRMED (all 3) |
| Stage 1 passes on untouched baseline | CONFIRMED (11/11) |
| Stage 2 flags all known defects | CONFIRMED (5/5 baselines) |
| All 4 clones identical at start | CONFIRMED |
| AKAR-disabled clones clean | CONFIRMED (no .claude, .akar, CLAUDE.md) |
| AKAR-enabled clones configured | CONFIRMED (settings.local.json, CLAUDE.md, akar init) |
| Two-stage verification design | VALIDATED (all models ran both stages) |

### Model capability findings

- **Sonnet 5 is gold standard**: Fixed all 3 defects correctly in both runs (C, D), with zero regressions and full test coverage. No guidance needed.
- **Haiku 4.5 is not reliable**: Fixed 1/3 in best run (A), 0/3 in other run (B). Within expected variance but insufficient for this class of bug-fixing task.

## Final Scores

| Clone | Model | AKAR | Defects Fixed | Score (/60) |
|-------|-------|------|:---:|:---:|
| A | Haiku 4.5 | disabled | 1/3 | 44 |
| B | Haiku 4.5 | enabled* | 0/3 | 32 |
| C | Sonnet 5 | disabled | 3/3 | 60 |
| D | Sonnet 5 | enabled* | 3/3 | 60 |

\* Hook configured but did not fire — effectively disabled.

**Decision rule:** FAIL. D == C (need >=10 gap). B < A (need >=15 gap, B was worse).

## Claim Classifications

| Claim | Status |
|-------|--------|
| "AKAR improves AI coding work" | **NOT PROVEN** — hook didn't fire |
| "Cheap models benefit disproportionately from AKAR" | **NOT MEASURED** — hook didn't fire |
| "Sonnet 5 fixes all 3 defects without AKAR" | **PROVEN** — both C and D (D had no AKAR context) |
| "Haiku 4.5 is insufficient for this task" | **PROMISING** — 2 runs, 1/3 and 0/3 |
| "Two-stage verification design is sound" | **PROVEN** — all clones ran both stages |
| "Benchmark fixture detects known defects" | **PROVEN** — Stage 2 flags all 3 defects |
| "Claude Code --bare disables AKAR hooks" | **PROVEN** — confirmed in Phase 1 |
| "Claude Code --print mode is not equivalent to interactive" | **PROVEN** — Phase 1 vs Phase 2 |
| "AKAR hook fires in interactive sessions" | **REFUTED** — did not fire in 2 independent runs |

## Model Run Budget

| Phase | Runs | Productive | Notes |
|-------|:---:|:---:|-------|
| Phase 1 (automated) | 8 | 1 | --bare/--print failures |
| Phase 2 (interactive) | 4 | 4 | All produced results |
| **Total** | **12** | **5** | Budget was 4, exceeded due to automation failures |

## Next Steps

1. **Debug hook delivery**: Verify `akar hook user-prompt-submit` works in clone directory from non-interactive pwsh
2. **Fix hook mechanism**: Ensure UserPromptSubmit hook fires reliably in interactive sessions
3. **Re-run clones B and D only**: Once hook is fixed, only need 2 model runs
4. **Re-score**: Same rubric, same fixture, fresh clones

## Artifacts

| File | Purpose |
|---|---|
| `docs/audits/benchmarks/BENCHMARK_REPORT.md` | Final scored report (this audit's companion) |
| `docs/audits/benchmarks/RESULTS.json` | Structured result data (updated with scores) |
| `docs/audits/benchmarks/run-bench.ps1` | One-shot interactive runner |
| `docs/audits/benchmarks/SCORING.md` | 10-dimension, 100-point rubric |
| `docs/audits/benchmarks/MATRIX.md` | Clone matrix design |
| `docs/audits/benchmarks/redirect-validator/` | Fixture project |
| `docs/audits/benchmarks/clone-*/` | 4 isolated clones |
| `docs/audits/benchmarks/clone-*-RESULT.txt` | Raw interactive session output (4 files) |
