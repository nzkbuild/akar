# Kernel Policy: Learning Intelligence

## Core Rule

Every real failure must produce at least one artifact. Silent absorption is not permitted.

## Output Rule

Every real failure → one of:
- Fixed code (compile_error, test_failure, weak_test, ui_slop, overdiff)
- Fixed behavior (request_spike, loop_retry, skill_conflict, verification_gap)
- Documented limitation (model_drift, gateway_error, dependency_risk)
- Future eval (any type where fix is deferred)

## Learning Loop

```
Observe → Measure → Classify → Adapt → Repair → Verify → Store → Add Eval → Apply
```

## Failure Taxonomy

| Type               | Description                                         |
|--------------------|-----------------------------------------------------|
| compile_error      | Code that does not compile or parse                 |
| test_failure       | Test that fails unexpectedly                        |
| weak_test          | Test passes but does not cover the failure          |
| context_bloat      | Context too large; relevant content evicted         |
| request_spike      | Mission used far more requests than expected        |
| loop_retry         | Same operation retried > 2 times without progress  |
| model_drift        | Model output inconsistent with prior behavior       |
| gateway_error      | Gateway routing or auth failure                     |
| hook_error         | Claude Code hook failed or produced bad output      |
| memory_conflict    | Two memory writes conflict on the same key          |
| skill_conflict     | Two skills claimed the same role                    |
| ui_slop            | Generated UI does not meet design bar               |
| overdiff           | Change larger than necessary for stated goal        |
| dependency_risk    | Introduced dependency with unclear maintenance      |
| migration_risk     | Schema or API change with no rollback path          |
| security_risk      | Change that weakens auth, perms, or data safety     |
| doctor_failure     | Doctor ran but could not diagnose or repair         |
| verification_gap   | Task marked done without observable evidence        |

## Learning Patch Format (LP-NNNN)

```yaml
id: LP-NNNN
date: <ISO date>
source: <run-id or session description>
trigger: "<what was observed>"
failure_type: <type from taxonomy>
observed: "<specific failure description>"
rule: "<the rule this patch adds or reinforces>"
fix_applied: "<what was changed, or 'deferred'>"
eval_added: "<eval file path, or 'none'>"
status: applied | deferred | blocked
```

## Must

- Classify every failure before closing a task.
- Write an LP entry for every skill_conflict, request_spike, and verification_gap.
- Add or reference an eval for every deferred fix.

## Never

- Mark a task done after a failure without an LP entry.
- Absorb a repeated failure with no new rule.
- Write an LP without a failure_type from the taxonomy.

## Effective

v0.1.1 (documented). v0.1.2 (live). See RFC-0004.
