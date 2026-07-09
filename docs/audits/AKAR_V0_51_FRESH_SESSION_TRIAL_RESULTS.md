# AKAR v0.51.0 — Fresh-Session Trial Results

## 1. Executive Verdict

**The CLAUDE.md snippet likely works for AI-facing delivery, but the current wording is
not safe enough.** Two of three fresh-session trials passed — Claude read `.akar/NEXT_RUN.md`
and applied AKAR-specific context with zero explicit AKAR/NEXT_RUN relay. But Trial C
exposed a stale-context vulnerability: NEXT_RUN from a previous task influenced a new,
unrelated task. The stale-context behavior is unsafe for implementation.

**Do not implement `akar init --claude` yet.** The snippet needs a stale-context revision
first.

## 2. Trial Results

### Trial A: Node Fixture — PASS

**Fixture:** `akar-dogfood-v049-snippet-node-fixture`  
**User message:** "Fix the multiply bug in this project."  
**Zero AKAR/NEXT_RUN relay in user message.**

| Observation | Result |
|---|---|
| Did Claude read `.akar/NEXT_RUN.md`? | YES |
| Did Claude reference AKAR-specific context? | YES — baseline, budget (3 files / 60 LOC), governor |
| Did Claude know the verification command? | YES — `npm test` |
| Bug fixed correctly? | YES — `a+b` → `a*b` plus zero-edge-case fix |
| Tests passed? | YES — 4/4 PASS |
| `akar finish` | PASS |
| Commit | `066a98f` |

Claude read NEXT_RUN.md from the CLAUDE.md snippet alone, followed the budget
constraints, ran the correct verification, and completed the task without the user
ever mentioning AKAR. This is the desired behavior.

### Trial B: Unknown No-Hint Fixture — PASS

**Fixture:** `akar-dogfood-v049-snippet-unknown-nohint-fixture`  
**User message:** "Fix the output mismatch in this project."  
**Zero AKAR/NEXT_RUN relay in user message.**

| Observation | Result |
|---|---|
| Did Claude read `.akar/NEXT_RUN.md`? | YES |
| Did Claude reference AKAR-specific context? | YES — baseline, governor |
| Did Claude know the verification command? | YES — manual comparison guidance from NEXT_RUN |
| Bug fixed correctly? | YES — `multiply(2,4)=7` → `8` |
| Output correct? | YES |
| `akar finish` | PASS |
| Commit | `2efb823` |

Claude read NEXT_RUN.md from the snippet, applied AKAR context, and completed the
manual-verification task correctly. Works across project kinds (Node and Unknown).

### Trial C: Stale-Context — FAIL / NEEDS SNIPPET REVISION

**Fixture:** (same Node fixture as Trial A, but with intentionally stale NEXT_RUN)  
**NEXT_RUN.md content:** task = "fix the multiply bug" (from previous session)  
**User message:** "Add a square function to this project."  
**Zero AKAR/NEXT_RUN relay in user message.**

| Observation | Result |
|---|---|
| Did Claude read `.akar/NEXT_RUN.md`? | YES |
| Did Claude add the square function? | YES |
| Did Claude also fix multiply? | YES — followed stale NEXT_RUN |
| Tests passed? | YES |
| `akar finish` | PASS |
| Problem | Stale NEXT_RUN caused Claude to ALSO fix multiply when the user only asked for square |

**The stale NEXT_RUN from a previous task influenced a new, unrelated task.** Claude
did what the snippet told it to ("Before starting any coding task, read .akar/NEXT_RUN.md")
— but NEXT_RUN was stale, and Claude treated its task as additional work beyond the user's
request. This is unsafe for implementation: a stale NEXT_RUN in a project with CLAUDE.md
snippet could cause Claude to execute work the user didn't request, or worse, apply
constraints (budgets, stop conditions) from a previous task to the current one.

## 3. What Was Proven

| Claim | Status |
|---|---|
| CLAUDE.md snippet causes Claude to read NEXT_RUN.md without user relay | PROVEN (Trials A, B) |
| AKAR-specific context (budget, baseline, governor) reaches Claude via snippet | PROVEN (Trials A, B) |
| Works across project kinds (Node, Unknown) | PROVEN (Trials A, B) |
| Works with no hints (Unknown no-hint) | PROVEN (Trial B) |
| Snippet is safe for single-session use | PROVEN (Trials A, B) |
| Snippet is safe across sessions with stale NEXT_RUN | **DISPROVEN** (Trial C) |

