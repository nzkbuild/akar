# Verification Ladder

Tests are evidence, not proof. Climb as high as the task warrants.

## Levels

**0 — Code understanding**
Read and understand what changed and why.

**1 — Typecheck / build**
Run compiler or type checker. No type errors, no build failures.

**2 — Existing tests**
Run the relevant test suite. No regressions introduced.

**3 — Meaningful behavior test**
Write or run a test that exercises the changed behavior directly.

**4 — Real user / system flow**
Drive the actual flow in the running app or system. Observe the outcome.

**5 — Edge cases**
Test boundary conditions, empty states, error paths.

**6 — Regression risk**
Identify what adjacent behavior could have broken. Test or flag it.

**7 — UX / security / accessibility**
If the change affects UI, auth, or data exposure — verify those dimensions specifically.

## Rules

Must:
- Reach at least level 2 before reporting done on any code change
- Disclose which levels were not reached

Should:
- Reach level 4 for user-facing or system-critical changes
- Reach level 7 for any change touching auth, payments, or accessibility

Never:
- Report done at level 0 or 1 alone
- Claim a level was reached without actually running it
