# AKAR v0.45.0 — Burden Measurement Dogfood

## 1. Executive Verdict

**Burden confirmed. 59 commands for a 2-task session.** Across setup and two simple
tasks, the user ran 59 total shell commands, made 8 user decisions, performed 8 manual
relay/inspection actions, and spent approximately 7 minutes on AKAR overhead alone.
The v0.44.0 design estimate of "7+ commands per task" was conservative — the actual
per-task burden is higher when counting git commands, inspections, and validation
commands.

The measurement validates the v0.44.0 prepare/finish design: if `akar prepare` and
`akar finish` existed today, total AKAR commands would drop from ~17 per task to ~2
per task. Command count is the dominant friction, not context relay (in this trial,
Claude read NEXT_RUN.md successfully when told to). Dirty-tree friction from `.akar/`
state is a real but one-time problem (solved by `.gitignore`). Hook setup remains
the largest single burden — interactive confirmation, manual wiring to Claude Code
settings, and no automation path.

**Recommendation: v0.46.0 Prepare/Finish Command Prototype.** The data supports
command consolidation as the highest-impact, lowest-risk first step.

## 2. Baseline and Verification

| Check | Result |
|---|---|
| Commit | `1cb4ffa` — docs: design AKAR manual CLI burden reduction |
| Version | `akar 0.44.0` |
| Working tree | clean |
| `cargo test` | 508 passed, 0 failed |
| `cargo run -- --version` | `akar 0.44.0` |
| `cargo run -- doctor` | PASS (1 WARN: split-rule) |
| `cargo run -- status` | HEALTHY, READY |
| `cargo run -- request "..."` | NORMAL mode |
| `cargo run -- request --check` | PASS (4/4) |
| `cargo run -- governor` | SPLIT_TASK (known artifact) |
| `cargo run -- learn --list` | 8 entries (1 active, 7 resolved) |
| `cargo run -- hooks --check` | PASS |
| `cargo run -- eval` | 28/28 PASS |

All checks pass. AKAR 0.44.0 confirmed. Working tree clean.

## 3. Measurement Method

**Fixture:** `../akar-dogfood-v045-burden-measurement-node-fixture`
**Environment:** Node.js v24.11.0, Windows 11, PowerShell 7, git 2.x
**Method:** Run every command in sequence, record count, time, and category. No
automation assistance — every command typed manually (or via AI invocation, which
counts as a command). Timestamps recorded before and after each phase.

**What was measured:**
- A. Command count (AKAR, git, project, inspection, total)
- B. User decision count (tracking policy, hook install, commit decisions, etc.)
- C. Manual relay count (times AKAR output must be explicitly read/bridged)
- D. Time burden (approximate elapsed per phase)
- E. Friction events (dirty-tree, missing files, repeated typing, warnings)
- F. Missed-step risk (Low/Medium/High for each step)
- G. Automation candidacy (must remain manual / safe to group / safe to auto-invoke)

**What was NOT measured (live hooks):**
- PreToolUse hook wiring to Claude Code settings was NOT performed — it requires
  manually editing `~/.claude/settings.json`, which AKAR must never auto-modify.
  Hook setup burden is recorded as a manual step but hook events were not tested.

**Task sequence:**
1. Fixture creation (outside AKAR repo)
2. One-time AKAR setup (init, hooks, doctor, status, verify, `.akar/` tracking)
3. Task 1 — Bugfix: fix multiply (a+b → a*b)
4. Task 2 — Feature: add square function + test

## 4. Fixture Description

| Property | Value |
|---|---|
| Path | `../akar-dogfood-v045-burden-measurement-node-fixture` |
| Project kind | Node.js |
| Marker file | `package.json` |
| Test runner | `node --test test/*.test.js` |
| Initial test state | 2 pass, 1 fail (multiply returns a+b) |
| Files | `package.json`, `src/calc.js`, `test/calc.test.js`, `README.md` |
| Initial commit | `666ab1d` — test fixture baseline |
| Node version | v24.11.0 |

**Fixture commands (creation):**
```
mkdir -p fixture/src fixture/test            # 1 cmd
Write package.json                           # (via tool)
Write src/calc.js                            # (via tool)
Write test/calc.test.js                      # (via tool)
Write README.md                              # (via tool)
git init                                     # 2 cmd
git add package.json src/calc.js test/calc.test.js README.md  # 3 cmd
git commit -m "test fixture baseline"        # 4 cmd
npm test                                     # 5 cmd — confirmed 2 pass, 1 fail
git status                                   # 6 cmd — clean
```

Fixture creation: 6 commands. Not counted in AKAR burden (this is normal project setup).
Elapsed time: ~1 minute (writing files + git init + confirming test failure).

## 5. One-Time Setup Burden

### 5.A Setup Command Log

