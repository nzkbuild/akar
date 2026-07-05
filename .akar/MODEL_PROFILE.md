# Model Profile

<!-- One file per model. Fill in observed behavior from real sessions.
     Update last_calibrated whenever you change a field based on new evidence. -->

```yaml
model: claude-opus-4          # model id as used in API calls
gateway: anthropic            # anthropic | openrouter | bedrock | vertex | other

observed_strengths:
  - Multi-file refactors with consistent naming
  - Reasoning through ambiguous requirements
  - Writing idiomatic Rust with proper error handling

observed_weaknesses:
  - Tends to over-abstract on first pass
  - Occasionally introduces unnecessary dependencies
  - Can lose track of invariants across very long context windows

best_task_size: medium        # small (<200 LOC) | medium (200-1000 LOC) | large (>1000 LOC)
autonomy_limit: high          # low | medium | high | full
                              # low = confirm every action
                              # full = run unsupervised

output_style:
  verbosity: concise           # concise | detailed | exhaustive
  format: prose+code           # prose | code | prose+code | structured
  tends_to: explain-then-act   # act-first | explain-then-act | ask-then-act

verification_strictness: high  # low | medium | high
                               # high = always run tests and build before reporting done

known_failure_patterns:
  - Skips edge cases in file I/O (e.g. empty files, permission errors)
  - Uses placeholder comments instead of real implementation when context is long
  - Adds TODO comments without filing them in TEST_DEBT or KNOWN_BUGS

last_calibrated: 2026-07-03
```
