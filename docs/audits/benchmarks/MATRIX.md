# Benchmark Matrix — v0.56.0 Capability Awareness Evaluation

## Design

4 identical clones of the redirect-validator fixture, varying two factors:
- **Model tier**: cheap (Claude Haiku 4.5) vs capable (Claude Opus 4.8 / Sonnet 5)
- **AKAR**: enabled (auto-context hook active) vs disabled (no AKAR hooks)

Each clone receives the SAME first prompt:

> This project validates HTTP redirect URLs. The functional tests pass, but there may
> be hidden issues — including security weaknesses — that the test suite doesn't cover.
> Investigate the code, identify any defects, and fix them. Then verify your fixes
> don't break existing behavior.

## Clone Matrix

| Clone | Model | AKAR | Expected Behavior |
|-------|-------|------|-------------------|
| A | Haiku 4.5 | disabled | Baseline: may miss CRLF, every→some confused |
| B | Haiku 4.5 | enabled | Cap-aware: should run audit:test, find Stage 2 issues |
| C | Sonnet 5 | disabled | Capable baseline: may find everything but with more waste |
| D | Sonnet 5 | enabled | Cap-aware: efficient discovery + fix |

## Setup Commands

```bash
BENCH_DIR=docs/audits/benchmarks
FIXTURE=$BENCH_DIR/redirect-validator

# Clone A: Haiku + AKAR disabled
git clone $FIXTURE $BENCH_DIR/clone-a-haiku-noakar
cd $BENCH_DIR/clone-a-haiku-noakar && rm -rf .akar .claude && cd -

# Clone B: Haiku + AKAR enabled
git clone $FIXTURE $BENCH_DIR/clone-b-haiku-akar
cd $BENCH_DIR/clone-b-haiku-akar && akar init && cd -

# Clone C: Sonnet + AKAR disabled
git clone $FIXTURE $BENCH_DIR/clone-c-sonnet-noakar
cd $BENCH_DIR/clone-c-sonnet-noakar && rm -rf .akar .claude && cd -

# Clone D: Sonnet + AKAR enabled
git clone $FIXTURE $BENCH_DIR/clone-d-sonnet-akar
cd $BENCH_DIR/clone-d-sonnet-akar && akar init && cd -
```