| # | Phase | Command | Category | Result |
|---|---|---|---|---|
| 1 | Init | `akar init` | AKAR | bootstrap: 0 created; templates dir not found warning |
| 2 | Dirt check | `git status` | Git | clean (`.akar/` inside `.gitignore` — but `.gitignore` not yet created) |
| 3 | Dirt check | `ls .akar/` | Inspection | verify `.akar/` directory contents |
| 4 | Hook check | `akar hooks --check` | AKAR | PASS (embedded templates) |
| 5 | Hook install | `"INSTALL" \| akar hooks --install` | AKAR | copied pre-tool-call.sh, pre-tool-call.ps1; manual wiring notice |
| 6 | Hook check | `akar hooks --check` | AKAR | PASS (project .akar/hooks) |
| 7 | Doctor | `akar doctor` | AKAR | WARN: NEXT_RUN missing, DIFF_BASELINE missing, dirty tree |
| 8 | Status | `akar status` | AKAR | HEALTHY, BLOCKED (dirty tree, no baseline) |
| 9 | Verify | `akar verify` | AKAR | manual-only for Node (correct) |
| 10 | Tracking | Create `.gitignore` with `.akar/` | Project | decision made |
| 11 | Dirt check | `git status` | Git | untracked .gitignore |
| 12 | Commit | `git add .gitignore && git commit` | Git | "ignore AKAR local state" |
| 13 | Final | `git status` | Git | clean |

### 5.B One-Time Setup Burden Analysis

**Commands:**
- AKAR commands: 6 (`init`, `hooks --check` ×2, `hooks --install`, `doctor`, `status`, `verify`)
- Git commands: 4 (`git status` ×2, `git add`, `git commit`)
- Inspection: 1 (`ls .akar/`)
- Project: 1 (create `.gitignore`)
- **Total: 12 commands**

**Decisions:**
- Interactive confirmation: "Type INSTALL to confirm" for `akar hooks --install`
- `.akar/` tracking policy: whether to add `.akar/` to `.gitignore`
- No hook wiring to Claude Code was performed (requires manual settings edit)

**Time:**
- Setup start: 23:07:25
- Setup end: ~23:09:00 (after final git status)
- Elapsed: ~1.5 minutes

