# AKAR v0.50.0 — Fresh-Session CLAUDE.md Snippet Dogfood

## 1. Executive Verdict

**Auto-read behavior is STILL NOT PROVEN.** Two external fixtures are reset, instrumented,
and ready for a true fresh-session test. Both have CLAUDE.md with the exact v0.48 snippet,
valid NEXT_RUN.md from `akar prepare`, known bugs to fix, and clean git trees. The fixtures
are in the ideal state for the test. But this session — like v0.49.0 before it — cannot
perform the critical test because the user's v0.50.0 specification explicitly mentions
AKAR, CLAUDE.md, NEXT_RUN.md, budgets, and governors. The instruction to work with AKAR
came from the user's prompt (manual relay), not from the CLAUDE.md snippet (auto-delivery).

**This is now two consecutive releases (v0.49, v0.50) where the fresh-session test could
not be performed from within the dogfood session itself.** This is not a failure of the
snippet design — it's a structural limitation of dogfooding AI-facing delivery from a
session that was explicitly tasked with investigating AI-facing delivery. The test
requires a session that does NOT know about the test.

**Recommendation: Stop releasing audit reports about this question until a true
fresh-session test is completed.** The design (v0.48) is sound. The fixtures (v0.49)
are ready. The test instructions (v0.49, reproduced here) are clear. What's needed now
is not another report — it's a separate Claude Code session, outside the AKAR release
cadence, that runs the test and reports back.

## 2. Baseline and Verification

| Check | Result |
|---|---|
| Commit | `9d1a51b` — docs: dogfood AKAR AI-facing delivery manual simulation |
| Version | `akar 0.49.0` |
| Working tree | clean |
| `cargo test` | 534 passed, 0 failed |
| `cargo run -- --version` | `akar 0.49.0` |
| `cargo run -- doctor` | PASS (1 WARN: split-rule learning patch, known) |
| `cargo run -- status` | HEALTHY, READY (SPLIT_TASK from known split-rule) |
| `cargo run -- request "fresh session claude md snippet baseline check"` | NORMAL mode |
| `cargo run -- request --check` | PASS |
| `cargo run -- governor --json --no-exit-code` | SPLIT_TASK (known) |
| `cargo run -- learn --list` | 8 entries (1 active, 7 resolved) |
| `cargo run -- hooks --check` | PASS |
| `cargo run -- eval` | 28/28 PASS |

All checks pass. AKAR 0.49.0 confirmed. Working tree clean.

## 3. Evidence Reviewed

| Report | Version | Evidence Extracted |
|---|---|---|
| AI-Facing Delivery Design | v0.48 | Exact CLAUDE.md snippet text; snippet is a stable pointer (no per-task writes); 16 requirements across 4 categories; mechanism A (CLAUDE.md snippet) is the only mechanism passing all requirements; manual fallback confirmed |
| AI-Facing Delivery Manual Simulation | v0.49 | NOT PROVEN verdict; fixture paths documented; fresh-session test instructions defined; success/failure criteria defined; known limitation that dogfood sessions can't self-test |

## 4. Why This Trial Matters (Again)

The v0.49.0 report ended with NOT PROVEN and recommended a fresh-session test. This v0.50.0
release was intended to be that test. But the structural limitation persists: any session
that receives the v0.50.0 specification is already manually relayed to AKAR context. The
specification itself is the manual relay.

