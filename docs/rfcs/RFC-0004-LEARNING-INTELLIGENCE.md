# RFC-0004: Learning Intelligence

- Status: Accepted
- Date: 2026-07-04
- Author: AKAR Kernel Team

## Problem

AKAR v0.1.0 repairs code when tests fail but does not repair its own behavior
when it fails structurally. Skill conflicts, request spikes, and verification
gaps were observed and then forgotten. The overnight run produced failures with
no corresponding behavior change. The same failure modes will recur.

## Why Now

Without learning intelligence, AKAR is a runtime that degrades silently.
The overnight run is a concrete dataset. It must produce patches, not just logs.

## Goals

- Every real failure produces at least one artifact.
- Failures are classified so patterns can be detected.
- Learning patches are structured, reviewable, and applicable.
- The system improves between runs, not just within them.

## Non-Goals

- Does not implement autonomous self-modification without human review.
- Does not replace evals; it adds evals.
- Does not cover every possible failure; covers classified failure types.

## Proposed Design

### Learning Loop

```
Observe → Measure → Classify → Adapt → Repair → Verify → Store → Add Eval → Apply
```

1. **Observe:** Detect a failure event (error, unexpected output, gap).
2. **Measure:** Quantify impact (requests wasted, tasks incomplete, conflicts).
3. **Classify:** Assign a failure type from the taxonomy.
4. **Adapt:** Determine the fix category (code fix, behavior fix, limitation doc,
   future eval).
5. **Repair:** Apply the fix or document why it cannot be applied now.
6. **Verify:** Confirm the fix addresses the root cause.
7. **Store:** Write a Learning Patch (LP-NNNN).
8. **Add Eval:** Write or reference an eval that would catch this failure.
9. **Apply:** Merge the patch into the relevant kernel policy or source file.

### Failure Taxonomy

| Type               | Description                                         |
|--------------------|-----------------------------------------------------|
| compile_error      | Code that does not compile or parse                 |
| test_failure       | Test that fails unexpectedly                        |
| weak_test          | Test that passes but does not cover the failure     |
| context_bloat      | Context too large; relevant content evicted         |
| request_spike      | Mission used far more requests than expected        |
| loop_retry         | Same operation retried > 2 times without progress  |
| model_drift        | Model output inconsistent with prior behavior       |
| gateway_error      | Gateway routing or auth failure                     |
| hook_error         | Claude Code hook failed or produced bad output      |
| memory_conflict    | Two memory writes conflict on the same key          |
| skill_conflict     | Two skills claimed the same role                    |
| ui_slop            | Generated UI does not meet design bar               |
| overdiff           | Change larger than necessary for the stated goal    |
| dependency_risk    | Introduced dependency with unclear maintenance      |
| migration_risk     | Schema or API change with no rollback path          |
| security_risk      | Change that weakens auth, perms, or data safety     |
| doctor_failure     | Doctor ran but could not diagnose or repair         |
| verification_gap   | Task marked done without observable evidence        |

### Learning Patch Format (LP-NNNN)

```yaml
id: LP-0001
date: 2026-07-04
source: overnight-autopilot-run
trigger: "Superpower + GSD both active; methodology undefined"
failure_type: skill_conflict
observed: "Two methodology controllers active simultaneously"
rule: "Only one Methodology role may be active per mission"
fix_applied: "RFC-0002 Skill Intelligence; SkillRole classification added"
eval_added: "eval/skill_conflict_detection.eval"
status: applied
```

### Output Rule

Every real failure must produce one of:
- Fixed code (compile_error, test_failure, weak_test, ui_slop, overdiff)
- Fixed behavior (request_spike, loop_retry, skill_conflict, verification_gap)
- Documented limitation (model_drift, gateway_error, dependency_risk)
- Future eval (any type where fix is deferred)

Silently absorbing a failure is not permitted.

## Acceptance Criteria

- [ ] Failure taxonomy published in kernel policy.
- [ ] LP format defined with at least one real example from overnight run.
- [ ] Learning loop steps documented and linked from kernel policy.
- [ ] At least 3 LPs written for overnight run failures before v0.1.2.

## Rollback

Remove LP format and taxonomy. Failures are logged but not acted on.
Reintroduces silent degradation.

## Decision

Accepted. Effective v0.1.1 (documented). Implementation target: v0.1.2.
