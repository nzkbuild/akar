# AKAR v0.49.0 — AI-Facing Delivery Manual Simulation Dogfood

## 1. Executive Verdict

**The CLAUDE.md snippet mechanism is structurally sound but auto-read behavior is NOT
PROVEN.** Two external fixtures with the exact v0.48 snippet were prepared, fixed, and
finished — the full advisory loop completed cleanly on both. The fixtures are instrumented
and ready for a fresh-session test. However, this session could not perform the critical
test: whether Claude Code, in a fresh session with no prior instructions, loads CLAUDE.md
and automatically reads `.akar/NEXT_RUN.md` without the user explicitly asking.

**Why not proven from this session:** This session was explicitly instructed by the user's
v0.49.0 specification to read NEXT_RUN.md. The instruction came from the user's prompt,
not from the CLAUDE.md snippet in the fixture. This is the manual-relay path — the exact
thing the snippet is designed to eliminate. A true test requires a fresh session where
the first instruction is only "start Claude Code in the fixture directory" with no mention
of AKAR, NEXT_RUN.md, or reading any file.

**The fixtures are ready and instrumented.** Both have CLAUDE.md with the exact v0.48
snippet, valid NEXT_RUN.md from `akar prepare`, known bugs to fix, and known expected
outcomes. Follow-up test instructions are provided in section 13.

**Recommendation:** Release this report as honest evidence. Do not claim the snippet
works until a fresh-session test confirms it. The structural properties of the mechanism
(CLAUDE.md auto-loading is documented Claude Code behavior, the snippet points to a file
that exists, Claude can read files) are well-understood. What remains unproven is the
behavioral property: does Claude actually follow the instruction without user prompting?

## 2. Baseline and Verification

| Check | Result |
|---|---|
| Commit | `3212bee` — docs: design AKAR AI-facing delivery |
| Version | `akar 0.48.0` |
| Working tree | clean |
| `cargo test` | 534 passed, 0 failed |
| `cargo run -- --version` | `akar 0.48.0` |
| `cargo run -- doctor` | WARN (split-rule learning patch, known) |
| `cargo run -- status` | HEALTHY, SPLIT_TASK (known) |
| `cargo run -- request "ai-facing delivery manual simulation baseline check"` | NORMAL mode |
| `cargo run -- request --check` | PASS |
| `cargo run -- governor --json --no-exit-code` | SPLIT_TASK (known) |
| `cargo run -- learn --list` | 8 entries (1 active, 7 resolved) |
| `cargo run -- hooks --check` | PASS |
| `cargo run -- eval` | 28/28 PASS |

All checks pass. AKAR 0.48.0 confirmed. Working tree clean.

## 3. Evidence Reviewed

Two prior reports read in relevant sections for v0.49 context:

| Report | Version | Evidence Extracted |
|---|---|---|
| Prepare/Finish Cross-Lane Dogfood | v0.47 | 2 commands per task across 5 lanes; safety boundaries hold; remaining bottleneck is AI-facing delivery; project-kind detection is correct |
| AI-Facing Delivery Design | v0.48 | CLAUDE.md snippet accepted as primary; 10-line exact snippet text; snippet is stable pointer (no per-task writes); manual fallback confirmed; 9 failure modes catalogued; future implementation scope defined |

## 4. Why This Trial Matters

v0.48.0 designed the CLAUDE.md snippet mechanism with 16 requirements across 4 categories.
The design is internally consistent and satisfies all requirements on paper. But the core
behavioral claim — that Claude reads `.akar/NEXT_RUN.md` automatically when CLAUDE.md
contains the instruction — has never been tested.

This trial creates the conditions for that test:
- Two external fixtures with CLAUDE.md containing the exact v0.48 snippet
- Valid NEXT_RUN.md compiled by `akar prepare`
- Known bugs to fix
- Known expected outcomes

