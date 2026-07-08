# AKAR v0.44.0 — Manual-CLI Burden Reduction Design

## 1. Executive Verdict

**Design report. No implementation.** AKAR's current manual command cycle requires the
user to run 7+ CLI commands per task, bridging AKAR output into the AI session
manually. This contradicts the North Star mandate that AKAR "should not be a manual
checklist the user repeatedly operates."

This report designs a three-phase burden reduction path that can be built without
auto-executing project code, without model/API calls, and without modifying Claude Code
settings automatically:

1. **Command consolidation** — reduce the per-task cycle to 2 commands (`akar prepare`
   and `akar finish`) by grouping safe advisory operations.
2. **AI-facing context delivery** — design how AKAR context reaches the AI session
   through a managed project instruction snippet, without user copy/paste.
3. **Auto-invocation** — trigger advisory commands at session lifecycle moments via
   existing PreToolUse hook infrastructure.

All three phases preserve AKAR's advisory-only boundary. AKAR never executes project
code. The AI model remains the worker. The design is concrete enough to implement but
intentionally leaves implementation for future releases.

## 2. Problem Statement

The v0.43 North Star drift assessment identified a "manual-CLI trap" — AKAR has become
a manual checklist the user repeatedly operates. The evidence:

**Current per-task cycle (7+ commands):**
```
akar preflight --snapshot "<task>"    # baseline snapshot
akar request "<task>"                 # compile NEXT_RUN.md
akar request --check                  # validate
[user inspects or relays NEXT_RUN.md]
[AI session — user manually bridges context]
akar postmortem --diff --baseline     # measure diff
akar learn --list                     # check patches
akar status                           # check state
```

**One-time setup:**
```
akar init                             # bootstrap project
akar hooks --install                  # write hook templates
[user manually edits ~/.claude/settings.json]
[user manually decides .akar/ tracking in .gitignore]
akar doctor                           # verify environment
```

**The North Star says:**
- "AKAR should not be a manual checklist the user repeatedly operates"
- "User installs AKAR and enables it inside their preferred CLI/TUI AI tool"
- "AKAR should clarify the job between user and AI without creating extra fuss"

Current AKAR creates extra fuss — the fuss of 7+ manual CLI invocations per task. The
value (discipline, safety, diff measurement) is real, but the delivery mechanism
(manual CLI operated by the user) contradicts the North Star.

## 3. Baseline and Verification

| Check | Result |
|---|---|
| Commit | `b53adcf` — docs: assess AKAR north star drift and gaps |
| Version | `akar 0.43.0` |
| Working tree | clean |
| `cargo test` | 508 passed, 0 failed |
| `cargo run -- eval` | 28/28 PASS |
| `cargo run -- doctor` | PASS (1 WARN: split-rule) |
| `cargo run -- status` | HEALTHY, READY |
| `cargo run -- request "..."` | NORMAL mode |
| `cargo run -- request --check` | PASS (4/4) |
| `cargo run -- governor` | SPLIT_TASK (known artifact) |
| `cargo run -- learn --list` | 8 entries (1 active, 7 resolved) |
| `cargo run -- hooks --check` | PASS (source-tree) |

## 4. Current Manual Command Cycle

Every AKAR command and its burden classification:

### One-Time Setup Commands