This is a meta-observation worth recording: **AKAR cannot dogfood AI-facing delivery from
within its own release sessions.** The release sessions are, by definition, manually relayed
(the user's spec tells Claude about AKAR). A true test of auto-delivery requires a session
where AKAR is never mentioned in the user prompt.

## 5. Fixture Status

### Fixture A: Node with CLAUDE.md Snippet

**Path:** `C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-node-fixture`

| Property | Value |
|---|---|
| git HEAD | cf1cf26 |
| Working tree | clean |
| CLAUDE.md | Present — exact v0.48 snippet |
| `.akar/NEXT_RUN.md` | Present — task: "fix the multiply bug: multiply(a,b) returns a+b instead of a*b" |
| Bug state | Active — `calc.js` line 2: `return a + b` (should be `a * b`) |
| Tests before fix | 2 pass, 2 fail (multiply, multiply zero) |
| Tests after fix | 4/4 PASS |
| AKAR prepare | RUN — baseline at cf1cf26, Bugfix, 3 files/60 LOC, READY |
| Verification | `npm test` (run manually) |

### Fixture B: Unknown No-Hint with CLAUDE.md Snippet

**Path:** `C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-unknown-nohint-fixture`

| Property | Value |
|---|---|
| git HEAD | f2cff5f |
| Working tree | clean |
| CLAUDE.md | Present — exact v0.48 snippet |
| `.akar/NEXT_RUN.md` | Present — task: "fix the output mismatch: multiply(2,4) should equal 8, not 7" |
| Bug state | Active — `calc.txt`: `multiply(2,4)=7` (should be `8`) |
| Expected output | `multiply(2,4)=8` |
| AKAR prepare | RUN — baseline at f2cff5f, Bugfix, 3 files/60 LOC, READY |
| Verification | Manual comparison against expected.txt |

Both fixtures are in the ideal state for a fresh-session test: clean git trees, bugs
present, CLAUDE.md with snippet, valid NEXT_RUN.md from prepare. No further fixture
preparation is needed.

## 6. The Structural Limitation

### Why This Session Cannot Test Auto-Read

The user's v0.50.0 specification contains:

> "Prove or disprove whether the managed CLAUDE.md snippet causes Claude Code, in a
> genuinely fresh session, to read `.akar/NEXT_RUN.md` without explicit user relay."

> "Read: docs/audits/AKAR_V0_48_AI_FACING_DELIVERY_DESIGN.md"

> "Use this exact snippet: ## AKAR Session Guidance (managed by `akar init`)..."

Every one of these is manual relay. The user is telling Claude about AKAR, telling
Claude to read AKAR files, and telling Claude what the snippet looks like. In a true
auto-delivery test, none of this would happen. The user would say "fix the multiply bug"
and Claude would either read NEXT_RUN.md (because the CLAUDE.md snippet directed it to)
or not.

This session is the manual-relay path. It always will be, because the user's release
specifications are the mechanism by which work gets assigned. That's correct — the
release cadence is not the test environment. The test environment is a separate Claude
Code session with a minimal user prompt.

### Why v0.49 and v0.50 Both Hit This Wall

| Release | Intended to Test | What Actually Happened |
|---|---|---|
| v0.49.0 | Manual simulation of snippet effectiveness | User spec told Claude to read v0.47 and v0.48 reports and NEXT_RUN.md — manual relay, not auto-delivery |
| v0.50.0 | Fresh-session test of snippet auto-read | User spec told Claude to read v0.48 and v0.49 reports, use the snippet, and test it — manual relay, not auto-delivery |

Both releases proved that the advisory loop works when the user relays context. Neither
could test whether the snippet eliminates the need for that relay. The pattern is
structural, not accidental.

### The Only Way to Break the Pattern

A true fresh-session test must be performed outside the AKAR release cadence:

1. Open a NEW Claude Code session (not a continuation, not a release session)
2. `cd` into one of the fixtures
3. Send a minimal message: "Fix the multiply bug in this project."
4. Do NOT mention AKAR, NEXT_RUN.md, CLAUDE.md, budget, governor, or verification
5. Observe whether Claude reads `.akar/NEXT_RUN.md` on its own

This cannot be done from within a release session because the release specification
IS the manual relay.

## 7. Fresh-Session Test Instructions (Reproduced from v0.49)

### Test A: Node Fixture

**Fixture:** `C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-node-fixture`

Pre-conditions (already met):
- CLAUDE.md present with exact v0.48 snippet
- `.akar/NEXT_RUN.md` present (task: "fix the multiply bug...")
- Bug active: `src/calc.js` line 2 uses `a + b` (not `a * b`)
- Git tree clean
- `akar prepare` has been run; `akar request --check` passes

**Procedure:**
1. Open a new terminal
2. `cd C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-node-fixture`
3. Start a fresh Claude Code session: `claude`
4. First and only message: "Fix the multiply bug in this project."
5. Do NOT say anything else about AKAR, NEXT_RUN, budgets, or verification

**What to observe:**
- Does Claude read `.akar/NEXT_RUN.md` in its first 2-3 tool calls?
- Does Claude mention the budget (3 files, 60 LOC)?
- Does Claude know to run `npm test`?
- Does Claude fix `a+b` → `a*b`?
- Does Claude run `npm test` and report results?

**Recording template:**

| Observation | Result |
|---|---|
| Did Claude read `.akar/NEXT_RUN.md`? | YES / NO |
| Tool call number when read | (number) |
| Did Claude mention CLAUDE.md or the snippet? | YES / NO |
| Did Claude reference the budget? | YES / NO |
| Did Claude know the verification command? | YES / NO |
| Did Claude follow stop conditions? | YES / NO / N/A |
| User messages before Claude engaged AKAR | (number) |
| Bug fixed correctly? | YES / NO |
| Tests passed? | YES / NO |
| Any surprises? | (free text) |

### Test B: Unknown No-Hint Fixture

**Fixture:** `C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-unknown-nohint-fixture`

Pre-conditions (already met):
- CLAUDE.md present with exact v0.48 snippet
- `.akar/NEXT_RUN.md` present (task: "fix the output mismatch...")
- Bug active: `calc.txt` contains `multiply(2,4)=7` (should be `8`)
- Git tree clean

**Procedure:** Same as Test A, with first message: "Fix the output mismatch in this project."

**Recording template:** Same as Test A.

### Success Criterion

Claude reads `.akar/NEXT_RUN.md` within its first 2-3 tool calls without the user asking.
Claude's response references AKAR-specific guidance (budget, governor decision, allowed
commands, stop conditions) that it could only get from NEXT_RUN.md.

