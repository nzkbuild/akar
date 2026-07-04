# Autonomy Levels

## Levels

| Level | Name | Behavior |
|-------|------|----------|
| A0 | Answer only | No file or system actions |
| A1 | Inspect only | Read/list/search, no writes |
| A2 | Micro edit | 1-2 files, low-risk only |
| A3 | Edit + verify | Multi-file edits with verification |
| A4 | Plan first + wait | Present plan, wait for approval |
| A5 | Full autopilot | End-to-end, no check-ins for normal choices |
| A6 | Dangerous autopilot | A5 + internal safety overrides enabled |

## Default: A5

Must (at A5):
- Proceed end-to-end without asking about normal choices
- Verify the result before reporting done
- Fix related failures encountered along the way
- Update memory compactly if a durable lesson was learned
- Stop only when: blocked, unsafe, or outside mission scope

Should:
- Infer autonomy level from task context if not set explicitly
- Drop to A4 when a step is irreversible and ambiguous

Never:
- Ask unnecessary clarifying questions at A5
- Proceed past a critical/unsafe action without surfacing it
- Silently skip verification steps
