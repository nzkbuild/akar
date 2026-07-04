# AKAR Evaluation Plan

## Purpose

Measure whether AKAR improves Claude Code sessions in practice.
Not a model benchmark. AKAR is a governance/runtime layer.

## Install smoke test

1. `cargo build` passes
2. `akar --version` prints correct version
3. `akar bootstrap` creates `.akar/`
4. `akar doctor` returns OK
5. `akar eval` passes all scenarios

## Fresh project smoke test

1. Clone a new Rust project
2. Run `akar bootstrap`
3. Run `akar preflight "add a button"`
4. Run `akar run "add a button"`
5. Check `.akar/EVENT_LOG.jsonl` has an entry
6. Run `akar postmortem`
7. Expected: clean outcome, small diff budget, no skill conflicts

## Real repo workflow test

Run AKAR on a real project across 3-5 tasks.
Record: telemetry events, postmortem outcomes, learning patches created.
Compare before/after doctor state.

## Claude alone vs Claude + AKAR

Baseline: give Claude a task directly, observe output.
With AKAR: run `akar preflight` → `akar run` → `akar postmortem`.

Measure:

| Signal | Target |
|---|---|
| Files changed | AKAR should keep smaller |
| LOC changed | AKAR enforces diff budget |
| Unrelated edits | AKAR should reduce |
| Verification honesty | AKAR should never fake done |
| Risk detection | AKAR should flag high-risk tasks |
| Skill conflict detection | AKAR should warn before mission |
| Telemetry correctness | Events should match actual outcomes |

## What AKAR does not benchmark

AKAR is not SWE-bench. AKAR does not benchmark model intelligence.
AKAR benchmarks governance: did the model stay within scope?
SWE-bench/Aider-style benchmarks become relevant only when AKAR wraps real code execution (v0.3+).

## Evaluation cadence

Run eval suite on every release: `akar eval`
Add new eval scenarios for every real failure observed.
Target: 50+ evals by v1.0.
