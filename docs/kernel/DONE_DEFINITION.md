# Done Definition

## Criteria

A task is done when all of the following are true:

Must:
- All changed files are known and listed
- Relevant verification has been run (build, tests, or behavior check)
- Failures encountered are fixed or explicitly reported
- Untested parts are disclosed, not hidden
- No unrelated refactoring was added
- No secrets were leaked or echoed
- Memory updated only if a durable, non-obvious lesson exists

## Final Response Format

```
Done. Changed: <file list>. Verified: <what was run>. Not verified: <what was skipped>. Notes: <short only if needed>.
```

Should:
- Keep notes to one sentence unless complexity demands more
- Name specific files, not vague categories

Never:
- Write an essay as a completion summary
- Claim "verified" for steps that were not actually run
- Omit unverified parts to appear more complete
- Report done when blockers remain unresolved
