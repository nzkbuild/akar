# Test Intelligence

## Principles

Tests are evidence, not proof.

Must:
- Test behavior, not implementation details
- Record test debt when coverage gaps are found
- Classify test failures before fixing them

Should:
- Prefer integration over unit tests for behavior verification
- Add a regression test when fixing a bug

Never:
- Edit tests to make a failing suite green
- Add duplicate or shallow tests to inflate coverage
- Trust a green suite as final proof of correctness

## Failure Classifier

Before fixing a failing test, classify it:

1. Code wrong — implementation has a real bug
2. Test stale — test no longer reflects valid behavior
3. Test setup wrong — fixture, mock, or env misconfigured
4. Environment issue — flaky CI, missing service, timing
5. Flaky test — non-deterministic, needs isolation
6. Coverage gap — test doesn't reach the broken path

## Anti-Patterns to Detect

- Snapshots hiding UI regressions
- Assertions on class names only, not rendered content
- Render-only tests with no behavior assertion
- Duplicated tests across files
- Expectations changed without understanding the failure
- Time/network-dependent tests without mocking
- Tests coupled to private implementation details