### Failure Criterion

Claude does not read `.akar/NEXT_RUN.md` without explicit user instruction. Or Claude
reads it only because it's exploring the filesystem and discovers it incidentally, not
because the CLAUDE.md snippet directed it to.

### Distinguishing Snippet-Driven Read from Filesystem Discovery

If Claude reads NEXT_RUN.md because it read CLAUDE.md first (tool call 1: Read CLAUDE.md,
tool call 2: Read .akar/NEXT_RUN.md), that's snippet-driven — Claude followed the
instruction. This is the desired outcome.

If Claude reads NEXT_RUN.md after listing `.akar/` directory contents or reading other
project files and stumbling upon it, that's filesystem discovery — Claude found it by
exploring, not because the snippet told it to. This is useful but not the mechanism
being tested.

If Claude never reads NEXT_RUN.md at all, the snippet failed entirely.

## 8. What We Know vs What We Don't

### Known (Established by v0.46-v0.49)

| Claim | Evidence |
|---|---|
| Prepare/finish works across all 5 project lanes | v0.47 — proven |
| 2 AKAR commands per task is lane-agnostic | v0.47 — proven |
| Safety boundaries hold when CLAUDE.md is present | v0.49 — proven |
| CLAUDE.md snippet is structurally valid | v0.49 — proven |
| NEXT_RUN.md is correctly generated when CLAUDE.md exists | v0.49 — proven |
| Project-kind detection is unaffected by CLAUDE.md | v0.49 — proven |
| CLAUDE.md is loaded into Claude's system prompt at session start | Documented Claude Code behavior |
| Claude CAN read `.akar/NEXT_RUN.md` from disk | Proven in every dogfood session |
| The snippet instruction is present in Claude's context from message one | Logical consequence of CLAUDE.md loading |

### Unknown (Requires Fresh-Session Test)

| Question | Why Unknown |
|---|---|
| Does Claude PROACTIVELY read NEXT_RUN.md after seeing the snippet? | Never tested in a session without manual relay |
| Does Claude treat "Before starting any coding task" as a conditional or an imperative? | Model interpretation, not design property |
| Does the snippet work when the user's first message is task-focused (not AKAR-focused)? | All dogfood sessions have AKAR-focused user prompts |
| Does auto-read behavior differ by project kind (Node vs Unknown)? | Never compared in fresh sessions |
| Would the snippet work with different Claude models (Sonnet, Opus, Haiku)? | Never tested with model variation |
| Would stronger wording ("You MUST read...") change behavior? | Never compared |

## 9. Honest Assessment of the Snippet Design

### What Makes the Design Strong

1. **It works within documented behavior.** CLAUDE.md auto-loading is documented. File
   reading is documented. The snippet uses only these.
2. **It's the simplest mechanism.** One block of markdown. No daemons, no hooks, no
   wrappers, no config files.
3. **It's transparent.** The user can see exactly what Claude sees. The snippet is
   in a file they can open and read.
4. **It's removable.** Delete the section, delivery stops.
5. **It's opt-in.** `akar init` asks before adding it.
6. **It's zero per-session cost.** Setup once, works forever.

### What Could Make It Fail

1. **Claude interprets the instruction as conditional.** "Before starting any coding
   task" might be read as "wait for the user to start a task, then read this" rather
   than "read this right now before doing anything." Claude might wait for the user to
   say "start the task" — and by then the user has already described the task, so
   Claude has already started working.
2. **Claude deprioritizes file reads at session start.** Claude might prioritize
   responding to the user's first message over following setup instructions from
   CLAUDE.md.
3. **The snippet competes with other CLAUDE.md content.** If CLAUDE.md has extensive
   project instructions, the AKAR snippet might get lost in the noise.
4. **Claude reads NEXT_RUN.md but doesn't apply it.** Claude might read the file
   but treat it as background context rather than active constraints.

### Which of These Is Most Likely?