The trial also completes the full advisory loop on both fixtures to prove prepare/finish
still work correctly when CLAUDE.md is present (no regression from the snippet's presence).

## 5. Fixture Design

### Fixture A: Node with CLAUDE.md Snippet

| Property | Value |
|---|---|
| Path | `../akar-dogfood-v049-snippet-node-fixture` |
| Project kind | Node (package.json) |
| Test runner | `npm test` (node --test) |
| Test state before fix | 2 pass, 2 fail (multiply, multiply zero) |
| Bug | `multiply(a,b)` returns `a+b` instead of `a*b` |
| CLAUDE.md | Present — exact v0.48 snippet (14 lines) |
| Files | `package.json`, `src/calc.js`, `test/calc.test.js`, `README.md`, `CLAUDE.md`, `.gitignore` |

### Fixture B: Unknown No-Hint with CLAUDE.md Snippet

| Property | Value |
|---|---|
| Path | `../akar-dogfood-v049-snippet-unknown-nohint-fixture` |
| Project kind | Unknown (no markers) |
| Verification | Manual comparison (calc.txt vs expected.txt) |
| Bug | `multiply(2,4)=7` should be `multiply(2,4)=8` |
| CLAUDE.md | Present — exact v0.48 snippet (14 lines) |
| Files | `calc.txt`, `expected.txt`, `README.md`, `CLAUDE.md`, `.gitignore` |

### Fixture Selection Rationale

Two lanes were chosen, not all five:
- **Node:** Has automated tests (npm test) — the most common non-Rust dogfood lane. Tests give clear pass/fail signal.
- **Unknown No-Hint:** Has no automated verification — represents the hardest case for AI-facing delivery. If the snippet works here, it works anywhere.

Rust was excluded for efficiency (already proven in v0.47 cross-lane). Python was excluded (same automated-test pattern as Node). Unknown Makefile was excluded (make not available on this Windows machine). The two chosen lanes cover automated + manual verification, which is the full range.

## 6. Snippet Text Used

In both fixtures, CLAUDE.md was created manually with this exact text (the v0.48 design):

```markdown
## AKAR Session Guidance (managed by `akar init`)

Before starting any coding task, read `.akar/NEXT_RUN.md`. It contains:
- The current task objective and scope
- Budget limits (files and lines of code)
- Allowed and forbidden commands
- Required verification steps
- Stop conditions
- The loop governor's decision

After completing work, verify you stayed within the budget and followed the
stop conditions. The user will run `akar finish` to measure the diff.

<!-- AKAR section ends -->
```

This is verbatim from the v0.48.0 report section 19. No modifications.

## 7. Fixture A: Node Advisory Loop

### Setup

```
mkdir fixture/src fixture/test
Write package.json, src/calc.js, test/calc.test.js, README.md
git init && git add -A && git commit -m "test fixture baseline"
npm test  → 2 pass, 2 fail (multiply returns a+b)
akar init → OK (1 warn: templates dir absent, known)
Add .gitignore with .akar/
git add .gitignore && git commit -m "ignore AKAR local state"
Write CLAUDE.md with v0.48 snippet
git add CLAUDE.md && git commit -m "add CLAUDE.md with AKAR snippet"
```

### Prepare

```
akar prepare "fix the multiply bug: multiply(a,b) returns a+b instead of a*b"
```

| Field | Value |
|---|---|
| Project | akar-dogfood-v049-snippet-node-fixture (Node) |
| Baseline | snapshot at cf1cf265bb58 (3 files, 60 LOC) |
| Task type | Bugfix |
| Request mode | NORMAL |
| Check | PASS |
| Governor | READY |
| Verify | npm test (run manually) |

### Fix

Changed `src/calc.js` line 2: `return a + b;` → `return a * b;`

1 file, 1 added, 1 deleted (2 total changed LOC).

### Verify

`npm test` → 4/4 PASS (add, add negative, multiply, multiply zero all pass)

### Finish

```
akar finish
```

| Field | Value |
|---|---|
| Baseline | Bugfix at cf1cf265bb58 |
| Budget | 3 files, 60 LOC |
| Actual | 1 files, 2 total changed LOC |
| Budget verdict | PASS |
| Patches | none |
| Governor | RUN_POSTMORTEM |
| Health | 1 WARN (dirty tree, expected after fix) |

### Commit

```
git add src/calc.js && git commit -m "fix multiply to use multiplication"
```

Clean tree after commit.

## 8. Fixture B: Unknown No-Hint Advisory Loop

### Setup

```
mkdir fixture
Write calc.txt, expected.txt, README.md
git init && git add -A && git commit -m "test fixture baseline"
akar init → OK (1 warn: templates dir absent; Unknown WARN and no verification hints — expected)
Add .gitignore with .akar/
git add .gitignore && git commit -m "ignore AKAR local state"
Write CLAUDE.md with v0.48 snippet
git add CLAUDE.md && git commit -m "add CLAUDE.md with AKAR snippet"
```

### Prepare

```
akar prepare "fix the mismatch: multiply(2,4) should equal 8, not 7"
```

| Field | Value |
|---|---|
| Project | akar-dogfood-v049-snippet-unknown-nohint-fixture (Unknown) |
| Baseline | snapshot at f2cff5fef300 (3 files, 60 LOC) |
| Task type | Bugfix |
| Request mode | NORMAL |
| Check | PASS |
| Governor | READY |
| Verify | (no verification command discovered) |

### Fix

Changed `calc.txt` line 1: `multiply(2,4)=7` → `multiply(2,4)=8`

1 file, 1 added, 1 deleted (2 total changed LOC).

### Verify

Manual comparison: `calc.txt` content matches `expected.txt` content → PASS.

### Finish

```
akar finish
```

| Field | Value |
|---|---|
| Baseline | Bugfix at f2cff5fef300 |
| Budget | 3 files, 60 LOC |
| Actual | 1 files, 2 total changed LOC |
| Budget verdict | PASS |
| Patches | none |
| Governor | RUN_POSTMORTEM |
| Health | 3 WARNs (dirty tree, Unknown project kind, no verification hints — all expected) |

### Commit

```
git add calc.txt && git commit -m "fix calc.txt to match expected output"
```

Clean tree after commit.

## 9. Command Count

### Fixture A (Node)

| Category | Count | Commands |
|---|---|---|
| AKAR | 5 | init, prepare, finish ×1 each; 2 extra (init from prior session re-init, .gitignore commit) |
| Git | 7 | init, add ×3, commit ×3 |
| Project | 2 | npm test ×2 (before and after fix) |
| Manual file creation | 0 | (done by AI, not counted as user commands) |
| **Total** | **14** | |

Per-task AKAR commands: **2** (prepare, finish). Consistent with v0.47.

### Fixture B (Unknown No-Hint)

| Category | Count | Commands |
|---|---|---|
| AKAR | 5 | init, prepare, finish ×1 each; same extras as Fixture A |
| Git | 7 | init, add ×3, commit ×3 |
| Project | 0 | (manual comparison; no automated test) |
| Manual file creation | 0 | (done by AI) |
| **Total** | **12** | |

Per-task AKAR commands: **2** (prepare, finish). Consistent with v0.47.

## 10. Snippet Auto-Read Test: NOT PROVEN

### What the Test Requires

The critical question: in a fresh Claude Code session anchored in the fixture directory,
where CLAUDE.md contains the v0.48 snippet and `.akar/NEXT_RUN.md` exists, does Claude
read NEXT_RUN.md on its own initiative — without the user saying "read NEXT_RUN.md" or
"check AKAR" or any equivalent instruction?

### Why This Session Could Not Test It

This session was explicitly directed by the user's v0.49.0 specification, which states:

> "Read: docs/audits/AKAR_V0_47_... and docs/audits/AKAR_V0_48_..."

And later, after fixture setup:

> "Read: `.akar/NEXT_RUN.md` in both fixtures"

The instruction to read NEXT_RUN.md came from the user's spec, not from the CLAUDE.md
snippet. This is the manual-relay path — the user told Claude to read the file, which
is exactly the bottleneck the snippet is designed to eliminate.

**A true test requires:** start a completely fresh Claude Code session with `claude`
in the fixture directory, with no prior context about AKAR or this trial, and observe
whether Claude's first actions include reading `.akar/NEXT_RUN.md`.

### What We Know vs What We Don't

**Known (from documented Claude Code behavior):**
- CLAUDE.md is loaded into the system prompt at session start
- The snippet text will be in Claude's context from message one
- Claude can use the Read tool to read `.akar/NEXT_RUN.md`

**Unknown (requires fresh-session test):**
- Does Claude proactively read `.akar/NEXT_RUN.md` after seeing the CLAUDE.md instruction,
  or does it wait for the user to say something?
- Does Claude mention AKAR or NEXT_RUN.md in its first response?
- Does Claude follow the budget, stop conditions, and verification requirements from
  NEXT_RUN.md without the user reminding it?
- Does the behavior differ between Node (automated tests) and Unknown (no tests)?
- Does the behavior differ if the user's first message is "fix the multiply bug" vs
  a vague "hi"?

### Test Design for Fresh Session

A controlled test requires two Claude Code sessions, one per fixture. The exact procedure:

**Pre-session (do this before starting Claude Code):**
1. Verify CLAUDE.md exists and contains the exact v0.48 snippet
2. Verify `.akar/NEXT_RUN.md` exists (run `akar prepare` if needed)
3. Start Claude Code with `claude` in the fixture directory
4. Do NOT mention AKAR, NEXT_RUN.md, or "read any file" in any message
5. Do NOT use the word "budget," "governor," "stop condition," or "verification"

**Session start — first message:**
> "Fix the multiply bug in this project."

This is a minimal task instruction. It contains no mention of AKAR. If the snippet works,
Claude should read NEXT_RUN.md and respond with budget-aware planning.

**Expected behavior if snippet works:**
- Claude's first response mentions having read NEXT_RUN.md (or references AKAR guidance)
- Claude references the budget (3 files, 60 LOC)
- Claude knows the verification command (npm test for Node, manual compare for Unknown)
- Claude follows stop conditions

**Expected behavior if snippet fails:**
- Claude proceeds without mentioning NEXT_RUN.md or AKAR
- Claude may or may not discover NEXT_RUN.md through filesystem exploration
- Claude does not reference budget limits unless the user brings them up
- Claude may fix the bug correctly but without AKAR discipline

**Success criterion:** Claude reads `.akar/NEXT_RUN.md` within its first 2-3 tool calls
without the user asking it to. Claude's response references AKAR-specific guidance
(budget, governor decision, allowed commands, stop conditions) that it could only get
from NEXT_RUN.md.

**Failure criterion:** Claude does not read `.akar/NEXT_RUN.md` without explicit user
instruction. Or Claude reads it only because it's exploring the filesystem and discovers
it, not because the CLAUDE.md snippet directed it to.

**Honest outcome documentation:** After the session, record:
1. Did Claude read `.akar/NEXT_RUN.md`? (check tool call history)
2. At what point in the session? (first tool call, after user message, after discovery?)
3. Did Claude mention the snippet or CLAUDE.md in its reasoning?
4. Did Claude follow AKAR guidance (budget, verification, stop conditions)?
5. How many user messages before Claude engaged with AKAR context?
6. Any surprises or edge cases?

### Known Limitation: This Session Is Not a Substitute

Even though this conversation began with "You are continuing AKAR from a fresh Claude
Code session," the user's message includes the full v0.49.0 specification with explicit
instructions to read files. This is sufficient for the advisory loop (prepare → fix →
verify → finish) but insufficient for the auto-read test. The session is not "fresh"
with respect to AKAR awareness — the user's spec IS the manual relay.

## 11. Safety Boundary Verification

| Boundary | Fixture A (Node) | Fixture B (Unknown) |
|---|---|---|
| AKAR executed project code | No | No |
| AKAR mutated git state | No | No |
| AKAR modified Claude settings | No | No |
| AKAR called model APIs | No | No |
| AKAR auto-edited CLAUDE.md | No (manually written) | No (manually written) |
| AKAR auto-edited .gitignore | No (manually written) | No (manually written) |
| prepare only wrote to .akar/ | Yes | Yes |
| finish only wrote to .akar/ | Yes (learning patches: none) | Yes (learning patches: none) |

All safety boundaries hold. The CLAUDE.md snippet was manually written — no AKAR command
touched CLAUDE.md. This is consistent with v0.48's "no implementation" rule.

## 12. CLAUDE.md Snippet Structural Verification

| Property | Fixture A | Fixture B |
|---|---|---|
| CLAUDE.md exists | Yes | Yes |
| Snippet header matches v0.48 design | Yes | Yes |
| Snippet footer (HTML comment delimiter) present | Yes | Yes |
| Snippet is 14 lines | Yes | Yes |
| Snippet points to `.akar/NEXT_RUN.md` | Yes | Yes |
| Snippet is clearly delimited from other content | Yes (only content in the file) | Yes (only content in the file) |
| NEXT_RUN.md exists and is valid | Yes (passed request --check) | Yes (passed request --check) |
| Snippet does not inline task context | Yes (pointer only) | Yes (pointer only) |
| Snippet does not include AKAR version | Yes | Yes |

The snippet in both fixtures is structurally identical to the v0.48 design. The pointer
target (`.akar/NEXT_RUN.md`) exists and is valid in both fixtures.

## 13. Fresh-Session Test Instructions

### Test A: Node Fixture

**Fixture:** `C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-node-fixture`

Pre-conditions:
- CLAUDE.md present with v0.48 snippet
- `.akar/NEXT_RUN.md` present (task: "fix the multiply bug...")
- Bug is RE-INTRODUCED: `src/calc.js` line 2 uses `a + b` (not `a * b`)
- Tests pass after fix (4/4)
- Git tree clean, no uncommitted changes except the re-introduced bug

**Resetting the fixture for fresh test:**
```
cd C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-node-fixture
# Re-introduce the bug
(edit src/calc.js: change return a * b back to return a + b)
git add src/calc.js && git commit -m "re-introduce bug for fresh-session test"
# Run prepare for the fresh session
akar prepare "fix the multiply bug: multiply(a,b) returns a+b instead of a*b"
# Verify NEXT_RUN.md is fresh
akar request --check
```

**Fresh session instructions:**
1. Open a new terminal
2. `cd C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-node-fixture`
3. `claude`
4. First message: "Fix the multiply bug in this project."
5. Do NOT mention AKAR, NEXT_RUN.md, budget, governor, or verification
6. Observe whether Claude reads `.akar/NEXT_RUN.md` on its own

**Expected outcome if snippet works:**
- Claude reads `.akar/NEXT_RUN.md` within first 2-3 tool calls
- Claude mentions the 3-file/60-LOC budget
- Claude knows to run `npm test` for verification
- Claude fixes `a+b` → `a*b`
- Claude runs `npm test` → 4/4 PASS
- Claude reports staying within budget

### Test B: Unknown No-Hint Fixture

**Fixture:** `C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-unknown-nohint-fixture`

Pre-conditions:
- CLAUDE.md present with v0.48 snippet
- `.akar/NEXT_RUN.md` present (task: "fix the mismatch...")
- Bug is RE-INTRODUCED: `calc.txt` contains `multiply(2,4)=7`
- Expected output is `multiply(2,4)=8`
- Git tree clean, no uncommitted changes except the re-introduced bug

**Resetting the fixture for fresh test:**
```
cd C:\Users\nbzkr\Coding\akar-dogfood-v049-snippet-unknown-nohint-fixture
# Re-introduce the bug
(edit calc.txt: change multiply(2,4)=8 back to multiply(2,4)=7)
git add calc.txt && git commit -m "re-introduce bug for fresh-session test"
# Run prepare for the fresh session
akar prepare "fix the mismatch: multiply(2,4) should equal 8, not 7"
# Verify NEXT_RUN.md is fresh
akar request --check
```

**Fresh session instructions:** Same as Test A, with first message: "Fix the output mismatch in this project."

**Expected outcome if snippet works:**
- Claude reads `.akar/NEXT_RUN.md` within first 2-3 tool calls
- Claude knows there's no automated verification (Unknown project)
- Claude fixes `multiply(2,4)=7` → `multiply(2,4)=8`
- Claude performs or suggests manual comparison against expected.txt
- Claude reports staying within budget

### How to Record Results

After each fresh session, fill in:

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
| Tests/verification passed? | YES / NO |
| Any surprises? | (free text) |

## 14. What This Trial Proved

| Claim | Status | Evidence |
|---|---|---|
| Prepare/finish works when CLAUDE.md is present | **PROVEN** | Both fixtures completed full advisory loop: prepare PASS, fix, verify PASS, finish PASS |
| The snippet text is syntactically valid markdown | **PROVEN** | Both fixtures' CLAUDE.md files are valid markdown |
| NEXT_RUN.md is correctly generated when CLAUDE.md exists | **PROVEN** | Both fixtures: request --check PASS, governor READY, all 11 sections present |
| Project-kind detection is unaffected by CLAUDE.md presence | **PROVEN** | Node detected as Node, Unknown detected as Unknown — no interference from CLAUDE.md |
| Safety boundaries hold when CLAUDE.md is present | **PROVEN** | No project code execution, no git mutation, no settings modification, no model API calls |
| CLAUDE.md snippet causes Claude to auto-read NEXT_RUN.md | **NOT PROVEN** | This session was explicitly told to read NEXT_RUN.md by the user's spec — manual relay, not auto-delivery |
| The snippet works in a truly fresh session | **NOT PROVEN** | Requires a separate fresh Claude Code session with no prior AKAR context |

## 15. Additional Observations

### Observed: `akar prepare` Output Does Not Mention CLAUDE.md

The prepare output says:

> "next: Ask the AI to read .akar/NEXT_RUN.md, then do the task."

This is the pre-v0.48 manual-relay message. When CLAUDE.md is present with the snippet,
this message should ideally say:

> "next: CLAUDE.md configured — Claude will read .akar/NEXT_RUN.md automatically. Start your AI session."

This is a cosmetic issue — the prepare command doesn't check for CLAUDE.md presence.
The v0.48 design (section 22, "Silent Failure Mitigation") already specified this
improvement as part of the future implementation scope. This trial confirms the need.

### Observed: CLAUDE.md Snippet Does Not Interfere with AKAR Operations

AKAR's prepare, finish, doctor, status, request --check, governor, and learn --list
all work identically whether CLAUDE.md is present or not. The snippet is a markdown
file that AKAR ignores. This is correct behavior — AKAR should not parse or depend
on CLAUDE.md content until snippet management is implemented.

### Observed: CLAUDE.md Is Committed to Git

Both fixtures have CLAUDE.md committed to git. The snippet travels with the project.
This is the intended behavior from the v0.48 design — the snippet is version-controlled
and shared with teammates who clone the repo. The user (or AKAR, in a future release)
decides whether to commit CLAUDE.md.

### Observed: No CLAUDE.md in the AKAR Repo Itself

The AKAR repo has no CLAUDE.md. This is notable — AKAR's own project doesn't dogfood
the CLAUDE.md snippet. If the snippet is proven in fresh-session tests, the AKAR repo
should add it as a self-dogfood demonstration.

## 16. Recommendations

### Immediate (After This Report)

1. **Run the fresh-session tests** described in section 13. These require a separate
   Claude Code session, not this one. The fixtures are prepared and ready.
2. **If the snippet works:** v0.50.0 can move forward with implementation (CLAUDE.md
   snippet management in `akar init`, doctor/status delivery checks, prepare output
   update).
3. **If the snippet fails:** Reopen mechanism evaluation. The hook-mediated path
   (mechanism D) or a completely different approach may be needed. Document exactly
   what failed and why.

### Future Release (After Fresh-Session Confirmation)

4. **Add CLAUDE.md to the AKAR repo** as a self-dogfood demonstration once the snippet
   is proven.
5. **Update `akar prepare` output** to detect CLAUDE.md presence and report delivery
   state (as designed in v0.48 section 22).
6. **Implement snippet management** in `akar init` (v0.48 section 23 implementation
   scope).

### If the Snippet Fails

7. **Test hook output visibility** — the v0.44 and v0.48 reports both postponed
   mechanism D (hook-mediated reminder) pending this test. If CLAUDE.md snippet fails,
   hook-mediated injection may be the next-best mechanism.
8. **Investigate why Claude ignores the instruction** — is the instruction too weak?
   Does Claude interpret "before starting any coding task" as conditional on the user
   starting a task? Would stronger wording help, or is the problem fundamental to
   CLAUDE.md instructions?

## 17. Honest Conclusion

The v0.48 CLAUDE.md snippet design is structurally implemented in two external fixtures.
The advisory loop (prepare → fix → verify → finish) completed cleanly on both. The
snippet doesn't interfere with AKAR operations. Safety boundaries hold.

But the critical claim — that the snippet causes Claude to auto-read NEXT_RUN.md in a
fresh session without user prompting — remains unproven. This session could not test it
because the user's v0.49.0 specification explicitly instructed the reading of files,
which is the manual-relay path the snippet is designed to replace.

The fixtures are instrumented and ready. The test instructions are clear. The outcome
will determine whether v0.50.0 implements snippet management or reopens the mechanism
question entirely.

AKAR's honesty principle demands: do not claim what is not proven. The snippet design
is sound on paper. Its behavioral effectiveness in a real Claude Code session is the
next piece of evidence needed.
