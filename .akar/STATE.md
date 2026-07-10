# State

## Current Goal

v0.56.0 benchmark scored — AKAR hook failure invalidates comparison. Decision: investigate hook delivery mechanism.

## Last Completed

v0.56.0 benchmark completed and scored (commit pending). Interactive benchmark executed by user across 4 conditions. Final report produced at `docs/audits/benchmarks/BENCHMARK_REPORT.md`.

**Key finding:** AKAR UserPromptSubmit hook did not fire in either AKAR-enabled clone (B, D). No NEXT_RUN.md was generated in either clone; HOOK_EVENTS.jsonl contains only PreToolUse events. The A-vs-B and C-vs-D comparisons are NOT valid AKAR-vs-no-AKAR comparisons.

**Scored results (measurable dimensions only, max 60):**

| Clone | Model | AKAR | Defects | Score | Notes |
|-------|-------|------|:---:|:---:|-------|
| A | Haiku 4.5 | disabled | 1/3 | 44/60 | every→only, missed CRLF |
| B | Haiku 4.5 | enabled* | 0/3 | 32/60 | Hook not fired, zero changes |
| C | Sonnet 5 | disabled | 3/3 | 60/60 | Perfect — all defects fixed |
| D | Sonnet 5 | enabled* | 3/3 | 60/60 | Hook not fired, same as C |

\* Hook configured but did not fire — effectively disabled.

**Decision rule:** FAIL. D > C by 0 pts (need >=10). B > A by -11 pts (need >=15, B was worse).

**What was proven:**
- Sonnet 5 fixes all 3 defects without any guidance (gold standard)
- Haiku 4.5 is insufficient for this task
- Benchmark fixture + two-stage verification design is sound
- `--bare` disables AKAR hooks (confirmed)
- `--print` mode is not equivalent to interactive code-editing sessions

**What was NOT proven:**
- Whether AKAR capability awareness improves AI coding work (hook didn't fire)

## Next Steps

1. Debug why UserPromptSubmit hook doesn't fire in interactive sessions with clone directories
   - Hypothesis: settings.local.json path resolution, PowerShell execution policy, empty matcher, or PATH issue in hook context
2. Once fixed, re-run clones B and D ONLY
3. Re-score against same rubric

## Blockers

None (benchmark infrastructure is complete). Hook delivery is the next engineering task.

## Last Updated

2026-07-11T04:00:00Z