| Step | Command | Purpose | Writes State | Could Auto-Invoke | Must Remain Manual | Risk if Auto'd |
|---|---|---|---|---|---|---|
| 1 | `akar init` | Bootstrap .akar/ directory and memory files | Yes (.akar/ files) | No | **Yes** — user must explicitly opt into AKAR | Medium — auto-init on `cd` would be surprising |
| 2 | `akar hooks --install` | Write hook templates to .akar/hooks/ | Yes (hook files) | No (today) | **Yes** — user must confirm, but could be guided | Low — writing templates is safe; the risk is user not knowing it happened |
| 3 | Manual wiring | Edit ~/.claude/settings.json | No (AKAR doesn't touch it) | No | **Yes** — AKAR must never edit Claude settings automatically | High — auto-editing user config is hostile |
| 4 | `.gitignore` decision | User decides .akar/ tracking | No | No | **Yes** — AKAR must never auto-modify .gitignore | High — removing user agency over their repo |
| 5 | `akar doctor` | Verify environment | No | Yes — could run as part of init/prepare | **Partial** — one-time but useful to re-check | Low — read-only check |

### Per-Task Commands

| Step | Command | Purpose | Writes State | Could Auto-Invoke | Must Remain Manual | Risk if Auto'd |
|---|---|---|---|---|---|---|
| 6 | `akar status` | Check current state | No | Yes — could embed in prepare output | No | Low |
| 7 | `akar preflight --snapshot "<task>"` | Write diff baseline | Yes (DIFF_BASELINE.json) | Yes — but requires clean tree; dirty tree needs user action | **Partial** — auto only if tree is clean | Medium — snapshot on dirty tree produces wrong baseline |
| 8 | `akar request "<task>"` | Compile NEXT_RUN.md | Yes (NEXT_RUN.md) | **Yes** — safe; writes advisory file only | No | Low |
| 9 | `akar request --check` | Validate NEXT_RUN.md | No | **Yes** — safe; read-only validation | No | Low |
| 10 | [AI session] | User works with Claude | N/A | N/A | N/A | N/A |
| 11 | `akar postmortem --diff --baseline` | Measure diff vs baseline | Yes (telemetry) | **Yes** — safe; measurement only | No | Low |
| 12 | `akar learn --list` | Check learning patches | No | Yes — could embed in finish output | No | Low |
| 13 | `akar learn --resolve` | Retire resolved patches | Yes (LEARNING_PATCHES.md) | No | **Yes** — user must confirm learning | Medium — auto-resolving patches could hide real issues |

### Expert/Meta Commands

| Command | Purpose | Frequency | User Burden |
|---|---|---|---|
| `akar governor` | Check governor decision | Per-task (embedded in prepare/finish) | Low |
| `akar verify` | Run verification | Varies by project kind | Low |
| `akar safety <cmd>` | Classify a command | As needed | Low |
| `akar eval` | Run eval suite | Development only | Low |
| `akar telemetry` | View metrics | Rare | Low |
| `akar mission` | Scaffold mode | Rare | Low |
| `akar run` | Full state machine walk | Rare | Low |
| `akar skills` | List skills | Rare | Low |
| `akar calibrate` | Model profile | Rare | Low |

### Burden Summary

- **One-time setup:** 5 steps (init, hooks install, manual wiring, .gitignore, doctor)
- **Per-task cycle:** 7 steps (status, preflight, request, request --check, postmortem,
  learn --list, learn --resolve if needed)
- **Total per task:** ~7 manual CLI invocations
- **User must remember order, flags, and task text for each command**
- **User must manually bridge NEXT_RUN.md content into AI session**
- **No command depends on another's output in a way the user can skip** — each produces
  a file the next reads

## 5. Auto-Invocation vs. Auto-Execution

These terms must be defined precisely before any implementation.

### Auto-Invocation

AKAR automatically runs its own advisory/read-only/state-writing commands at lifecycle
moments (session start, task start, task end, session end).

Auto-invocation **may:**
- Run `akar request "<task>"` to generate NEXT_RUN.md
- Run `akar preflight --snapshot "<task>"` if tree is clean and user confirmed
- Run `akar request --check` to validate output
- Run `akar postmortem --diff --baseline` to measure diff
- Run `akar status` and embed its output in other commands
- Run `akar doctor` and surface warnings
- Write AKAR-owned state files under `.akar/`
- Read git status, diff, HEAD
- Ask for user confirmation when state is ambiguous

Auto-invocation **must not:**
- Run project tests (except Rust `cargo build`/`cargo test` via verify, which is
  bounded and project-kind-specific)
- Install dependencies
- Edit project source files
- Edit Claude Code settings (`~/.claude/settings.json`)
- Modify `.gitignore`
- Commit, push, reset, clean, stash, checkout
- Delete files outside `.akar/`
- Call model APIs
- Hide failures or silently continue after dangerous state
- Make decisions about task completion or code quality

### Auto-Execution

AKAR runs project commands, edits project files, or makes decisions about project work.

Auto-execution **remains out of scope for all current and planned AKAR versions**
until and unless:
- An explicit, separate design proposal is accepted
- Safety boundaries are proven in isolation
- External users request it with specific use cases
- A toggle mechanism exists with default-off

### The Distinction in Practice

| Action | Category | Allowed Now | Allowed in Future Design |
|---|---|---|---|
| `akar request "<task>"` | Auto-invocation (AKAR advisory) | Manual only | Yes (Layer 2) |
| `akar postmortem --diff --baseline` | Auto-invocation (AKAR advisory) | Manual only | Yes (Layer 2) |
| `cargo build` (Rust verify) | Bounded project execution | Manual only | Manual only (existing boundary) |
| `npm test` (Node verify) | Project execution | Manual only | Manual only |
| `git commit` | Project mutation | Manual only | Never |
| Editing source files | Project mutation | Never | Never |

## 6. AI-Facing Delivery Options

Seven mechanisms evaluated for how AKAR context reaches the AI session. None are
currently implemented. This is design analysis only.

### A. Project File Delivery (Current Default)

**How it works:** AKAR writes `.akar/NEXT_RUN.md`. Claude discovers it from the
filesystem when instructed or when exploring the project. No direct injection.

**What user sees:** User must tell Claude "read .akar/NEXT_RUN.md" or hope Claude
discovers it. In dogfood trials, this works when the user is disciplined but fails
when the user forgets.

**How AI receives context:** AI reads the file with the Read tool. Content is
available if the AI looks for it, invisible if it doesn't.

**Commands reduced:** None. User must still relay or prompt the AI to read.

**Safety risks:** None. File is on disk like any other project file.

**Portability:** High. Works on any filesystem.

**Claude Code compatibility:** **Proven.** Works today — the AI can read
`.akar/NEXT_RUN.md` from disk if prompted. The problem is not compatibility, it's
discoverability.

**Advisory boundary preserved:** Yes.

**Toggle support:** Implicit — removing the file removes the context.

**Recommendation:** **Keep as default fallback.** Not sufficient as the only
mechanism because discoverability depends on user diligence.

### B. CLAUDE.md Managed Snippet

**How it works:** AKAR writes or suggests a small section in the project's CLAUDE.md
(similar to how CLAUDE.md currently contains project instructions). The section points
Claude to `.akar/NEXT_RUN.md` or `.akar/session/` files. AKAR would add a block like:

```markdown
## AKAR Session Guidance
Before starting any coding task, read `.akar/NEXT_RUN.md`. It contains the
current task objective, budget limits, allowed commands, and stop conditions.
After completing work, verify against those constraints.
```

**What user sees:** On `akar init`, AKAR asks: "Add AKAR guidance to CLAUDE.md?
This helps Claude discover session context automatically. [y/N]". If accepted, AKAR
appends the snippet. On `akar init --claude`, the snippet is included. CLAUDE.md
can be version-controlled alongside other project instructions.

**How AI receives context:** Claude reads CLAUDE.md at session start (it's loaded
into the system prompt). The snippet instructs Claude to read `.akar/NEXT_RUN.md`
before starting work. This is the key advantage — CLAUDE.md is auto-loaded into
Claude's context, so the instruction to read NEXT_RUN.md is always present.

**Commands reduced:** Eliminates user relay/copy-paste step. User no longer needs to
tell Claude to read NEXT_RUN.md — the CLAUDE.md instruction does it automatically.

**Safety risks:** Medium. AKAR writes to a project file (CLAUDE.md). This is more
invasive than writing to `.akar/` only. The snippet must be clearly demarcated so the
user can remove it. Must not overwrite existing CLAUDE.md content. Must ask for
confirmation. Must never edit CLAUDE.md without explicit user action (`akar init` or
a dedicated `akar claude-init` command).

**Portability:** High. CLAUDE.md is a standard Claude Code convention. Other AI tools
may have equivalent files but would need their own snippets.

**Claude Code compatibility:** **Proven.** CLAUDE.md is loaded into the system prompt
at session start. This is documented Claude Code behavior.

**Advisory boundary preserved:** Yes. The snippet is an instruction to read a file,
not an execution command.

**Toggle support:** Easy — remove the snippet from CLAUDE.md to disable. The presence
of the snippet is visible and auditable.

**Recommendation:** **Accept as primary Layer 2 delivery mechanism.** This is the
smallest change with the highest leverage. A single CLAUDE.md snippet that says "read
`.akar/NEXT_RUN.md`" eliminates the user relay step entirely. The snippet is opt-in
(confirmed during `akar init`), visible (in CLAUDE.md), auditable (in git), and
removable (delete the lines). It does not auto-execute anything.

### C. Hook-Mediated Reminder

**How it works:** The existing PreToolUse hook detects the first Bash/Read/Edit tool
use in a session and writes a reminder to a log file or hook output. The hook could
embed a short reminder like "Check .akar/NEXT_RUN.md before continuing."

**What user sees:** Nothing visible. The hook fires silently on every tool call
already. A reminder would appear in hook output if Claude Code surfaces it.

**How AI receives context:** **Uncertain.** Hook stdout/stderr is not guaranteed to
be injected into the model's context by Claude Code. The hook exit code affects
whether the tool call proceeds, but hook text output is not documented as
model-visible. Without proof that hook output reaches the AI, this mechanism is
unreliable.

**Commands reduced:** None directly. Could remind the AI to read NEXT_RUN.md but
can't guarantee the AI sees the reminder.

**Safety risks:** Low. Hook already fires on every tool call. Adding a log line or
reminder is low-risk. Risk is false confidence — believing the AI received context
when it didn't.

**Portability:** Claude Code-specific (PreToolUse hook schema).

**Claude Code compatibility:** **Unknown.** Hook output visibility to the model is
not documented. Must be tested before relying on it. If hook output is NOT
model-visible, this mechanism is useless for context delivery.

**Advisory boundary preserved:** Yes.

**Toggle support:** Hook can be disabled or modified.

**Recommendation:** **Postpone — test first.** Before designing around hook-mediated
reminders, verify whether Claude Code surfaces hook output to the model. If yes, this
becomes a powerful auto-injection mechanism. If no, it's a dead end. Do not design
around unverified behavior.

### D. Slash Command / Command File

**How it works:** AKAR provides a project-level command (e.g., `/akar:context` or a
shell script) that the user invokes inside Claude Code. The command reads and
summarizes `.akar/NEXT_RUN.md` for the AI. Reduces copy/paste but still requires user
invocation.

**What user sees:** User types `/akar:context` or runs a local script in Claude Code.
The output is the AKAR context, visible in the conversation.

**How AI receives context:** The command output appears in the conversation context.
The AI sees it directly.

**Commands reduced:** Reduces copy/paste to one invocation. Still manual.

**Safety risks:** Low. Command is read-only.

**Portability:** Medium. Slash commands are Claude Code-specific. Shell scripts are
portable.

**Claude Code compatibility:** **Proven.** Slash commands and shell scripts work.

**Advisory boundary preserved:** Yes.

**Toggle support:** User simply stops invoking the command.

**Recommendation:** **Accept as intermediate Layer 1 solution.** A `/akar` slash
command or project command file is better than copy/paste but worse than auto-injection
via CLAUDE.md. It could serve as the user-facing invocation for `akar prepare` output
before Layer 2 auto-injection is ready. Worth building as a stepping stone.

### E. Shell Wrapper / Launcher

**How it works:** User starts Claude Code through an AKAR wrapper script. The wrapper
runs `akar prepare "<task>"` before launching Claude, and optionally `akar finish`
after Claude exits.

**What user sees:** User runs `akar-claude "<task>"` instead of `claude`. The wrapper
runs preflight/request, then launches Claude with the project directory.

**How AI receives context:** NEXT_RUN.md is on disk. If CLAUDE.md snippet is present,
AI reads it. If not, same discoverability problem as option A.

**Commands reduced:** Combines prepare + launch into one command. Still requires
wrapper invocation.

**Safety risks:** Medium. Wrapper could obscure errors. User must trust the wrapper
not to modify Claude behavior.

**Portability:** Low. Wrapper is shell-specific (bash vs PowerShell). Different on
each platform. Fragile — breaks if Claude Code launch mechanism changes.

**Claude Code compatibility:** **Likely but fragile.** Claude Code is invoked via
`claude` command. Wrapper works as long as the invocation path is stable.

**Advisory boundary preserved:** Yes — wrapper only runs AKAR advisory commands.

**Toggle support:** Use wrapper for AKAR-enhanced sessions, plain `claude` for normal
sessions.

**Recommendation:** **Reject for v1.0.** Too fragile, too platform-specific, too much
magic. The wrapper approach hides AKAR's behavior behind an opaque launcher, which
contradicts AKAR's transparency principle. The CLAUDE.md snippet approach (option B)
achieves the same result more transparently.

### F. File Watcher / Daemon

**How it works:** AKAR runs as a background process watching repo changes. When it
detects a clean tree with no NEXT_RUN.md, it prompts the user. When it detects a
dirty tree after a period of changes, it suggests running postmortem.

**What user sees:** Desktop notifications or terminal messages suggesting AKAR
actions. User still confirms manually.

**How AI receives context:** No direct AI injection — daemon notifies user, not AI.

**Commands reduced:** Reminders reduce forgetting, but user still runs commands
manually.

**Safety risks:** High. Daemon has persistent filesystem access. Could conflict with
git operations. On Windows, background processes have different lifecycle than Unix.
Adds complexity without clear benefit over simpler approaches.

**Portability:** Low. Daemon behavior differs significantly across platforms.

**Claude Code compatibility:** **Risky.** File watcher could trigger during AI
sessions, causing race conditions. AI might modify files while watcher is reading
them.

**Advisory boundary preserved:** Yes — daemon only advises, doesn't execute.

**Toggle support:** Daemon can be started/stopped.

**Recommendation:** **Reject for v1.0 and likely v1.x.** Too complex, too risky,
too platform-specific. The benefit (reminders) doesn't justify the complexity when
simpler mechanisms (CLAUDE.md snippet, slash command) achieve the same result.

### G. Manual Status Quo

**How it works:** User continues to run all AKAR commands manually, as today.

**What user sees:** Full CLI cycle. 7+ commands per task.

**How AI receives context:** User copies/pastes or tells Claude to read files.

**Commands reduced:** None.

**Safety risks:** None — manual operation is the safest mode.

**Portability:** High — manual CLI works everywhere.

**Claude Code compatibility:** **Proven.** Works today.

**Recommendation:** **Keep as fallback, not as target.** Manual operation should
always be available as a fallback and for expert users who want fine-grained control.
But it must not be the only mode for normal users.

### Delivery Options Summary

| Option | AI Context Delivery | Safety | Portability | CC Compat | Recommendation |
|---|---|---|---|---|---|
| A. Project file | Indirect (AI must discover) | Low risk | High | Proven | Keep as fallback |
| B. CLAUDE.md snippet | Direct (loaded into system prompt) | Medium risk (edits project file) | High | Proven | **Accept — primary mechanism** |
| C. Hook reminder | Uncertain (hook output may not be model-visible) | Low risk | CC-specific | Unknown | Postpone — test hook visibility first |
| D. Slash command | Direct (via conversation) | Low risk | Medium | Proven | Accept — intermediate solution |
| E. Shell wrapper | Same as A (file on disk) | Medium risk | Low | Likely | Reject — too fragile |
| F. File watcher | No AI delivery (notifies user) | High risk | Low | Risky | Reject — too complex |
| G. Manual status quo | Indirect (user relays) | No risk | High | Proven | Keep as fallback |

## 7. Minimum Viable Burden Reduction

The smallest safe path from 7+ commands to fewer, without auto-execution, without
model/API calls, without editing Claude settings automatically.

### Layer 1A — Prepare Command

A single advisory command that combines safe AKAR-only steps that happen before an AI
session:

```
akar prepare "<task>"
```

This future command would:
1. Run `akar status` checks (git state, doctor summary, readiness)
2. If tree is clean and user confirmed (via `--snapshot` flag or interactive prompt):
   run `akar preflight --snapshot "<task>"`
3. Run `akar request "<task>"` to compile NEXT_RUN.md
4. Run `akar request --check` to validate
5. Print a compact summary: project kind, task, budget, governor decision, verification
   command, any warnings

It would NOT:
- Run project code (tests, builds, installs)
- Edit project source
- Commit, push, or modify git state
- Edit Claude Code settings
- Auto-resolve learning patches
- Hide failures — if request --check fails, the command fails with the error

**User burden reduction:** 7 commands → 2 commands (prepare + finish).
**Safety:** Same as current manual cycle — all operations are already manual today.
**Implementation risk:** Low — combines existing commands, doesn't add new behavior.

### Layer 1B — Finish Command

A single advisory command that combines safe AKAR-only steps that happen after an AI
session:

```
akar finish
```

This future command would:
1. Run `akar postmortem --diff --baseline` to measure diff
2. Run `akar learn --list` and show active patches
3. Run `akar status` summary
4. Print commit guidance: "2 files, 15 LOC. Run git diff --stat to review, then
   git add + git commit. Do not force-clean or stash."
5. If postmortem shows over-budget: surface SPLIT_TASK guidance

It would NOT:
- Commit, push, or reset
- Auto-resolve learning patches (user must run `akar learn --resolve` explicitly)
- Revert changes
- Run project verification (user runs that separately)

**User burden reduction:** Eliminates 3 separate post-task commands.
**Safety:** Same as current manual cycle.

### Layer 2 — AI-Facing Context Delivery

After prepare/finish are proven, add the CLAUDE.md snippet mechanism:

1. `akar init` or `akar init --claude` offers to add an AKAR snippet to CLAUDE.md
2. The snippet instructs Claude to read `.akar/NEXT_RUN.md` before starting any
   coding task
3. Claude reads NEXT_RUN.md at session start (because CLAUDE.md is in the system
   prompt)
4. User no longer needs to relay context or tell Claude to read the file

This is the critical transition from manual-CLI to enhancement layer. The user still
runs `akar prepare "<task>"` (Layer 1A), but no longer needs to bridge the output
into the AI session. Claude receives the instruction automatically.

## 8. Command Consolidation Candidates

Every current AKAR command classified for future consolidation:

| Command | Classification | Rationale |
|---|---|---|
| `init` | **Keep manual** | One-time setup; user must explicitly opt in |
| `hooks --install` | **Keep manual** (but guided) | User must confirm; could be offered during init |
| `hooks --check` | **Group into prepare** | Run as part of prepare health check |
| `doctor` | **Group into prepare** (summary mode) | Run as part of prepare; full output available via standalone command |
| `status` | **Group into prepare + finish** | Embed summary in both |
| `preflight --snapshot` | **Group into prepare** (if clean + confirmed) | Core prepare step; skip if tree dirty (warn user) |
| `request` | **Group into prepare** | Core prepare step |
| `request --check` | **Group into prepare** | Validation step; fail prepare if check fails |
| `governor` | **Group into prepare** (summary) | Embed decision in prepare/finish output |
| `verify` | **Keep manual** | Project-dependent; Rust auto-verify is bounded; other projects manual-only |
| `postmortem --diff --baseline` | **Group into finish** | Core finish step |
| `learn --list` | **Group into finish** (summary) | Embed summary in finish output |
| `learn --resolve` | **Keep manual** | Requires user confirmation; learning patches need human judgment |
| `safety` | **Hide from normal users** | Used by hooks internally; rare direct use |
| `eval` | **Expert-only** | Development/testing only |
| `telemetry` | **Expert-only** | Rare diagnostic use |
| `mission` | **Expert-only** | Scaffold mode; rare use |
| `run` | **Expert-only** | Full state machine walk; rare use |
| `skills` | **Expert-only** | Inventory check; rare use |
| `calibrate` | **Expert-only** | Model profile; rare use |

### User-Visible Command Surface After Consolidation

**Normal user commands (Layer 1):**
- `akar init` — one-time setup
- `akar prepare "<task>"` — before AI session
- `akar finish` — after AI session
- `akar doctor` — environment check (also embedded in prepare)
- `akar hooks --install` — hook setup (offered during init)

**Expert/advanced commands (available but not advertised as primary):**
- All current standalone commands remain available for scripting, debugging, and
  expert use
- `akar preflight --snapshot "<task>"` — manual baseline only
- `akar request "<task>"` — manual NEXT_RUN only
- `akar postmortem --diff --baseline` — manual measurement only
- `akar learn --resolve` — manual patch resolution
- `akar verify` — manual verification
- `akar eval`, `akar telemetry`, `akar governor`, etc.

**This is critical:** Consolidation adds convenience commands. It does not remove
existing commands. Expert users who want fine-grained control can still use every
individual command. Normal users get a simpler surface.

## 9. Future Prepare Command Design

```
akar prepare "<task>" [--snapshot] [--no-snapshot] [--json]
```

### Behavior

1. **Health check (always):**
   - Run doctor summary (environment, files, hooks, telemetry, git)
   - Run status summary (readiness, governor decision baseline)
   - If doctor FAIL: abort with guidance (fix issues first)
   - If doctor WARN: proceed with warnings displayed

2. **Baseline snapshot (conditional):**
   - If `--snapshot` flag: run `preflight --snapshot "<task>"` regardless
   - If `--no-snapshot` flag: skip snapshot
   - If neither flag:
     - Tree clean → prompt "Create diff baseline snapshot? [Y/n]"
     - Tree dirty → warn "Working tree is dirty. Cannot snapshot. Review changes
       first." (do not snapshot)
   - If snapshot fails (dirty tree, no git repo): warn and continue without snapshot

3. **NEXT_RUN compilation (always):**
   - Run `request "<task>"` to compile NEXT_RUN.md
   - Run `request --check` to validate
   - If validation fails: abort with the validator error
   - If validation passes: continue

4. **Summary output:**
   ```
   prepare: READY
     project:  Rust (Cargo.toml)
     task:     "fix the login button"
     budget:   3 files, 60 LOC (Bugfix)
     verify:   cargo build && cargo test
     governor: READY — proceed with task
     baseline: snapshot taken (3 files, 60 LOC baseline)

     warnings:
       - LEARNING_PATCHES.md has 1 active split-rule entry

     next: start your AI session. Claude will read .akar/NEXT_RUN.md
           automatically (CLAUDE.md configured).
   ```

### Flags

| Flag | Purpose |
|---|---|
| `--snapshot` | Always take baseline snapshot (even if dirty — user accepts risk) |
| `--no-snapshot` | Skip baseline snapshot entirely |
| `--json` | Machine-readable output for scripting/wrapper use |
| `--compact` | One-line summary only |

### Safety Properties

- Never executes project code
- Never edits project source
- Never commits or modifies git state
- Never edits Claude Code settings
- All operations are existing advisory commands composed together
- Failures in any step abort the command with clear error messages
- Warnings are surfaced but don't block progress

## 10. Future Finish Command Design

```
akar finish [--json] [--compact]
```

### Behavior

1. **Diff measurement (always):**
   - Run `postmortem --diff --baseline`
   - Compare against budget from NEXT_RUN.md
   - If over-budget: surface SPLIT_TASK guidance
   - If within budget: PASS

2. **Learning patch check (always):**
   - Run `learn --list` summary
   - Show active patches
   - If active split-rule entries: suggest `akar learn --resolve` when ready

3. **State summary (always):**
   - Run `status` summary (readiness, governor decision)
   - Show commit guidance: changed files, LOC, suggested next action

4. **Summary output:**
   ```
   finish: PASS
     diff:     2 files, 15 LOC (budget: 3 files, 60 LOC)
     budget:   within limits
     patches:  1 active (split-rule)
     state:    RUN_POSTMORTEM → commit → clean → ready for next task

     next: review changes with 'git diff --stat', then commit.
           Do not force-clean or stash. Run 'akar prepare "<next task>"'
           when ready for the next task.
   ```

### What Finish Does NOT Do

- Does not commit
- Does not push
- Does not reset or clean
- Does not run verification (user runs that separately)
- Does not auto-resolve learning patches
- Does not start the next task

### Flags

| Flag | Purpose |
|---|---|
| `--json` | Machine-readable output |
| `--compact` | One-line summary only |

## 11. AI-Facing Context Delivery Design

### Primary Mechanism: CLAUDE.md Managed Snippet

**Snippet content (proposed):**

```markdown
## AKAR Session Guidance (managed by `akar init`)

Before starting any coding task, read `.akar/NEXT_RUN.md`. It contains:
- The current task objective and scope
- Budget limits (files and lines of code)
- Allowed and forbidden commands
- Required verification steps
- Stop conditions

After completing work, verify you stayed within the budget and followed the
stop conditions. The user will run `akar finish` to measure the diff.
```

**Delivery mechanism:**

1. `akar init` detects whether CLAUDE.md exists
2. If CLAUDE.md exists:
   - Offer to append the AKAR snippet: "Add AKAR session guidance to CLAUDE.md?
     This helps Claude discover your session context automatically. [y/N]"
   - If accepted: append snippet at the end of CLAUDE.md with clear delimiters
   - If rejected: skip; user can add manually later or use `akar init --claude`
3. If CLAUDE.md does not exist:
   - `akar init --claude` creates CLAUDE.md with the snippet
   - Plain `akar init` offers to create CLAUDE.md with the snippet
4. `akar init --claude` always adds/updates the snippet (idempotent — replace
   existing AKAR section if present, leave rest of CLAUDE.md untouched)

**Safety properties:**
- Snippet is clearly delimited (`## AKAR Session Guidance (managed by akar init)`)
- AKAR only modifies its own section; never touches other CLAUDE.md content
- User can remove the section at any time by deleting those lines
- Snippet is added to git like any other project file — user decides to commit it
- AKAR never modifies CLAUDE.md outside of `akar init` or `akar init --claude`
- The snippet is an instruction to READ a file, not to EXECUTE anything

**Why this works:**
CLAUDE.md is loaded into Claude's system prompt at session start. Any instruction
in CLAUDE.md becomes part of Claude's permanent context for that session. "Read
`.akar/NEXT_RUN.md`" in CLAUDE.md means Claude will read NEXT_RUN.md at the start
of every session without the user asking.

This is not speculation — it's how CLAUDE.md works today. The only question is
whether AKAR should write to it, and the answer is: yes, with explicit user
confirmation, in a clearly delimited section, as part of `akar init`.

### Secondary Mechanism: Slash Command (Intermediate)

For projects where the user prefers not to modify CLAUDE.md, a slash command
provides an intermediate solution:

```
/akar:context
```

The command reads `.akar/NEXT_RUN.md` and outputs a compact summary. The user
invokes it once at the start of a session. Better than copy/paste, worse than
auto-injection.

This can be implemented as a Claude Code custom slash command pointing at a
small script that AKAR installs during `akar init`.

## 12. Toggle Model

Auto-run / AKAR enhancement must be switchable on/off per the North Star.

### Evaluated Options

| Option | Pros | Cons | Safety | Clarity | Portability |
|---|---|---|---|---|---|
| `.akar/config.toml` | Project-scoped, versionable, explicit | New file format, needs parser | High — file on disk | High — visible in project | High |
| Environment variable (`AKAR_ENABLE=1`) | Simple, session-scoped | Ephemeral, easy to forget, not project-persistent | Medium — env can leak | Low — invisible | High |
| Command flag (`akar enable`/`akar disable`) | Explicit action, user intention clear | Requires remembering to run it | High — explicit | Medium — state in file | High |
| Project-local marker file (`.akar/enabled`) | Simple, exists/doesn't exist | Binary only, no config granularity | High — file on disk | Medium — need command to check | High |
| Global user config (`~/.akar/config.toml`) | Cross-project defaults | Cross-project bleed, harder to reason about | Medium — global state | Low — not in project | High |

### Recommendation: `.akar/config.toml` + `akar enable`/`akar disable` commands

**`.akar/config.toml`** stores project-level AKAR configuration:

```toml
[auto]
# Auto-invoke prepare on session start (future)
prepare_on_session_start = false
# Auto-invoke finish on session end (future)
finish_on_session_end = false
# Add AKAR guidance to CLAUDE.md
claude_md_snippet = true
# Auto-snapshot on prepare (skip confirmation prompt)
auto_snapshot = false
```

**Commands:**
- `akar enable` — sets the relevant config flags to true
- `akar disable` — sets all auto flags to false
- `akar config` — shows current config

**Default:** All auto features default to `false` (off). User must explicitly enable.

**State visibility:** `akar status` and `akar doctor` show whether auto-run is
enabled and which features are active. The enabled/disabled state is always obvious.

**Safety:** Toggle-off is immediate — setting `auto.*` to false in config.toml
disables all auto behavior. No hidden state. No background processes. The config
file is in `.akar/` (not tracked by git unless user chooses to).

## 13. AI Context Size Budget

The North Star says AKAR should reduce wasted tokens. The current NEXT_RUN.md is
comprehensive (~80–120 lines) — useful for full context but too large for frequent
injection. A size budget is needed.

### Size Tiers (Design Only)

| Tier | Max Lines | Content | Use Case |
|---|---|---|---|
| **Tiny** | 10 | Task, project kind, verification command, budget, governor decision, stop rule | Default injected context (every session) |
| **Normal** | 30 | Tiny + allowed commands, forbidden commands, stop conditions, verification hints | Expanded context (complex tasks) |
| **Full** | Current | All 11 sections of NEXT_RUN.md | On-disk reference (AI reads when needed) |

### Tiny Context Design

```markdown
## AKAR Task Context
- Task: fix the login button
- Project: Rust (Cargo.toml)
- Budget: 3 files, 60 LOC
- Verify: cargo build && cargo test
- Stop if: over budget, dirty tree, failing tests
- Governor: READY
```

This is 7 lines. It contains the minimum the AI needs to stay disciplined without
adding significant token overhead. The AI can read full NEXT_RUN.md from disk when
it needs more detail (allowed commands list, safety contract, evidence requirements).

### What Should Be Injected by Default

**Tiny context** in CLAUDE.md snippet (7 lines, always present in system prompt).

Rationale: The tiny context is always available at near-zero token cost. It gives
the AI the critical guardrails (budget, verification, stop conditions) without
requiring the AI to read a file. The full NEXT_RUN.md remains on disk for detailed
reference when the AI needs it (which it will, since CLAUDE.md tells it to read it).

### What Should Remain on Disk

**Full NEXT_RUN.md** — all 11 sections. Always generated by `akar prepare`. Always
validated by `request --check`. Always available for the AI to read via the Read
tool.

### How to Avoid Repeating Context Every Tool Call

The CLAUDE.md snippet is in the system prompt — loaded once at session start, not
repeated on every tool call. The tiny context (7 lines) adds negligible token
overhead. The full NEXT_RUN.md is only read when the AI uses the Read tool to fetch
it.

**This is the key insight:** Token waste comes from repeating context, not from
having it available. A 7-line snippet in CLAUDE.md + a 100-line file on disk that
the AI reads once is far more token-efficient than the user pasting the same context
into every prompt.

### Measurements Needed Before Claiming Token Reduction

1. Baseline: token count for a typical task session without AKAR
2. With AKAR (current): token count with user copy/pasting NEXT_RUN.md or AI reading
   it mid-session
3. With AKAR (Layer 2): token count with CLAUDE.md snippet + AI reading NEXT_RUN.md
   once
4. Comparison across task types (bugfix, feature, refactor)

**No token reduction claims should be made until these measurements exist.**

## 14. Safety Boundaries

### Future AKAR Auto-Invocation MAY

- Run AKAR-owned advisory commands (prepare, finish, status, doctor, request,
  postmortem, learn --list)
- Write AKAR-owned state files under `.akar/` (NEXT_RUN.md, DIFF_BASELINE.json,
  EVENT_LOG.jsonl, LEARNING_PATCHES.md)
- Read git status, diff, HEAD, branch
- Generate NEXT_RUN-like guidance files
- Summarize status and warnings
- Categorize dirty tree entries (AKAR-state vs source-change vs unknown)
- Ask for user confirmation when state is ambiguous (dirty tree, active split-rule,
  broken hooks)
- Append a managed snippet to CLAUDE.md (with explicit user confirmation during
  `akar init`)

### Future AKAR Auto-Invocation MUST NOT

- Run project tests automatically (except existing Rust `cargo build`/`cargo test`
  boundary in `akar verify`, which is manually invoked)
- Install project dependencies
- Edit project source code
- Edit Claude Code settings (`~/.claude/settings.json`)
- Modify `.gitignore`
- Commit, push, reset, clean, stash, checkout, merge, rebase
- Delete files outside `.akar/`
- Call model APIs
- Hide failures or errors
- Silently continue after detecting a dangerous state
- Make decisions about task completion ("task is done, auto-finish")
- Auto-resolve learning patches (human confirmation required)
- Modify hooks or hook wiring without user action

### Boundary Enforcement

- Every auto-invocation code path must check: "Am I running an AKAR advisory
  command or a project command?"
- The check must be structural (enum/match), not convention-based (string check)
- The `prepare` and `finish` commands only compose existing advisory commands
- Any addition to the auto-invocation surface requires explicit design review

### Opt-In Requirement

- All auto-invocation features default to OFF
- User must explicitly enable via `.akar/config.toml` or `akar enable`
- Toggle state is visible in `akar status` and `akar doctor`
- Disabling is immediate and complete (no lingering background state)

## 15. Future User Experience

### Current Experience (v0.43.0)

```
$ akar status                    # check state
$ akar preflight --snapshot "fix login button"  # baseline
$ akar request "fix login button"  # compile NEXT_RUN
$ akar request --check            # validate
$ # user opens NEXT_RUN.md, copies relevant parts
$ # user pastes into Claude Code prompt
$ claude                           # start AI session
  # ... AI work ...
$ akar postmortem --diff --baseline  # measure diff
$ akar learn --list               # check patches
$ akar status                     # check state
$ git add -A && git commit -m "..."  # manual commit
```

**7 commands per task. User manually bridges context into AI.**

### Future Layer 1 Experience (Prepare/Finish)

```
$ akar prepare "fix login button" --snapshot
  prepare: READY
  project: Rust | budget: 3 files, 60 LOC | verify: cargo test
  governor: READY — proceed with task
  next: start your AI session
$ # user opens Claude, tells it to read .akar/NEXT_RUN.md (or uses /akar:context)
$ claude                           # start AI session
  # ... AI work ...
$ akar finish
  finish: PASS
  diff: 2 files, 15 LOC (budget: 3 files, 60 LOC) | within limits
  next: review changes and commit
$ git add -A && git commit -m "..."
```

**2 commands per task. Context relay reduced to one instruction or slash command.**

### Future Layer 2 Experience (AI-Facing Delivery)

```
$ akar prepare "fix login button" --snapshot
  prepare: READY
  project: Rust | budget: 3 files, 60 LOC | verify: cargo test
  context: CLAUDE.md configured — Claude will read .akar/NEXT_RUN.md automatically
$ claude                           # start AI session
  # Claude reads .akar/NEXT_RUN.md automatically (CLAUDE.md instruction)
  # Tiny context (7 lines) is in system prompt via CLAUDE.md snippet
  # ... AI work ...
$ akar finish
  finish: PASS
  diff: 2 files, 15 LOC | within limits
$ git add -A && git commit -m "..."
```

**2 commands per task. Zero context relay. AI receives AKAR guidance automatically.**

### Future Layer 3+ Experience (Auto-Invocation — Not Yet Designed)

This section intentionally left as a placeholder. Auto-invocation (AKAR commands
triggered by session lifecycle events without user CLI) requires:

1. Proving that Layer 2 delivery works reliably in dogfood trials
2. Designing a session-lifecycle detection mechanism (hook-based, not daemon)
3. Proving that auto-invocation doesn't cause race conditions with AI tool calls
4. External user validation that auto-invocation is wanted

This is NOT designed in this report. It belongs in a future design report after
Layer 2 is proven.

## 16. Implementation Sequencing

Proposed future release sequence. This is a recommendation, not a commitment.
Each release should produce evidence before the next begins.

| Version | Release | Type | Description |
|---|---|---|---|
| **v0.45.0** | Burden Measurement Dogfood | Evidence | Measure exact current burden: command count, time per command, user errors, AI context relay methods, dirty-tree frequency. Use measurement data to validate or adjust the design in this report. |
| **v0.46.0** | Prepare/Finish Command Prototype | Feature | Implement `akar prepare "<task>"` and `akar finish`. No CLAUDE.md modification. No auto-invocation. Keep all existing commands. |
| **v0.47.0** | Prepare/Finish Dogfood | Evidence | Dogfood prepare/finish across all four project lanes. Measure command count reduction (expect 7→2). Identify friction in the new workflow. |
| **v0.48.0** | CLAUDE.md Snippet + Tiny Context | Feature | Implement CLAUDE.md snippet during `akar init` (opt-in). Implement tiny context tier (7 lines). Implement `.akar/config.toml` and `akar enable`/`akar disable`. |
| **v0.49.0** | AI-Facing Delivery Dogfood | Evidence | Dogfood full Layer 2: prepare/finish + CLAUDE.md snippet + tiny context. Measure whether AI auto-reads NEXT_RUN.md. Measure token impact (with/without AKAR context). |
| **v0.50.0** | Negative Behavior Measurement Baseline | Evidence | Run controlled task comparisons (with/without AKAR Layer 2). Establish baseline metrics for the North Star negative behavior targets. Publish honest measurements. |

**Total releases from current state to measured Layer 2: 6 releases (v0.45–v0.50).**

### Sequencing Rules

1. **Never skip evidence releases.** Every feature release (v0.46, v0.48) must be
   followed by a dogfood/measurement release (v0.47, v0.49) before the next feature.
2. **Never implement Layer N+1 before Layer N is proven.** CLAUDE.md snippet
   (v0.48) waits until prepare/finish is dogfooded (v0.47). Auto-invocation (future)
   waits until AI-facing delivery is proven (v0.49).
3. **Measurement before claims.** Token reduction claims wait until v0.49 evidence.
   Negative behavior reduction claims wait until v0.50 evidence.
4. **All existing commands remain available.** Consolidation adds, never removes.

## 17. What This Design Rejects

### Rejected for v1.0

| Idea | Why Rejected |
|---|---|
| Auto-execution of project code | Crosses advisory boundary; requires separate safety design |
| Auto-commit after finish | Removes human judgment from commit decision |
| Auto-wiring of Claude Code hooks | Editing user config without explicit action is hostile |
| Shell wrapper / launcher | Too fragile, platform-specific, hides AKAR's behavior |
| File watcher / daemon | Too complex, risk of race conditions, platform-inconsistent |
| Auto-resolve learning patches | Requires human judgment about whether the lesson was learned |
| Token optimization engine | No measurement baseline; premature |
| Removing existing commands | Expert users need fine-grained control; backwards compatibility |

### Rejected Entirely (Any Version)

| Idea | Why Rejected |
|---|---|
| AKAR editing Claude Code settings automatically | Violates user trust; AKAR is advisory |
| AKAR auto-committing | Removes human review step; dangerous |
| AKAR deciding "task is complete" | AKAR doesn't understand code; completion is human/AI judgment |

## 18. Evidence Needed Before Implementation

Before implementing any part of this design, these measurements are needed:

### From v0.45.0 Burden Measurement Dogfood

1. **Exact command count per task cycle** — not estimated, measured across 5+ real
   tasks with timestamps
2. **Time per command** — which commands are slow? Where is the friction?
3. **User error rate** — how often are commands run out of order, forgotten, or
   re-run due to mistakes?
4. **Context relay method** — in what fraction of sessions does the AI actually
   read NEXT_RUN.md? How does it discover it (user prompt vs filesystem exploration)?
5. **Dirty-tree frequency** — how often does `.akar/` state cause dirty-tree
   refusal on preflight --snapshot?
6. **Hook wiring time** — how long does a new user take from `akar hooks --install`
   to working PreToolUse hooks?

### From v0.47.0 Prepare/Finish Dogfood

1. **Command count reduction** — does prepare/finish actually reduce the cycle to
   2 commands?
2. **User errors** — are users confused by the consolidated commands?
3. **Missed steps** — does consolidation cause users to skip postmortem or learning
   patches?
4. **Cross-lane compatibility** — does prepare/finish work correctly for Rust, Node,
   Python, and Unknown projects?

### From v0.49.0 AI-Facing Delivery Dogfood

1. **AI auto-read rate** — in what fraction of sessions does Claude read
   NEXT_RUN.md without user prompting?
2. **Token impact** — does the CLAUDE.md snippet + tiny context increase or decrease
   total token usage?
3. **Context accuracy** — does the AI follow the budget and stop conditions from
   NEXT_RUN.md?

## 19. Recommended Next Release

**v0.45.0 Burden Measurement Dogfood.**

### Why Measurement Before Implementation

The design in this report is well-reasoned but unmeasured. Before implementing
`akar prepare` and `akar finish`, AKAR needs baseline data:

- How many commands does the user actually run per task? (Estimated 7, but not
  measured across real sessions.)
- How long does each command take?
- Where is the real friction — command count, context relay, or dirty-tree handling?
- Is the AI reliably reading NEXT_RUN.md from disk today, or is the user always
  prompting it?

If measurement shows that context relay is the dominant friction (user always has
to tell Claude to read NEXT_RUN.md), then CLAUDE.md snippet design (v0.48) should
be accelerated. If measurement shows that command count is the dominant friction
(user forgets steps, runs commands out of order), then prepare/finish (v0.46) is
correct. The measurement tells us which to prioritize.

### What v0.45.0 Should Produce

A measurement report (`docs/audits/AKAR_V0_45_BURDEN_MEASUREMENT_DOGFOOD.md`)
containing:

1. Command count per task across 5+ real tasks
2. Time per command
3. User error taxonomy (forgotten steps, wrong order, repeated commands)
4. Context relay method used in each session (user copy/paste, user prompt, AI
   auto-discovery)
5. Dirty-tree frequency and resolution time
6. Adjusted burden reduction priorities based on measured data
7. Confirmation or revision of the v0.44.0 design

### Alternative: v0.45.0 Prepare/Finish Prototype

Only choose this if the design is considered so obviously correct that measurement
can happen during the prototype dogfood (v0.47.0). The risk is building the wrong
consolidation — e.g., if dirty-tree friction dominates, prepare/finish won't fix it.

## 20. Honest Conclusion

AKAR v0.43.0 identified a manual-CLI trap: the user runs 7+ commands per task,
manually bridging AKAR output into the AI session. This v0.44.0 design defines a
concrete, safe path out of the trap.

The path has three phases:

1. **Command consolidation** — `akar prepare "<task>"` and `akar finish` reduce
   the per-task cycle from 7 commands to 2, by composing existing advisory commands.
   This is low-risk because every operation is already manual today.

2. **AI-facing context delivery** — a managed CLAUDE.md snippet instructs Claude
   to read `.akar/NEXT_RUN.md` automatically. This eliminates the user relay step
   without auto-execution, without model calls, and without editing Claude settings
   secretly. The snippet is opt-in, clearly delimited, and removable.

3. **Auto-invocation** — future work (not designed here) to trigger AKAR advisory
   commands at session lifecycle moments.

All three phases preserve AKAR's core guarantee: AKAR never executes project code.
The AI model remains the worker. AKAR enhances the AI's work by delivering
discipline-tuned context, not by doing the work itself.

The most important design decision in this report is the CLAUDE.md snippet. It is
the smallest mechanism with the highest leverage — one block of markdown in a file
Claude already reads, instructing Claude to read a file AKAR already writes. That
single change transforms AKAR from "manual checklist" to "enhancement layer" without
changing AKAR's safety model at all.

The next step is measurement. Before building, measure the exact current burden so
the design can be validated or adjusted with evidence. That is v0.45.0.