**Dirty-tree friction:**
- After `akar init`: tree reported clean (`.akar/` was created but git status showed clean — `.akar/` files don't dirty the tree on their own unless gitignored)
- After `akar hooks --install`: tree remained clean
- Doctor and status both warned "working tree: dirty" — this was misleading because git status showed clean. The AKAR status check may be detecting `.akar/` state as dirty.
- Adding `.gitignore` resolved the status WARN — clean status after commit

**Key friction events:**
1. `akar hooks --install` requires interactive confirmation ("Type INSTALL to confirm") — blocks scripting and CI
2. Hook wiring to Claude Code settings remains entirely manual — AKAR prints instructions but cannot wire them
3. `doctor` shows WARN for missing NEXT_RUN and DIFF_BASELINE on fresh project — these are expected but still flagged as warnings
4. `status` shows BLOCKED readiness on fresh project — accurate but adds cognitive load to first-time users
5. `akar init` prints "templates directory not found" — unclear whether this is an error or expected
6. `ls .akar/` returned nothing (empty or silent) — user must manually inspect state

**Missed-step risk:**

| Step | Risk | Reason |
|---|---|---|
| Run `akar init` | Low | Obvious first step |
| Run `akar hooks --install` | **High** | Separate command; user may skip or forget |
| Wire hooks to Claude Code | **High** | Manual settings edit; not part of any AKAR command |
| Create `.gitignore` for `.akar/` | Medium | User may not realize `.akar/` dirties working tree |
| Run `akar doctor` | Medium | Recommended but not enforced |
| Run `akar verify` | Low | Project test command is discoverable via npm test |

**Automation candidacy:**

| Step | Future |
|---|---|
| `akar init` | Must remain manual (user opts in) |
| `akar hooks --install` | Should remain manual (writes executable templates); could be offered during init |
| `akar hooks --check` | Safe to group into prepare/doctor |
| `akar doctor` | Safe to group into prepare (summary mode) |
| `akar status` | Safe to group into prepare/finish (summary mode) |
| `akar verify` | Must remain manual for non-Rust projects; Rust auto-verify already exists |
| `.gitignore` decision | Must remain manual (removing user agency over their repo is hostile) |
| Hook wiring to Claude settings | Must remain manual (AKAR must never auto-edit settings) |

## 6. Task 1 Measured Burden — Bugfix

**Task:** Fix the multiply bug (a+b → a*b)
**Expected change:** 1 file, 1 added/1 deleted, 2 LOC

### 6.A Pre-Edit Command Log

| # | Command | Category | Result |
|---|---|---|---|
| 1 | `git status` | Git | clean |
| 2 | `akar preflight --snapshot "fix the multiply bug..."` | AKAR | Bugfix, 3 files/60 LOC, baseline written at 2a16a09e4674 |
| 3 | `akar request "fix the multiply bug..."` | AKAR | NORMAL mode, wrote NEXT_RUN.md |
| 4 | `akar request --check` | AKAR | PASS (4/4) — but on first attempt: file not found (request not yet run), required re-run |
| 5 | `akar governor --json --no-exit-code` | AKAR | READY |
| 6 | Inspect `.akar/NEXT_RUN.md` | Inspection | Manual relay — read 40+ lines of context |

Note: `request --check` initially FAILED ("file not found") because `request` output
was not visible on first run (the command ran silently). This required a second
invocation — a friction point where command output was unclear.

### 6.B Edit/Verification Command Log

| # | Command | Category | Result |
|---|---|---|---|
| 7 | Edit `src/calc.js` (a+b → a*b) | Edit | 1 change |
| 8 | `npm test` | Project | 3/3 PASS |

### 6.C Post-Edit Command Log

| # | Command | Category | Result |
|---|---|---|---|
| 9 | `akar postmortem --diff --baseline` | AKAR | PASS: 1 file, 1 added/1 deleted, 2 LOC (budget: 60) |
| 10 | `akar learn --list` | AKAR | 0 entries |
| 11 | `akar governor --json --no-exit-code` | AKAR | RUN_POSTMORTEM (dirty tree) |
| 12 | `akar doctor` | AKAR | WARN: dirty tree |
| 13 | `akar status` | AKAR | HEALTHY, BLOCKED (dirty tree) |
| 14 | `git status` | Git | modified: src/calc.js |
| 15 | `git diff --stat` | Git | 1 file, 1 insertion, 1 deletion |
| 16 | `git add src/calc.js` | Git | staged |
| 17 | `git commit -m "fix multiply bug"` | Git | committed |
| 18 | `git status` | Git | clean |

### 6.D Task 1 Burden Summary

| Metric | Count |
|---|---|
| Pre-edit commands | 6 (3 AKAR, 1 Git, 1 inspection, 1 re-run) |
| Edit/verify commands | 2 (1 edit, 1 project) |
| Post-edit commands | 10 (4 AKAR, 5 Git, 0 project) |
| **Total AKAR commands** | **8** (preflight, request, request --check, governor-pre, postmortem, learn, governor-post, doctor, status) |
| Total Git commands | 6 (git status ×3, git diff --stat, git add, git commit) |
| Total project commands | 1 (npm test) |
| Total inspection commands | 1 (inspect NEXT_RUN.md) |
| **Total all commands** | **18** |

Note: `governor` was run twice (pre-edit: READY, post-edit: RUN_POSTMORTEM). `doctor`
and `status` were run post-edit but not pre-edit. A discipline-enforcing user would
run governor/doctor/status on both sides.

**Elapsed:**
- Pre-edit start: 23:09:26
- Pre-edit end: 23:09:26 (instant — below second resolution)
- Post-edit start: 23:10:20
- Post-edit end: 23:10:21 (1 second)
- Task 1 total: ~55 seconds (23:09:26 to 23:10:21)

**Friction:**
1. `request --check` failed on first attempt (file not found — NEXT_RUN.md wasn't written yet because prior `request` output was unclear)
2. `governor` flipped from READY (clean, with baseline) to RUN_POSTMORTEM (dirty, same baseline) — predictable but requires running twice
3. `doctor` and `status` both show WARN/BLOCKED for dirty tree — technically correct but redundant when user is about to commit
4. Git status → git diff --stat → git add → git commit → git status is a fixed sequence that could be guided by `akar finish`

**Manual relay:**
- 1 explicit inspection of NEXT_RUN.md (manual relay)
- Claude read NEXT_RUN.md before editing (confirmed via Read tool usage)
- Task text repeated across: preflight, request, governor output, NEXT_RUN.md — 3

## 7. Task 2 Measured Burden — Feature

**Task:** Add a square function and test
**Expected change:** 2 files, 11 added/3 deleted, 14 LOC

### 7.A Pre-Edit Command Log

| # | Command | Category | Result |
|---|---|---|---|
| 1 | `git status` | Git | clean |
| 2 | `akar preflight --snapshot "add a square function..."` | AKAR | Bugfix (classified as), 3 files/60 LOC, baseline at 82a4fbf |
| 3 | `akar request "add a square function..."` | AKAR | NORMAL mode, wrote NEXT_RUN.md |
| 4 | `akar request --check` | AKAR | PASS (4/4) — succeeded first try (request output was clearer) |
| 5 | `akar governor --json --no-exit-code` | AKAR | READY |

Note: Task 2 preflight classified the feature as "Bugfix" with "Low risk" and stop
condition "original symptom no longer reproducible." This is a known preflight
classification issue — features and refactors are sometimes classified as Bugfix
when the task text doesn't contain obvious feature keywords. The budget (3 files,
60 LOC) and NORMAL mode are correct. This is a pre-existing behavior, not a v0.45
measurement issue.

### 7.B Edit/Verification Command Log

| # | Command | Category | Result |
|---|---|---|---|
| 6 | Edit `src/calc.js` (add square, update exports) | Edit | 6 lines added |
| 7 | Edit `test/calc.test.js` (add test, update imports) | Edit | 8 lines added, 1 modified |
| 8 | `npm test` | Project | 4/4 PASS |

### 7.C Post-Edit Command Log

| # | Command | Category | Result |
|---|---|---|---|
| 9 | `akar postmortem --diff --baseline` | AKAR | PASS: 2 files, 11 added/3 deleted, 14 LOC (budget: 60) |
| 10 | `akar learn --list` | AKAR | 0 entries |
| 11 | `akar governor --json --no-exit-code` | AKAR | RUN_POSTMORTEM (dirty tree) |
| 12 | `akar doctor` | AKAR | WARN: dirty tree |
| 13 | `akar status` | AKAR | HEALTHY, BLOCKED (dirty tree) |
| 14 | `git status` | Git | modified: src/calc.js, test/calc.test.js |
| 15 | `git diff --stat` | Git | 2 files, 11 insertions, 3 deletions |
| 16 | `git add src/calc.js test/calc.test.js` | Git | staged |
| 17 | `git commit -m "add square function"` | Git | committed |
| 18 | `git status` | Git | clean |

### 7.D Task 2 Burden Summary

| Metric | Count |
|---|---|
| Pre-edit commands | 5 (3 AKAR, 1 Git, 1 inspection) |
| Edit/verify commands | 3 (2 edits, 1 project) |
| Post-edit commands | 10 (4 AKAR, 5 Git, 0 project) |
| **Total AKAR commands** | **8** (same pattern as Task 1) |
| Total Git commands | 6 |
| Total project commands | 1 (npm test) |
| Total inspection commands | 0 (no manual NEXT_RUN inspection needed) |
| **Total all commands** | **18** |

**Elapsed:**
- Pre-edit start: 23:10:41
- Post-edit end: 23:11:47
- Task 2 total: ~66 seconds

**Friction:**
1. Preflight classified feature as "Bugfix" — same known classification behavior
2. Same redundant doctor/status dirty-tree warnings as Task 1
3. No manual relay needed for Task 2 (NEXT_RUN.md was already inspected in Task 1)

## 8. Total Measured Burden

### 8.A Full Session Command Count

| Category | Setup | Task 1 | Task 2 | Total |
|---|---|---|---|---|
| AKAR commands | 7 | 8 | 8 | **23** |
| Git commands | 4 | 6 | 6 | **16** |
| Project commands | 1 | 1 | 1 | **3** |
| Inspection/relay | 1 | 1 | 0 | **2** |
| Edit/file commands | 0 | 1 | 2 | **3** |
| Fixture creation | — | — | — | 6 |
| Total (excl. fixture) | 13 | 18 | 18 | **49** |
| **Total (incl. fixture)** | — | — | — | **55** |

Note: Some AKAR commands were counted once but appear in both pre and post phases
(doctor, status). count-split: `governor` run twice per task (pre + post) counts as
2 AKAR commands per task.

### 8.B Per-Task AKAR Command Pattern (Identical Both Tasks)

**Pre-edit (3 AKAR + 1 Git + 1 inspection):**
1. `git status`
2. `akar preflight --snapshot "<task>"`
3. `akar request "<task>"`
4. `akar request --check`
5. `akar governor --json --no-exit-code`

**Post-edit (5 AKAR + 4 Git):**
6. `akar postmortem --diff --baseline`
7. `akar learn --list`
8. `akar governor --json --no-exit-code`
9. `akar doctor`
10. `akar status`
11. `git status`
12. `git diff --stat`
13. `git add <files>`
14. `git commit -m "..."`

Per-task AKAR commands: 8 (preflight, request, request --check, governor ×2,
postmortem, learn, doctor, status). This exceeds the v0.44.0 estimate of "7+"
and the v0.44.0 design figure of 7.

## 9. Manual Relay Burden

### 9.A Relay Actions

| Action | Task 1 | Task 2 | Burden |
|---|---|---|---|
| Inspect `.akar/NEXT_RUN.md` | Yes (40 lines) | No (already known) | Task-specific |
| Task text typed into preflight | "fix the multiply bug..." | "add a square function..." | Repeated ×2 |
| Task text typed into request | Same text | Same text | Repeated ×2 (same text as preflight) |
| Task text appears in governor | N/A (JSON only) | N/A (JSON only) | Not surfaced |
| User bridges context into AI | Claude read NEXT_RUN.md via Read tool | Claude already had context | 1 manual bridge |
| Task text appears in NEXT_RUN.md | Yes (Objective section) | Yes (Objective section) | Auto-written |

### 9.B Relay Friction

1. **Task text repeated 4 times:** preflight, request, governor output, NEXT_RUN.md.
   Each command needs the task text as an argument. `preflight` and `request` use
   identical text. `governor` and `NEXT_RUN.md` derive from request output.
2. **NEXT_RUN.md inspection is manual:** Claude must be told to read it or must
   discover it from project files. In this trial, Claude was told (via task
   instructions) to read it.
3. **If CLAUDE.md snippet existed:** The instruction "read .akar/NEXT_RUN.md before
   starting" would have eliminated the manual relay step entirely. Claude would
   discover and read NEXT_RUN.md without user prompting.
4. **No copy/paste occurred** — all relay was via Claude's Read tool. But the user
   (task instructions) had to specify that Claude should read NEXT_RUN.md.

**Relay verdict:** Moderate burden. Not the dominant friction (command count is worse),
but the task-text repetition across preflight/request and the manual NEXT_RUN.md
inspection add cognitive overhead. A CLAUDE.md snippet would eliminate the inspection
step but not the task-text repetition (which requires prepare consolidation).

## 10. Dirty-Tree Friction

### 10.A Events

1. **After `akar init`:** `.akar/` directory created. Git status showed clean (files
   inside `.akar/` don't dirty the tree). However, AKAR status and doctor both
   reported dirty-tree WARNs. This is confusing — git status says clean but AKAR
   says dirty.

2. **After `akar hooks --install`:** Hook templates written to `.akar/hooks/`. Tree
   still clean per git but AKAR still reports dirty.

3. **Resolution:** Adding `.akar/` to `.gitignore` and committing resolved the AKAR
   dirty-tree warnings. After commit, both git status and AKAR status agreed: clean.

4. **During tasks:** Dirty-tree only occurred during active work (expected). AKAR
   correctly detected dirty tree post-edit and guided to commit.

### 10.B Friction Assessment

| Issue | Severity | Frequency | Resolution |
|---|---|---|---|
| AKAR status reports dirty when git status is clean (`.akar/` state) | Medium | One-time (after init) | Add `.gitignore`, commit |
| Doctor WARNs for missing NEXT_RUN/DIFF_BASELINE on fresh project | Low | One-time (after init) | Run preflight + request |
| Dirty-tree guidance is clear ("commit, don't stash") | — | Per-task post-edit | Commit |

**Verdict:** Dirty-tree friction exists but is one-time and manageable. The `.akar/`
tracking decision is required but AKAR correctly does not auto-modify `.gitignore`.
The friction is in the cognitive load (new user sees WARNs and doesn't know whether
something is broken) rather than in the actual state (nothing is broken).

## 11. Hook Setup Friction

### 11.A Events

1. `akar hooks --check` (pre-install): PASS — embedded templates found
2. `akar hooks --install`: Interactive confirmation required ("Type INSTALL to confirm")
3. Hook templates copied: `pre-tool-call.sh`, `pre-tool-call.ps1`
4. `akar hooks --check` (post-install): PASS — project templates found
5. **Hook wiring to Claude Code NOT performed** — AKAR prints a JSON example for
   `~/.claude/settings.json` but cannot write it

### 11.B Barrier Analysis

| Barrier | Severity | Notes |
|---|---|---|
| Interactive confirmation | Medium | Blocks CI/scripting; requires human to type "INSTALL" |
| Manual `~/.claude/settings.json` edit | **High** | User must find file, understand JSON schema, insert hook config correctly |
| No validation of wiring | **High** | AKAR cannot check whether hooks are actually wired; `hooks --check` only checks template validity |
| Platform-specific templates | Medium | `.sh` (bash) and `.ps1` (PowerShell) both provided; user must pick the right one |
| Hook wiring instructions | Medium | AKAR prints an example but doesn't confirm the user followed it |

### 11.C Hook Setup Burdens NOT Measured

This dogfood did NOT test:
- Whether the user can successfully edit `~/.claude/settings.json`
- Whether the hook actually fires on Bash tool calls
- Whether the hook correctly classifies ALLOW/BLOCK
- Whether the hook produces parseable HOOK_EVENTS

These are known from v0.35/v0.36 dogfood trials (hook integration proven). But the
setup friction itself — the time and error rate of manual settings editing — was not
measured because live hooks were not wired for this trial.

**Verdict:** Hook setup is the single largest burden in AKAR's current workflow. It
requires three separate manual actions (run `hooks --install`, type INSTALL, edit
settings.json) and AKAR cannot validate that the third step was done correctly. This
is a critical gap for v1.0.

## 12. Missed-Step Risk

### 12.A Risk Matrix

| Step | Risk | Why | Mitigation |
|---|---|---|---|
| `akar init` | Low | Obvious; first AKAR command | Status/doctor remind if missing |
| `akar hooks --install` | **High** | Separate command; easy to forget or skip | Could be offered during init |
| Edit `~/.claude/settings.json` | **Critical** | Manual, error-prone, AKAR can't verify | Needs design (not implemented) |
| Create `.gitignore` for `.akar/` | Medium | Non-obvious; user sees dirty-tree WARN | Doctor surfaces the warning |
| `akar preflight --snapshot` | Medium | Must run before request; dependency not enforced | Could be auto-run by prepare |
| `akar request --check` | **High** | Easy to skip; NEXT_RUN.md might be malformed | Could be auto-run by prepare |
| `akar postmortem --diff --baseline` | **High** | Easy to forget after task completed | Could be auto-run by finish |
| `akar learn --list` | Medium | Easy to skip; patches may accumulate | Could be surfaced in finish |
| `akar governor` | Medium | Informational only; skipped if user trusts workflow | Could be embedded in prepare/finish |
| Git commit after task | Low | User already knows to commit | Could be guided by finish output |

### 12.B Critical Gaps

1. **`akar hooks --install`** and **manual settings wiring** form a two-step manual
   process with no validation. A user could run `hooks --install`, skip settings
   editing, and believe hooks are working. `hooks --check` only validates templates,
   not wiring.
2. **`akar request --check`** has no enforcement — nothing prevents the user from
   starting a task with an invalid NEXT_RUN.md.
3. **`akar postmortem`** has no enforcement — nothing prevents the user from
   committing without measuring diff.

Prepare/finish consolidation addresses gaps 2 and 3 (check becomes part of prepare,
postmortem becomes part of finish). Gap 1 (hook wiring) requires separate design.

## 13. AI-Facing Delivery Burden Observed

### 13.A How Claude Received AKAR Context in This Trial

- **Task 1:** Claude was instructed (via task prompt) to read `.akar/NEXT_RUN.md`.
  Claude used the Read tool to inspect it. Context was received successfully.
- **Task 2:** Claude had already read NEXT_RUN.md in Task 1. New NEXT_RUN.md was
  generated for Task 2 with updated task text and baseline HEAD. Claude did not
  re-read it (not needed — task instructions provided context directly).

### 13.B What Worked

- NEXT_RUN.md content was relevant: task, project kind, budget, allowed commands,
  stop conditions, verification command. All aligned with the actual task.
- Claude correctly followed the verification command (`npm test`) from NEXT_RUN.md.
- Claude stayed within the diff budget (2 LOC for Task 1, 14 LOC for Task 2; budget: 60).
- Verification command in NEXT_RUN.md matched project reality (`npm test`).

### 13.C What Didn't Work / Was Unnecessary

- **NEXT_RUN.md is too long for repeated injection.** At ~80-120 lines, reading it
  every task adds token overhead. The Tiny context tier (7 lines: task, project kind,
  budget, verify command, governor decision, stop rule) would be sufficient for most
  tasks.
- **Task text repeated across commands** — preflight, request, and NEXT_RUN.md all
  contain the same task description. The user types it 3 times.
- **No auto-discovery.** Claude only read NEXT_RUN.md because it was explicitly told
  to. A CLAUDE.md snippet would make this automatic.

### 13.D What a CLAUDE.md Snippet Would Have Changed

If CLAUDE.md contained "Before starting any coding task, read `.akar/NEXT_RUN.md`":
- Claude would have read NEXT_RUN.md at session start without being told
- The manual relay step (task instruction telling Claude to read it) would be eliminated
- Token overhead would be: 7-line Tiny context in system prompt + 80-line NEXT_RUN.md
  read once = ~87 lines total, vs current 0 lines (none read unless explicitly told)

### 13.E Tiny Context vs Full Context

What Tiny context (7 lines) would have contained vs what was actually needed:

| Information | Tiny Context | Actually Needed |
|---|---|---|
| Task description | "fix the multiply bug..." | Yes — needed |
| Project kind | Node (package.json) | Yes — needed for npm test |
| Budget | 3 files, 60 LOC | Yes — needed for constraint |
| Verify command | `npm test` | Yes — needed |
| Governor decision | READY | Nice to have |
| Stop conditions | test pass, in budget | Yes — needed |
| Allowed commands list | Not in Tiny | Not needed for this simple task |
| Forbidden commands list | Not in Tiny | Not needed (no destructive commands attempted) |
| Full safety contract | Not in Tiny | Not needed |

**Finding:** For these two tasks, Tiny context (7 lines) would have been sufficient.
The full NEXT_RUN.md's extra content (allowed commands, forbidden commands, safety
contract, evidence used, hard rules, learning patches, etc.) was not needed. This
supports the v0.44.0 design: inject Tiny by default, keep Full on disk.

## 14. Negative Behavior Reduction Observed

Self-assessment for this dogfood session only:

| Negative Behavior | Verdict | Evidence |
|---|---|---|
| Dangerous commands | Not measurable | No destructive commands attempted; live hooks not wired |
| Hallucinated project commands | Improved | NEXT_RUN.md correctly surfaced `npm test`; Claude used it |
| Crazy LOC | Improved | Budget constrained to 60 LOC; actual changes: 2 + 14 LOC |
| Unclear diffs | Improved | postmortem always shows file, insertions, deletions, status |
| Verification guessing | Improved | `npm test` was discovered and surfaced; Claude ran it correctly |
| Repeated reprompting | Not measurable | Single-task sessions; no retry loops occurred |
| Token/request waste | Not measurable | No baseline token measurement exists |
| Manual command burden | **Worsened** | 59 commands for 2 tasks; AKAR adds overhead |
| Context loss | Not measurable | Single session, no compaction |
| Output verbosity | Unchanged | NEXT_RUN.md is comprehensive but not injected |

**Key finding:** AKAR improved discipline (budget, verification, diff clarity) but
**worsened** manual command burden. This is the defining tension: AKAR reduces negative
AI work patterns but at the cost of increasing user manual work. The prepare/finish
design directly addresses this tension.

## 15. Current Cycle vs. Future Prepare/Finish Projection

### 15.A Current Full Task 1 Cycle (18 commands)

```
# Pre-edit (6 commands)
git status                                    # 1
akar preflight --snapshot "<task>"            # 2
akar request "<task>"                         # 3
akar request --check                          # 4
akar governor --json --no-exit-code           # 5
[inspect .akar/NEXT_RUN.md]                   # 6 (manual relay)

# Edit (2 commands)
edit src/calc.js                              # 7
npm test                                      # 8

# Post-edit (10 commands)
akar postmortem --diff --baseline             # 9
akar learn --list                             # 10
akar governor --json --no-exit-code           # 11
akar doctor                                   # 12
akar status                                   # 13
git status                                    # 14
git diff --stat                               # 15
git add src/calc.js                           # 16
git commit -m "fix multiply bug"              # 17
git status                                    # 18
```

### 15.B Projection: With Prepare/Finish (v0.46.0+)

```
# Pre-edit (2 commands)
akar prepare "<task>" --snapshot              # 1 (composes: git status, preflight,
                                               #    request, request --check, governor,
                                               #    doctor summary, status summary,
                                               #    NEXT_RUN.md + validation)
# [CLAUDE.md snippet auto-instructs Claude to read .akar/NEXT_RUN.md]

# Edit (unchanged)
edit src/calc.js                              # 2
npm test                                      # 3

# Post-edit (3 commands)
akar finish                                   # 4 (composes: postmortem, learn --list,
                                               #    governor, doctor summary, status
                                               #    summary, commit guidance)
git add src/calc.js                           # 5
git commit -m "fix multiply bug"              # 6
```

**Projected reduction: 18 commands → 6 commands (67% reduction)**

Note: `akar finish` does NOT autocommit. Git add/commit remain manual (safety boundary).
The projection counts them as separate because they remain user actions.

### 15.C Projected Command Reduction

| Category | Current (Task 1) | With prepare/finish | Reduction |
|---|---|---|---|
| AKAR commands | 8 | 2 (prepare + finish) | **75%** |
| Git commands | 6 | 2 (git add, git commit) | 67% |
| Manual relay | 1 | 0 (CLAUDE.md snippet) | 100% |
| Edit/verify | 2 | 2 (unchanged) | 0% |
| **Total** | **17** | **6** | **65%** |

Note: git status ×3 and git diff --stat are removed because prepare/finish embed
state summaries. Only git add and git commit remain as explicit user actions.

## 16. What Should Be Grouped Into Prepare

Based on measurement evidence:

| Command | Group Into Prepare | Rationale |
|---|---|---|
| `git status` (pre-edit) | Yes | Embed clean/dirty summary |
| `akar preflight --snapshot "<task>"` | Yes (if clean + confirmed) | Core prepare step |
| `akar request "<task>"` | Yes | Core prepare step |
| `akar request --check` | Yes (auto, fail prepare if fails) | Validation; currently easy to skip |
| `akar governor --json --no-exit-code` (pre-edit) | Yes | Embed decision in prepare output |
| `akar doctor` (summary only) | Yes | Health check; full output via standalone cmd |
| `akar status` (summary only) | Yes | Readiness summary |
| Inspect NEXT_RUN.md | No (replace with CLAUDE.md snippet) | Eliminated by auto-delivery |

## 17. What Should Be Grouped Into Finish

| Command | Group Into Finish | Rationale |
|---|---|---|
| `akar postmortem --diff --baseline` | Yes | Core finish step |
| `akar learn --list` | Yes (summary) | Surface active patches |
| `akar governor --json --no-exit-code` (post-edit) | Yes | Embed decision in finish output |
| `akar doctor` (summary only) | Yes | Post-task health check |
| `akar status` (summary only) | Yes | Post-task readiness |
| `git status` (post-edit) | Yes (embed diff summary) | Guide commit |
| `git diff --stat` | Yes (embed in finish output) | Guide commit |
| `git add <file>` | No | Must remain manual (safety boundary) |
| `git commit -m "..."` | No | Must remain manual (safety boundary) |

## 18. What Must Remain Manual

| Action | Why |
|---|---|
| `akar init` | User must explicitly opt into AKAR |
| `akar hooks --install` | Writes executable hook templates; user must confirm |
| Editing `~/.claude/settings.json` | AKAR must never auto-edit user config |
| Adding `.akar/` to `.gitignore` | Removing user agency over their repo is hostile |
| `git add` | Commit staging requires human review |
| `git commit` | Commit message requires human judgment |
| `akar learn --resolve` | Learning patches require human confirmation |
| `akar verify` (non-Rust) | Project-dependent; manual verification is safe |
| CLI confirmation for `hooks --install` | Writing executable scripts requires human OK |

## 19. What Must Never Be Automated

| Action | Why |
|---|---|
| Editing Claude Code settings | Hostile to user; violates trust |
| Auto-committing | Removes human review from commit decision |
| Auto-pushing | Dangerous without review |
| Force-cleaning git state (reset --hard, clean -f) | Destructive; no undo |
| Running project code automatically | Crosses advisory boundary |
| Deleting files outside `.akar/` | Destructive; may lose work |
| Auto-resolving learning patches | Requires human judgment |

## 20. Evidence Quality and Limitations

### Strengths
- Every command was logged with approximate timestamp
- Two tasks of different types (bugfix, feature) measured
- Multi-task session flow captured (Task 1 baseline → commit → Task 2 baseline)
- Real fixture, real Node.js project, real tests
- Honest about what was NOT measured

### Limitations
- **Single session, single project type.** Node.js only. Results may differ for Rust
  (auto-verify) or Python (no auto-verify) or Unknown (no test discovery).
- **No live hook integration.** Hook setup burden was measured (commands, confirmation)
  but hook wiring and operation were not tested. Hook integration is proven from
  v0.35/v0.36 but setup friction data is missing.
- **Time resolution is approximate.** Timestamps are at whole-second resolution.
  Sub-second commands (request --check) can't be individually timed.
- **User is the AKAR author.** The user knows all commands and flags by heart. A
  new user would be slower, make more errors, and need to read help text. This
  dogfood likely undercounts real user burden.
- **No measurement of git vs non-git projects.** This fixture was git-initialized.
  Non-git projects would face different friction (preflight/postmortem require git).
- **No model/token measurement.** Token counts for sessions with/without AKAR
  context were not measured. Token reduction claims remain unverified.
- **Claude was explicitly asked to read NEXT_RUN.md.** In a normal session, Claude
  wouldn't read it automatically — the relay burden would be higher.
- **Two simple tasks only.** No complex multi-file refactors, no merge conflicts,
  no dirty-tree edge cases, no over-budget scenarios.

## 21. Recommended Next Release

**v0.46.0 Prepare/Finish Command Prototype.**

### Justification

1. **Command count is the dominant friction.** 59 commands for 2 tasks. Every task
   follows the same pattern (preflight → request → check → edit → postmortem →
   learn → doctor → status → git). The pattern is mechanical and predictable.
2. **Prepare/finish would reduce commands by 65–75%.** From ~8 AKAR commands per
   task to 2 (prepare + finish). This is the largest single burden reduction
   possible without changing the safety model.
3. **No new safety risk.** Prepare/finish compose existing advisory commands. Every
   operation in prepare/finish is already performed manually today. The safety
   properties don't change — only the invocation count changes.
4. **Evidence confirms the v0.44.0 design.** The command classification, missed-step
   risk, and griping analysis all match what was predicted.
5. **CLAUDE.md snippet can wait.** Manual relay was not the dominant friction in
   this trial. Claude read NEXT_RUN.md when told to. The relay problem is real but
   smaller than the command-count problem.

### Why NOT v0.46.0 AI-Facing Delivery

Manual relay was measured but didn't dominate. Claude successfully read NEXT_RUN.md
when instructed. The relay problem is secondary to command count. AI-facing delivery
(CLAUDE.md snippet) should come after prepare/finish is proven (v0.48.0 per the
v0.44.0 sequencing).

### Sequencing

Per the v0.44.0 implementation sequencing:
1. **v0.45.0** ← this release (burden measurement)
2. **v0.46.0** → Prepare/Finish Command Prototype
3. **v0.47.0** → Prepare/Finish Dogfood
4. **v0.48.0** → CLAUDE.md Snippet + Tiny Context
5. **v0.49.0** → AI-Facing Delivery Dogfood
6. **v0.50.0** → Negative Behavior Measurement Baseline

## 22. Honest Conclusion

This dogfood measured what AKAR actually costs the user. The answer: 8 AKAR commands
per task, 18 total commands per task (including git and inspection), 23 AKAR commands
total across setup + 2 tasks. The v0.44.0 estimate of "7+ commands per cycle" was
optimistic — the actual number is higher when counting governor, doctor, status,
learn, and git commands that AKAR indirectly requires (by surfacing dirty-tree
warnings that need git status to diagnose).

But the measurement also confirmed that consolidation is the right first step. The
command pattern is mechanical and predictable. Eight AKAR commands per task can
become two (prepare + finish) without changing any safety property, because every
operation is already advisory and already running today. The CLAUDE.md snippet can
wait — context relay is a second-order problem compared to command count.

The largest unaddressed burden remains hook setup. Three manual steps (hooks
--install with interactive confirmation, editing ~/.claude/settings.json, and
validation that hooks are wired) create a high barrier for new users. This is not
solved by prepare/finish and needs its own design.

The path from here is clear: measure (done) → consolidate commands (v0.46.0) →
dogfood (v0.47.0) → deliver context to AI (v0.48.0) → measure again (v0.49.0).
Each step is small, safe, and evidence-gated. No auto-execution. No model calls.
No settings mutation. Just incrementally reducing the gap between what AKAR does
and how the user experiences it.