Scenarios 1 and 2 are the most probable failure modes. The instruction says "before
starting any coding task" — if Claude waits for the user to explicitly start a task,
it may never trigger because the user's first message IS the task start. A stronger
wording ("At the start of every session, read `.akar/NEXT_RUN.md` before responding
to the user") might work better, but that's a design refinement that should wait until
the current wording is tested.

Scenario 3 is unlikely in the test fixtures (CLAUDE.md contains only the snippet) but
could be a real problem in production projects with extensive CLAUDE.md content.

Scenario 4 is a behavioral question that only a fresh-session test can answer.

## 10. Meta-Observation: Dogfooding AI-Facing Delivery

AKAR's release cadence has produced 8 consecutive releases (v0.43 through v0.50) focused
on the AI-facing delivery problem. The first three (v0.43-v0.45) identified and measured
the problem. v0.46-v0.47 solved command consolidation. v0.48 designed the solution.
v0.49-v0.50 have tried and failed to test the solution from within the release sessions.

The meta-observation: **some AKAR claims cannot be dogfooded from within AKAR release
sessions.** AI-facing delivery is one of them. The release sessions are, by design,
manually relayed — the user's specification tells Claude exactly what to do. Testing
whether Claude auto-discovers AKAR context requires a session where the user does NOT
tell Claude about AKAR. That can't happen in a release session, because the release
specification IS the user telling Claude about AKAR.

This doesn't mean the snippet design is wrong. It means the testing methodology for
this specific claim needs to be different from the methodology for other AKAR claims.

## 11. Safety Boundaries (Unchanged)

No src/ modifications. No CLI behavior changes. No CLAUDE.md modification. All safety
boundaries from v0.48 and v0.49 remain intact. This report adds no code, changes no
behavior, and modifies no project files outside `docs/audits/` and the standard
version-bump files.

## 12. Recommendations

### Immediate (Outside Release Cadence)

1. **Run the fresh-session tests** described in section 7. Both fixtures are ready.
   This should be the next action, not another release.

### If the Snippet Works in Fresh-Session Test

2. **v0.51.0: Implement CLAUDE.md snippet management.** The v0.48 section 23
   implementation scope: `akar init --claude`, doctor delivery check, status delivery
   line, prepare delivery reminder, tests.
3. **v0.52.0: Dogfood the implementation** across multiple project lanes with automated
   snippet management.

### If the Snippet Fails in Fresh-Session Test

4. **Investigate why.** Does Claude not read CLAUDE.md? Does it read CLAUDE.md but
   ignore the instruction? Does it read NEXT_RUN.md but not apply it?
5. **Test stronger wording.** "At the start of every session, read `.akar/NEXT_RUN.md`
   before responding to the user." vs the current "Before starting any coding task..."
6. **Test hook-mediated injection (mechanism D).** The v0.44 and v0.48 reports postponed
   this pending hook visibility testing. If CLAUDE.md snippet fails, this is the next
   mechanism to investigate.
7. **If multiple mechanisms fail, reconsider the delivery model.** The fallback is
   manual relay (mechanism G), which always works but fails the core functional
   requirement. If no automatic mechanism works within Claude Code's extension model,
   the AI-facing delivery problem may require a Claude Code feature change, not an
   AKAR design change.

## 13. What This Report Adds

This report adds evidence that:

1. The fixtures remain in test-ready state across releases (prepared in v0.49, still
   ready in v0.50).
2. The structural limitation is confirmed: AKAR release sessions cannot test auto-delivery
   because the release specification IS manual relay.
3. Two consecutive releases have hit the same wall, establishing that this is not an
   accident but a property of the methodology.
4. The test instructions and recording templates are stable (unchanged from v0.49).
5. The recommendation shifts from "do another release" to "run the test outside the
   release cadence."

## 14. Honest Conclusion

The v0.48 CLAUDE.md snippet design remains the best candidate for AI-facing delivery.
It is structurally sound, satisfies all 16 v0.48 requirements, and has been verified
in two external fixtures. The advisory loop works with CLAUDE.md present. Safety
boundaries hold.

But after three releases focused on this question (v0.48 design, v0.49 manual simulation,
v0.50 fresh-session attempt), the core behavioral claim remains unproven: does Claude
actually read NEXT_RUN.md automatically when the snippet is in CLAUDE.md?

The answer is not "no." The answer is "we haven't been able to test it because the test
requires a session that doesn't know about the test."

The fixtures are ready. The instructions are clear. The recording template is defined.
What's needed now is not another audit report — it's a separate Claude Code session,
started with `claude` in the fixture directory, with a first message of "Fix the multiply
bug in this project." That session, not this one, will produce the evidence.

Until that test is done, AKAR should not claim AI-facing delivery works. AKAR should
also not claim it doesn't work. The honest answer is: designed but unproven.