## 4. The Stale-Context Problem

The current snippet reads:

> Before starting any coding task, read `.akar/NEXT_RUN.md`

The instruction is unconditional — if NEXT_RUN.md exists on disk, Claude reads it. But
NEXT_RUN.md persists on disk after the task it was created for. If the user starts a new
session with a different task, the old NEXT_RUN.md is still there, and Claude reads it.

This is a failure mode the v0.48 design did not anticipate in detail. The design noted
that NEXT_RUN.md is per-task volatile state and should not be inlined in CLAUDE.md — that
part is correct. But the design did not account for NEXT_RUN.md being stale from a
previous session.

### Why This Matters

- Claude treated NEXT_RUN's task as authoritative alongside the user's task
- If NEXT_RUN had contained `SAFETY: BLOCKED` or `SPLIT_TASK` from a previous session,
  Claude might have refused to work or applied wrong constraints
- The snippet creates a persistence hazard: stale AKAR state on disk can silently
  influence new sessions

### Required Revision

The snippet must distinguish fresh from stale NEXT_RUN. Options to evaluate:

1. **Timestamp guard:** "If `.akar/NEXT_RUN.md` was written less than N hours ago, read it."
   Problem: arbitrary threshold, fragile.
2. **Session-id guard:** `akar prepare` writes a session ID into NEXT_RUN; snippet checks it
   against a known value. Problem: how does Claude know the expected session ID?
3. **Freshness marker:** `akar prepare` writes a freshness marker; `akar finish` clears or
   invalidates it. Claude checks the marker before applying NEXT_RUN. Simplest option with
   clear lifecycle.
4. **Scope the instruction:** "If you are continuing a previous AKAR session, read
   `.akar/NEXT_RUN.md`." Problem: Claude can't know if it's continuing.
5. **Prepare-and-clear model:** `akar prepare` writes NEXT_RUN.md; snippet says "read it
   once, then ignore it for this session"; `akar finish` archives or deletes NEXT_RUN.md.
   This is the most complete lifecycle but requires `akar finish` to clean up.

The revision should be designed in a separate release (v0.52.0 or similar).

## 5. Implementation Readiness

**NEEDS SNIPPET REVISION.** The snippet mechanism works — Claude reads NEXT_RUN.md from
CLAUDE.md snippet alone. But stale-context is a real safety concern that must be resolved
before `akar init --claude` can be implemented.

| Gate | Status |
|---|---|
| Snippet causes auto-read: proven? | YES |
| Auto-read works across project kinds? | YES |
| Snippet is safe for multi-session use? | **NO — stale-context failure** |
| Ready to implement `akar init --claude`? | **NO** |

## 6. Baseline and Verification

| Check | Result |
|---|---|
| Commit | `776ae08` — docs: dogfood AKAR fresh-session CLAUDE.md snippet |
| Version | `akar 0.50.0` |
| Working tree | clean |
| `cargo test` | 534 passed, 0 failed |
| `cargo run -- --version` | `akar 0.50.0` |
| `cargo run -- eval` | 28/28 PASS |

All checks pass. AKAR 0.50.0 confirmed.

## 7. Safety Boundaries

No src/ modifications. No CLI behavior changes. No CLAUDE.md modification. No
`akar init --claude` implementation. All safety boundaries from v0.48–v0.50 remain intact.

## 8. Recommendations

1. **Design a stale-context revision for the snippet** — evaluate options in section 4,
   select the simplest mechanism that prevents stale NEXT_RUN from influencing new sessions.
2. **Re-test with revised snippet** — re-run all three trial scenarios with the revised
   wording, especially Trial C (stale-context).
3. **Only implement `akar init --claude` after** stale-context is resolved and re-tested.

## 9. Honest Conclusion

After five releases focused on AI-facing delivery (v0.48 design → v0.49 manual simulation
→ v0.50 fresh-session attempt → actual fresh-session trials → v0.51 results), we have our
answer:

**The CLAUDE.md snippet works.** Claude reads NEXT_RUN.md without user relay and applies
AKAR context. The mechanism is sound.

**The current wording is not safe enough.** Stale NEXT_RUN from a previous session can
influence a new, unrelated task. This is a real failure mode discovered through testing.

The path forward is clear: revise the snippet to handle stale context, re-test, then
implement. The design was ~90% correct — the missing piece is freshness/lifecycle, not
the delivery mechanism itself.
