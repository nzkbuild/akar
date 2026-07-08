# AKAR v0.43.0 — North Star Drift and Gap Assessment

## 1. Executive Verdict

**AKAR has built useful foundation but drifted into a manual-CLI trap that contradicts
the North Star.** The current stable advisory alpha does meaningful work (project
detection, safety hooks, diff budgets, verification discovery) but requires 7+ manual
CLI commands per task cycle — the user operates AKAR rather than AKAR operating
underneath the AI tool. The North Star says the user should "install AKAR and enable
it inside their preferred CLI/TUI AI tool" with "auto-run / AKAR enhancement switched
on or off." Current AKAR requires the opposite: the user manually invokes AKAR before
and after every AI interaction. Six speculative ideas (capsules, token optimization,
Codex/OpenCode adapters, skill resolver, autopilot, memory engine) reflect an
understandable desire to close the gap, but none are justified by current evidence and
several would cross the advisory→autonomous boundary before the foundation is ready.

The path forward is not to build capsules or autopilot. It is to solve the
AI-facing delivery problem — how AKAR's discipline-tuned output reaches the AI session
without user manual relay, and without executing project code. This is a design
problem, not a feature problem.

## 2. Current AKAR vs. North Star

### A. Current AKAR (v0.42.0)

A local Rust CLI discipline layer that the **user manually operates** before and after
each AI coding task. It prepares NEXT_RUN.md, measures diffs, surfaces verification
guidance, and blocks dangerous commands through optional hooks. It never executes
project code, never calls models, and never auto-injects its guidance into AI sessions.
All value depends on user diligence — the user runs 7+ CLI commands per task cycle.

### B. Intended AKAR North Star

A **root/runtime enhancement layer** that sits underneath CLI/TUI AI coding tools
(Claude Code, Codex, OpenCode). The user installs it once, enables it inside their
preferred tool, and AKAR enhances the AI's work automatically — reducing waste (tokens,
requests, bad code, hallucinations, dangerous commands) and increasing quality (better
task understanding, safer execution, cleaner diffs, better verification) without the
user running manual commands. The AI model remains the worker; AKAR is the enhancement
layer. Auto-run can be toggled on/off. Cheap models should work better with AKAR;
expensive models should waste less.

**These are not the same thing.** Current AKAR is a manual CLI toolkit. The North Star
is an automated enhancement layer. The gap between them is large and honest.

## 3. Original North Star Doctrine

```
- AKAR means root.
- AKAR should solve the root of AI work problems by understanding the root first.
- AKAR should help both the user and the AI model.
- AKAR should not be a manual checklist the user repeatedly operates.
- User installs AKAR and enables it inside their preferred CLI/TUI AI tool.
- Auto-run / AKAR enhancement can be switched on or off.
- The AI model remains the worker, builder, planner, and executor.
- AKAR is the root/runtime enhancement layer underneath tools like Claude Code,
  Codex, OpenCode, or other CLI/TUI coding agents.
- AKAR should clarify the job between user and AI without creating extra fuss.
- AKAR should reduce negative AI work patterns:
  - wasted tokens, wasted requests, wasted input/output
  - bad code, crazy LOC, unclear diffs
  - hallucination, context loss, memory overload
  - dangerous commands, half-baked/slop work
  - repeated user reprompting
- AKAR should increase positive work patterns:
  - better task understanding, safer execution
  - clearer planning, better verification
  - smaller and cleaner diffs
  - better model self-awareness of repo/prompt/environment
  - better use of cheap models, cheaper use of expensive models
- The theoretical goal is synthetic enhancement:
  AKAR should increase positive results and reduce negative results across AI work,
  similar to an enhancement layer that improves benchmark-like outcomes without
  replacing the AI model.
```

## 4. Baseline and Verification

| Check | Result |
|---|---|
| Commit | `5b0cbfa` — docs: record AKAR current reality re-grounding report |
| Version | `akar 0.42.0` |
| Working tree | clean |
| `cargo test` | 508 passed, 0 failed |
| `cargo run -- eval` | 28/28 PASS |
| `cargo run -- doctor` | PASS (1 WARN: active split-rule) |
| `cargo run -- status` | HEALTHY, READY |
| `cargo run -- request "..."` | NORMAL mode |
| `cargo run -- request --check` | PASS (4/4) |
| `cargo run -- governor` | SPLIT_TASK (known split-rule artifact) |
| `cargo run -- learn --list` | 8 entries (1 active, 7 resolved) |
| `cargo run -- hooks --check` | PASS (source-tree) |

## 5. Evidence Reviewed

All relevant audit reports were reviewed:

| Report | Version | Status |
|---|---|---|
| Stable Advisory Alpha Freeze | v0.34 | Read — defines current stage and boundaries |
| Anchored Live Hook Dogfood | v0.36 | Read — confirms hook integration for Node/Windows |
| Python External Dogfood | v0.37 | Read — confirms project-kind awareness for Python |
| Unknown Discovery Dogfood | v0.39 | Read — confirms verification discovery hints |
| Multi-Task Session Dogfood | v0.40 | Read — confirms state transitions across 3 tasks |
| Fresh-User Wording Polish | v0.41 | Read — confirms cosmetic friction reduction |
| Current Reality Re-Grounding | v0.42 | Read in full — comprehensive current-state baseline |

All reports present. No gaps in audit trail.

## 6. North Star Gap Matrix

Each dimension scored against the North Star target. Gap sizes: None / Small / Medium / Large / Huge.

| # | Dimension | North Star Target | Current AKAR Status | Gap | Drift Type | Next Evidence Needed |
|---|---|---|---|---|---|---|
| 1 | **Manual user invocation** | User installs once, enables in AI tool, AKAR runs automatically | User runs 7+ manual CLI commands per task cycle | **Huge** | Manual-CLI trap | Design for auto-invocation without auto-execution; measure command count reduction |
| 2 | **Auto-enable inside AI tool** | AKAR works inside Claude Code / Codex / OpenCode as enhancement layer | AKAR is a separate CLI; user bridges gap manually | **Huge** | Manual-CLI trap | Host enablement design (not adapter code); single-tool integration prototype |
| 3 | **AI-facing guidance delivery** | AKAR context auto-injected into AI session | NEXT_RUN.md written to disk; AI may discover it from filesystem or user relays it | **Large** | Manual-CLI trap | Mechanism for AKAR output to reach AI context without user copy/paste |
| 4 | **Token/input reduction** | AKAR reduces wasted tokens and input size | No token measurement or optimization exists | **Huge** | Not started | Token measurement baseline; comparison of session token counts with/without AKAR guidance |
| 5 | **Output verbosity reduction** | AKAR helps AI produce less verbose output | No output control mechanism exists | **Huge** | Not started | Output length measurement; whether structured NEXT_RUN reduces AI verbosity |
| 6 | **Request count reduction** | Fewer user prompts needed per task | CLI is manual; no auto-invocation; user reprompts manually | **Huge** | Manual-CLI trap | Count of user prompts per task with/without AKAR; auto-invocation design |
| 7 | **Bad code reduction** | AKAR reduces low-quality output | No code quality measurement exists; budgets measure quantity not quality | **Large** | Not started | Quality metrics (test pass rate, review findings); comparison with/without AKAR |
| 8 | **LOC/diff discipline** | Smaller, cleaner diffs | Budgets measure diffs but don't enforce; split-rule is advisory only | **Medium** | Useful foundation | Enforcement mechanism that doesn't require user manual invocation |
| 9 | **Verification discipline** | Better verification, AI knows how to verify | Verification discovery surfaces commands; NEXT_RUN includes them; but user/AI must act | **Medium** | Useful foundation | Auto-surfaced verification in AI context; measurement of verification completion rate |
| 10 | **Hallucination reduction** | Fewer invented commands, fewer false facts | Project-kind-aware commands prevent toolchain hallucination; proven in dogfood | **Small** | Useful foundation | Cross-platform and multi-language validation |
| 11 | **Context/memory handling** | Better context retention, less memory overload | Memory files exist but passive; no auto-load; no optimization | **Large** | Not started | Whether AI reads and uses AKAR memory files; auto-injection of relevant memory |
| 12 | **Environment awareness** | AI knows repo kind, tools, constraints | Project detection works; NEXT_RUN includes context; but user must relay it | **Medium** | Useful foundation (partial), Manual-CLI trap (delivery) | Auto-injection of project context into AI session |
| 13 | **Dangerous command prevention** | AKAR blocks dangerous commands | PreToolUse hooks proven on Node/Windows; blocks Critical commands | **Small** | Useful foundation | Cross-platform hook validation; all project lanes with live hooks |
| 14 | **Skill/tool choice** | AKAR helps AI choose right tools | Skill registry is inventory-only; no selection or recommendation | **Huge** | Not started (speculative drift in planning) | Evidence that wrong tool choice causes measurable harm; resolver design |
| 15 | **Cheap-model uplift** | Cheap AI works better with AKAR | No benchmark comparing model quality with/without AKAR | **Huge** | Not started | Controlled benchmark: same tasks, different models, with/without AKAR |
| 16 | **Expensive-model cost efficiency** | Expensive AI achieves quality with less waste | No cost or token measurement exists | **Huge** | Not started | Token cost comparison with/without AKAR guidance |
| 17 | **Cross-platform support** | Works on Linux, macOS, Windows | All dogfood trials on Windows only | **Medium** | Useful foundation (code is portable), not started (validation) | Linux and macOS dogfood trials |
| 18 | **Host/tool compatibility** | Works with Claude Code, Codex, OpenCode | Claude Code PreToolUse hooks only; no other host integration | **Large** | Speculative drift (adapters planned without evidence) | Single additional host integration with measured benefit |
| 19 | **User burden reduction** | Less user effort, not more | Current AKAR adds 7+ commands per task; user burden is higher with AKAR than without | **Huge** | Manual-CLI trap | Command count per task with/without AKAR; auto-invocation design |
| 20 | **Professional final output quality** | Better final code, tests passing, clean diffs | Dogfood trials show clean diffs within budget; but only on tiny fixtures | **Medium** | Useful foundation | Large-repo trials; multi-file refactors; external user results |

### Summary Count

| Gap Size | Count | Dimensions |
|---|---|---|
| **Huge** | 9 | Manual invocation, auto-enable, token reduction, output verbosity, request count, bad code reduction, skill/tool choice, cheap-model uplift, expensive-model efficiency, user burden reduction |
| **Large** | 4 | AI-facing delivery, context/memory, host compatibility |
| **Medium** | 5 | LOC/diff discipline, verification discipline, environment awareness, cross-platform, final output quality |
| **Small** | 2 | Hallucination reduction, dangerous command prevention |

**9 of 20 dimensions are Huge gaps.** All 9 are either Manual-CLI trap or Not started.
The North Star vision of AKAR as an automated enhancement layer is almost entirely
unbuilt.

## 7. Useful Foundation Work

These are the parts of current AKAR that genuinely move toward the North Star:

### Safety Hooks (v0.10–v0.36)
- **What was built:** PreToolUse hook templates, embedded in binary, `akar safety`
  command classifier, HOOK_EVENTS telemetry, session-anchoring fix
- **Why it helps:** The North Star says AKAR should block dangerous commands. Hooks do
  this today — proven on Node/Windows with anchored session.
- **North Star alignment:** Direct. Dangerous command prevention is a North Star goal.
- **What to keep:** Safety classifier, hook templates, embedded fallback, HOOK_EVENTS
  logging.
- **Gap remaining:** Cross-platform validation, all project lanes with live hooks.

### Project Detection (v0.31)
- **What was built:** Shared `project_detection.rs`, marker-file based (Rust/Node/
  Python/Unknown)
- **Why it helps:** Environment awareness is a North Star goal. AKAR knows what kind
  of project it's looking at without AI guesswork.
- **North Star alignment:** Direct. Better model self-awareness of repo/environment.
- **What to keep:** Detection module, marker-file logic, ProjectKind enum.
- **Gap remaining:** Auto-injection of project context into AI session.

### Verification Discovery (v0.38)
- **What was built:** File-system scan for verification commands, confidence levels,
  blocklist safety filter, surfaced in doctor/NEXT_RUN/verify
- **Why it helps:** "Better verification" and "less verification guessing" are North
  Star goals.
- **North Star alignment:** Direct. Surfaces real commands without hallucination.
- **What to keep:** Discovery module, confidence levels, safety blocklist.
- **Gap remaining:** Auto-surfacing in AI context; measurement of verification
  completion rate.

### Diff Budgets (v0.10–v0.28)
- **What was built:** Preflight baseline snapshots, postmortem diff measurement,
  budget limits in NEXT_RUN, split-rule learning patches
- **Why it helps:** "Smaller and cleaner diffs" is a North Star goal. Budgets provide
  measurement.
- **North Star alignment:** Direct but incomplete. North Star wants enforcement, not
  just measurement.
- **What to keep:** Baseline mechanism, diff measurement, budget limits.
- **Gap remaining:** Auto-enforcement without user manual invocation.

### NEXT_RUN Task Threading (v0.26)
- **What was built:** Task text threaded through NEXT_RUN.md, project-kind-aware
  allowed commands, 11-section structural contract, `request --check` validator
- **Why it helps:** "Better task understanding" and "clearer planning" are North Star
  goals. NEXT_RUN is the best discipline artifact AKAR produces.
- **North Star alignment:** Direct. The content is exactly what the AI needs.
- **What to keep:** NEXT_RUN compiler, structural contract, validator.
- **Gap remaining:** The content doesn't auto-reach the AI. This is the critical gap.

### Doctor/Status (v0.23, v0.32)
- **What was built:** Honest environment checks, project-kind awareness, sectioned
  report, severity levels (PASS/WARN/FAIL)
- **Why it helps:** Environment awareness and honest diagnostics.
- **North Star alignment:** Direct. Tool should honestly report its own state.
- **What to keep:** Doctor checks, status summary.
- **Gap remaining:** Doctor requires manual invocation.

### Postmortem (v0.21–v0.28)
- **What was built:** Diff measurement against baseline, budget comparison, learning
  patch generation
- **Why it helps:** Evidence collection and learning from failures.
- **North Star alignment:** Indirect but useful. Evidence enables future optimization.
- **What to keep:** Postmortem measurement, learn patch generation.
- **Gap remaining:** Auto-trigger after task completion.

### Dogfood Reports (v0.24–v0.42)
- **What was built:** 9 audit reports documenting real trials with evidence
- **Why it helps:** Honest evidence prevents self-deception about what works.
- **North Star alignment:** Indirect but essential. Without evidence, AKAR would be
  building in the dark.
- **What to keep:** Audit discipline, evidence requirements, honest limitation
  documentation.
- **Gap remaining:** External user evidence, cross-platform evidence, large-repo
  evidence.

### Fresh-User Wording Polish (v0.41)
- **What was built:** Three cosmetic wording fixes for first-run friction
- **Why it helps:** Reduces confusion that wastes user time.
- **North Star alignment:** Indirect. "Without creating extra fuss."
- **What to keep:** Improved wording, fresh-project severity changes.
- **Gap remaining:** The underlying manual-CLI problem wasn't addressed.

## 8. Manual-CLI Trap

The single biggest drift from the North Star. AKAR has become a manual checklist the
user repeatedly operates — exactly what the North Star says it should not be.

### Symptoms
- **7+ commands per task cycle:** `preflight --snapshot` → `request` → `request --check`
  → inspect NEXT_RUN → (AI session) → `postmortem --diff --baseline` → `learn --list`
  → `status`
- **User manually bridges output:** NEXT_RUN.md content reaches the AI only through
  user copy/paste or AI filesystem discovery
- **User manually wires hooks:** Editing `~/.claude/settings.json` by hand
- **User manually decides .akar/ tracking:** `.gitignore` policy is left to the user
  with advisory text only
- **Governor is passive:** Produces decisions but doesn't act on them
- **No auto-invocation:** Nothing triggers AKAR automatically before/after AI work
- **No daemon/watch mode:** AKAR is a fire-and-exit CLI, not a persistent enhancement

### How This Happened

The manual-CLI approach was a necessary first step — you can't automate what you
haven't proven manually. But the project stayed in manual mode too long. Each
dogfood trial proved the advisory loop worked, but none drove the question: "how do
we make this automatic?" The v0.34 stable advisory alpha freeze codified the manual
CLI as the product rather than as a stepping stone.

### Why This Contradicts the North Star

The North Star says:
- "AKAR should not be a manual checklist the user repeatedly operates"
- "User installs AKAR and enables it inside their preferred CLI/TUI AI tool"
- "Auto-run / AKAR enhancement can be switched on or off"

Current AKAR does the opposite on all three counts. The user operates AKAR; AKAR
does not operate underneath the AI tool; auto-run does not exist.

### What to Do

The manual-CLI trap must be addressed before any speculative features. The fix is
not to add more CLI commands (dirty-tree diagnostic, better postmortem, etc.) but
to design how AKAR output reaches the AI session without user relay. This is a
design problem, not a feature problem. See section 16 (Distance-to-North-Star layers)
and section 23 (Recommended next release).

## 9. Speculative Drift

Six ideas emerged in recent planning that are not justified by current evidence:

### Capsules
- **What was proposed:** Self-contained task units with auto-execution
- **Drift type:** Speculative. Crosses advisory→autonomous boundary.
- **Why it emerged:** Desire to reduce manual CLI overhead by packaging AKAR workflow
  into a single unit
- **Why it's drift:** No evidence that packaging the workflow into a "capsule" reduces
  burden more than auto-invocation of existing commands. Auto-execution adds risk
  without proven need.
- **Verdict:** Postpone. Solve AI-facing delivery first; capsule may become a
  packaging format for auto-delivered context, not an execution unit.

### Token Optimization
- **What was proposed:** Automatic context window management, token-aware trimming
- **Drift type:** Speculative. No token measurement exists.
- **Why it emerged:** North Star mentions reducing wasted tokens
- **Why it's drift:** You can't optimize what you don't measure. AKAR has zero
  visibility into token consumption. Optimization requires model API access or
  session introspection that AKAR doesn't have.
- **Verdict:** Postpone until token measurement baseline exists.

### Codex/OpenCode Host Adapters
- **What was proposed:** Multi-platform coding agent support
- **Drift type:** Speculative. Zero evidence of multi-agent demand.
- **Why it emerged:** North Star mentions Codex and OpenCode as target hosts
- **Why it's drift:** Claude Code integration is barely proven (Node only, Windows
  only). Adding a second host before the first is solid is premature. No user has
  asked for this.
- **Verdict:** Postpone. Make Claude Code integration solid first; adapter design
  can happen after external users validate the single-host model.

### Skill Conflict Resolver
- **What was proposed:** Automatic skill selection and conflict resolution
- **Drift type:** Speculative. Skill registry is inventory-only.
- **Why it emerged:** AKAR has a SKILL_INVENTORY.md but doesn't help the AI use it
- **Why it's drift:** Skill conflict resolution requires semantic understanding of
  skills and tasks that AKAR doesn't model. The skill registry is a project memory
  file, not an active system.
- **Verdict:** Postpone. Skill registry is useful as passive inventory; active
  resolution requires AI-level understanding that belongs in the AI model, not AKAR.

### Bounded Autopilot
- **What was proposed:** Limited autonomous task execution (auto preflight → request
  → postmortem)
- **Drift type:** Speculative. Crosses advisory→autonomous boundary.
- **Why it emerged:** Desire to reduce manual CLI overhead
- **Why it's drift:** Autopilot implies AKAR deciding when to snapshot and measure —
  crossing from advisory to executive. The right fix for manual overhead is
  auto-invocation of existing advisory commands, not autonomous decision-making.
- **Verdict:** Postpone. Auto-invocation (triggering existing advisory commands
  automatically) is different from auto-execution (making decisions and running
  project code). Do auto-invocation first.

### Memory Optimization Engine
- **What was proposed:** Cross-session knowledge compression and retrieval
- **Drift type:** Speculative. Memory files are passive.
- **Why it emerged:** North Star mentions reducing memory overload
- **Why it's drift:** Memory files exist but no evidence the AI reads them.
  Optimization before injection is backwards.
- **Verdict:** Postpone. Prove AI reads memory files first; then optimize.

## 10. Negative Behavior Reduction Gap

The North Star lists 12 negative AI work patterns to reduce. Here is the honest state:

| North Star Target | Current Status | Mechanism | Gap |
|---|---|---|---|
| Wasted tokens | Not addressed | None | Huge |
| Wasted requests | Not addressed | None | Huge |
| Wasted input/output | Not addressed | None | Huge |
| Bad code | Not measured | None | Large |
| Crazy LOC | Partially addressed | Diff budgets (advisory) | Medium |
| Unclear diffs | Partially addressed | Postmortem measurement | Medium |
| Hallucination | Addressed for toolchain commands | Project-kind awareness | Small |
| Context loss | Not addressed | None | Large |
| Memory overload | Not addressed | None | Large |
| Dangerous commands | Addressed for Node/Windows | PreToolUse hooks | Small |
| Half-baked/slop work | Not addressed | None | Large |
| Repeated user reprompting | Not addressed | CLI is manual | Huge |

**Current AKAR addresses 3 of 12 targets with Small/Medium gaps.** The remaining 9
have Large or Huge gaps. The two biggest wins (dangerous commands, hallucination
reduction) are real but narrow. The broadest targets (token waste, request waste,
repeated reprompting) are completely unaddressed.

## 11. Token/Request Waste Gap

**Current state: AKAR has zero visibility into token consumption or request counts.**

- No API integration — AKAR doesn't call models
- No session introspection — AKAR doesn't see Claude Code's token usage
- No measurement baseline — no data on tokens per task with/without AKAR
- No optimization mechanism — even if measured, AKAR can't trim context

**Why this matters for the North Star:** "Wasted tokens" and "wasted requests" are
the first two negative patterns listed. They are completely unaddressed.

**What would be needed to close this gap:**
1. A way to measure tokens per AI session (likely through Claude Code session
   metadata or API billing data, not through AKAR itself)
2. A baseline comparison: same task with and without AKAR NEXT_RUN guidance
3. A mechanism to reduce token waste (likely through better context compression
   in NEXT_RUN, not through model-level optimization)

**Honest assessment:** Token/request waste reduction is a Layer 4–5 concern (see
section 16). It requires AI-facing delivery (Layer 2) and measurement (Layer 3)
before optimization (Layer 4). It cannot be meaningfully addressed today.

## 12. AI-Facing Delivery Gap

**This is the critical architectural gap.** AKAR produces excellent discipline-tuned
content (NEXT_RUN.md) but has no mechanism to deliver it into the AI session.

Current delivery paths:
1. **User copy/paste:** User reads NEXT_RUN.md and pastes relevant parts into
   Claude Code prompt. High burden, unreliable.
2. **AI filesystem discovery:** Claude Code can read `.akar/NEXT_RUN.md` from the
   filesystem if prompted to or if it discovers it. Works in dogfood trials but
   depends on AI initiative.
3. **Hook-mediated injection:** Not built. PreToolUse hooks could theoretically
   prepend context to tool calls, but this is not implemented.

**What the North Star requires:** "AKAR is the root/runtime enhancement layer
underneath tools like Claude Code." This implies AKAR context is present in the AI
session without user action — either through automatic file injection, hook-mediated
context prepending, or a session initialization mechanism.

**What must be true for AI-facing delivery:**
- AKAR context reaches the AI session without user copy/paste
- The delivery mechanism does not execute project code
- The delivery mechanism works within Claude Code's existing extension points
  (CLAUDE.md, hooks, settings.json)
- The user can toggle it on/off ("auto-run / AKAR enhancement can be switched on
  or off")

**This is the most important unsolved problem in AKAR.** It is the prerequisite for
addressing token waste, request waste, context loss, memory overload, and user
burden reduction. Without AI-facing delivery, AKAR is a manual CLI toolkit. With it,
AKAR becomes an enhancement layer.

## 13. User Burden Gap

**Current AKAR increases user burden rather than reducing it.**

Before AKAR: User opens Claude Code, describes task, reviews output.
After AKAR: User runs 7+ CLI commands, inspects files, manually relays context,
then opens Claude Code, describes task with AKAR context, reviews output, runs
more CLI commands.

The North Star says AKAR should "clarify the job between user and AI without
creating extra fuss." Current AKAR creates extra fuss — the fuss of manual CLI
operation. The value (discipline, safety, diff measurement) is real, but the
cost in user attention is high.

**What v0.41 improved:** Cosmetic wording reduced "am I doing something wrong?"
confusion. But it didn't reduce the number of commands or the manual relay burden.

**What must change:** The user should run `akar init` once per project, enable
auto-run, and then interact with Claude Code normally. AKAR should prepare context
before sessions, measure diffs after sessions, and surface warnings during sessions —
all without the user running individual CLI commands. This requires AI-facing
delivery (section 12).

## 14. Model-Quality Uplift Gap

**The North Star's most ambitious claim is completely unmeasured.**

"Synthetic enhancement" — that AKAR improves benchmark-like outcomes across AI
models — has zero evidence. No controlled comparison exists:
- No tasks run with and without AKAR
- No cheap-model vs expensive-model comparison
- No quality metrics beyond "did the tests pass?"
- No measurement of code quality, review findings, or bug count

This is not a criticism — it's honest. You cannot claim uplift without measurement,
and measurement requires AI-facing delivery first (so AKAR guidance is consistently
present in the AI session).

**Realistic timeline:** Model-quality uplift validation is a Layer 5–6 concern
(see section 16). It requires: AI-facing delivery (Layer 2) → measured negative
behavior reduction (Layer 3) → host-enabled runtime (Layer 4) → benchmark design
and execution (Layer 5). This is multiple releases away.

## 15. Safety and Trust Gap

**AKAR's safety model is conservative and proven but narrow.**

Proven:
- PreToolUse hooks block Critical commands on Node/Windows
- Safety classifier correctly categorizes Safe/Medium/High/Critical
- Hook events correctly route to session-root project

Not proven:
- Cross-platform hook behavior (Linux/macOS)
- Hook behavior with Rust/Python/Unknown projects in live sessions
- Hook behavior under high-frequency tool calls (>100/ session)
- Whether the hook model works with other AI tools (Codex, OpenCode)

**Trustworthiness:** AKAR's safety model is trustworthy within its proven scope
(Node/Windows). Expanding scope requires evidence, not assumption. The blocklist
approach is sound but incomplete — it blocks known-dangerous patterns but doesn't
understand intent.

**What the North Star requires:** "Safer execution" as a positive pattern.
Current AKAR delivers this within a narrow scope. Broadening scope is a
cross-platform validation problem, not an architectural problem.

## 16. Distance-to-North-Star Layers

Rather than a vague roadmap, here are concrete milestone layers from current state
to North Star. Each layer has a definition, required capabilities, evidence
requirements, and exit criteria.

### Layer 0 — Current Stable Advisory Alpha (v0.42.0)

**Definition:** Manual CLI discipline layer. User operates AKAR before/after AI
sessions. AKAR prepares NEXT_RUN.md, measures diffs, blocks dangerous commands
through optional hooks.

**Capabilities present:**
- Project detection (4 kinds)
- Safety classification and hook templates
- NEXT_RUN compilation with project-kind-aware commands
- Diff baseline and postmortem measurement
- Verification discovery hints
- Governor decisions (advisory only)
- Learning patches
- 508 tests, 28 eval cases
- 6 dogfood trials across 4 project lanes

**Exit criteria for Layer 0:** Already met — this is current state.

### Layer 1 — Friction-Reduced Advisory Alpha

**Definition:** Same advisory model, but fewer manual commands. User still runs AKAR
manually but with less friction. Dirty-tree situations have guided recovery.

**Required capabilities:**
- Dirty-tree diagnostic command (categorize entries, provide guidance)
- Reduced command count per task cycle (consolidate preflight+request, or
  postmortem+learn)
- Improved hook wiring guidance (possibly a setup wizard or validation command)

**Required evidence:**
- Command count per task cycle measured before and after
- Dirty-tree recovery time measured before and after
- At least one external user completes the loop with reduced friction

**Estimated releases:** 2–3 (e.g., v0.44 dirty-tree recovery, v0.45 command
consolidation, v0.46 hook wiring simplification)

**What must not be added yet:** Auto-invocation, auto-execution, token optimization,
capsules, adapters.

**Exit criteria:** User runs ≤4 commands per task cycle (down from 7+). Dirty-tree
situations resolved in <2 minutes without manual file inspection.

### Layer 2 — AI-Facing Delivery Without Auto-Execution

**Definition:** AKAR context reaches the AI session without user manual relay, but
AKAR still never executes project code. This is the critical architectural
transition from manual CLI to enhancement layer.

**Required capabilities:**
- Mechanism for NEXT_RUN.md (or equivalent) content to auto-inject into AI session
  context (CLAUDE.md injection, session initialization hook, or filesystem path
  the AI always reads)
- Auto-invocation of advisory commands (preflight before task, postmortem after
  task) triggered by session lifecycle events, not user CLI
- Toggle mechanism: auto-run on/off
- AKAR still never executes project code, never calls models

**Required evidence:**
- AI session receives AKAR context without user copy/paste in ≥90% of sessions
- User command count drops to ≤1 per task cycle (toggle on/off only)
- No regression in safety (hooks still block dangerous commands)
- No regression in diff discipline (postmortem still measures correctly)
- Dogfood trial: full task cycle with AI-facing delivery and zero manual CLI
  commands

**Estimated releases:** 3–5 (this is the hardest layer — it requires design
exploration, prototype, and validation)

**What must not be added yet:** Auto-execution of project code, model API calls,
autopilot decision-making, token optimization, capsules.

**Exit criteria:** User types `akar enable` once per project. AKAR context appears
in AI sessions automatically. User never runs `akar preflight`, `akar request`, or
`akar postmortem` manually. AKAR still never executes project code.

### Layer 3 — Measured Negative-Behavior Reduction

**Definition:** With AI-facing delivery in place, measure whether AKAR actually
reduces negative AI behaviors. Establish baseline metrics and compare with/without
AKAR.

**Required capabilities:**
- Session measurement: task success rate, files touched, LOC changed, verification
  completion
- Comparison framework: same tasks with and without AKAR guidance
- Evidence collection: structured before/after data

**Required evidence:**
- At least 10 controlled task comparisons (with vs without AKAR)
- Measurable reduction in at least 5 of the 12 North Star negative patterns
- No measured increase in any negative pattern
- External user data, not just author dogfood

**Estimated releases:** 2–3 (measurement infrastructure + trials)

**What must not be added yet:** Claiming uplift without evidence, token optimization
without measurement baseline.

**Exit criteria:** Published evidence showing AKAR reduces negative AI behaviors
in controlled comparisons. Specific metrics with before/after numbers.

### Layer 4 — Host-Enabled Runtime Mode

**Definition:** AKAR works as a runtime enhancement within at least one AI tool
(Claude Code), not just as a pre/post session CLI. Hook integration is seamless.
Cross-platform validation is complete.

**Required capabilities:**
- Seamless Claude Code integration: hook wiring is automatic or one-command
- All four project lanes proven with live hooks
- Cross-platform validation complete (Linux, macOS, Windows)
- Multi-task sessions of 5+ tasks proven

**Required evidence:**
- Live hook dogfood trials: Rust, Python, Unknown with hooks (currently only Node)
- Cross-platform dogfood: at least one full trial each on Linux and macOS
- External user: at least 3 users complete the full loop with live hooks
- Large-repo trial: at least one trial on a repo with >100 files

**Estimated releases:** 3–4

**Exit criteria:** AKAR is proven as a runtime enhancement for Claude Code on all
three platforms across all four project lanes with at least 3 external users.

### Layer 5 — Model-Quality Uplift Validation

**Definition:** Controlled benchmark comparing AI outcomes with and without AKAR,
across cheap and expensive models.

**Required capabilities:**
- Benchmark suite: standardized tasks, metrics, comparison framework
- Model access: ability to run same tasks with different models
- Statistical rigor: enough trials to distinguish signal from noise

**Required evidence:**
- Cheap model + AKAR ≥ cheap model alone (on task success, verification, diff size)
- Expensive model + AKAR ≤ expensive model alone (on token cost, request count)
  while maintaining or improving quality
- Results published with methodology, raw data, and limitations

**Estimated releases:** 3–5 (benchmark design, execution, analysis, publication)

**Exit criteria:** Published benchmark with statistically significant results
showing AKAR improves cheap-model outcomes and/or reduces expensive-model waste.

### Layer 6 — North Star Beta

**Definition:** AKAR is a root/runtime enhancement layer for AI coding tools. The
user installs it, enables it, and AKAR enhances AI work automatically. Cheap models
work better; expensive models waste less. The enhancement is measurable and proven.

**Required capabilities:**
- All Layer 0–5 capabilities present and proven
- At least one additional host integration explored (Codex or OpenCode) if evidence
  supports it
- Auto-run is default-on for new projects
- Memory/context optimization based on Layer 3 measurement data

**Required evidence:**
- 50+ external users with measured outcomes
- Published benchmark with peer review
- Cross-host evidence if multi-host support exists
- No safety incidents attributable to AKAR auto-invocation

**Exit criteria:** AKAR is no longer "advisory alpha." It is a "runtime enhancement
layer beta" with published evidence of negative-behavior reduction and model-quality
uplift across multiple hosts and platforms.

### Layer Summary

| Layer | Name | Estimated Releases | Key Risk |
|---|---|---|---|
| 0 | Stable advisory alpha | Current | None — already here |
| 1 | Friction-reduced advisory | 2–3 | Scope creep into Layer 2 too early |
| 2 | AI-facing delivery | 3–5 | Design risk — may require Claude Code changes |
| 3 | Measured reduction | 2–3 | Data collection burden; may show smaller effect than hoped |
| 4 | Host-enabled runtime | 3–4 | Cross-platform surface area; external user recruitment |
| 5 | Model-quality validation | 3–5 | Benchmark design complexity; statistical rigor |
| 6 | North Star beta | 3–5 | Integration across all layers simultaneously |

**Total estimated releases from current state to North Star beta: 16–25 releases**
(roughly v0.43 through v0.60–v0.70 range, depending on discovery along the way).

## 17. Should v1.0 Be Advisory or North Star Beta?

**Recommendation: Option C — Split v1.0.**

- **v1.0 = Trustworthy Advisory Kernel** (roughly Layer 1 + partial Layer 2)
- **v1.1 / v2.0 = Runtime Enhancement Layer** (Layer 2 complete through Layer 6)

### Reasoning

**v1.0 cannot mean North Star beta.** The North Star requires AI-facing delivery,
measured behavior reduction, cross-platform validation, external user evidence, and
model-quality benchmarks. None of these exist today. Claiming v1.0 = North Star beta
would be a false promise that damages credibility. The distance is 16–25 releases.

**v1.0 can mean trustworthy advisory kernel.** This is the conservative path from the
v0.42 report, with one important addition: AI-facing delivery design (Layer 2) must
be explored before v1.0, even if full delivery isn't implemented until v1.1+. The
advisory kernel is trustworthy when:
- Cross-platform validation is complete
- All project lanes have live hook evidence
- Dirty-tree recovery guidance exists
- At least one external user has completed the loop
- AI-facing delivery has a validated design (not necessarily implemented)
- The manual-CLI trap is acknowledged and a path out of it is defined

**Why not make v1.0 = current state?** Because current state (Layer 0) is a
manual-CLI trap that contradicts the North Star. Shipping v1.0 without at least
a validated plan for AI-facing delivery would cement the manual model as the product.

**What v1.0 should promise:**
- AKAR provides discipline-tuned session guidance for AI coding tools
- AKAR blocks dangerous commands through optional PreToolUse hooks
- AKAR measures and budgets diffs across tasks
- AKAR works on Linux, macOS, and Windows for Rust, Node, Python, and Unknown projects
- AKAR never executes project code itself
- AKAR's guidance reaches the AI through documented, reliable mechanisms (manual
  relay or auto-injection, depending on Layer 2 progress)
- AKAR is honest about what it does and does not do

**What v1.0 should NOT promise:**
- "AKAR makes cheap AI models work better" — unmeasured
- "AKAR reduces your token costs" — unmeasured
- "AKAR works automatically — just install and forget" — not true before Layer 2+
- "AKAR works with Codex, OpenCode, or any AI tool" — not true before Layer 4+

## 18. Future AKAR Benchmark Requirement

The North Star makes specific claims about synthetic enhancement — that AKAR improves
AI outcomes "similar to an enhancement layer that improves benchmark-like outcomes."
These claims must eventually be tested. Here is what a future benchmark should look
like. **This is a design, not an implementation. Do not build this now.**

### Benchmark Design (Future — Layer 5)

**Comparison axes:**
1. AI without AKAR (baseline)
2. AI with current advisory AKAR (manual CLI relay)
3. AI with future AI-facing AKAR (auto-injected context)
4. Cheap model without AKAR
5. Cheap model with AKAR
6. Expensive model without AKAR
7. Expensive model with AKAR

**Task types:**
- Bugfix (single-function, known failing test)
- Feature addition (new function with tests)
- Refactor (restructure without behavior change)
- Documentation (update docs for changed API)
- Multi-file change (coordinated changes across 3+ files)

**Metrics per task:**
- Task success rate (did the fix work? tests pass?)
- Files touched (count)
- LOC changed (added + deleted)
- Number of user prompts (how many times did user have to re-ask?)
- Number of tool calls (Bash, Read, Write, Edit, etc.)
- Input/output token estimate (if available from session metadata)
- Hallucinated command count (commands that don't exist in the project)
- Dangerous command attempts (blocked or would-have-been-dangerous)
- Verification completion (were tests/checks actually run?)
- Final answer quality (manual review: correct, complete, minimal)
- Time-to-correct-fix (wall clock from task start to passing tests)

**Minimum trial size:** 10 tasks per comparison axis, 3 repetitions each to control
for AI non-determinism. Total: ~210 individual task runs.

**What AKAR has today:** Zero benchmark data. No controlled comparisons. No
measurement infrastructure. This is not a criticism — it's a statement of what
evidence is needed before making uplift claims.

**What must be true before running this benchmark:**
- AI-facing delivery (Layer 2) must be working, so AKAR context is consistently
  present in the AI session for conditions 3, 5, 7
- Measurement infrastructure (Layer 3) must exist to collect metrics
- Multiple models must be accessible for comparison

**Honest note:** This benchmark may show smaller effects than the North Star hopes.
That's fine — the point is to measure honestly, not to confirm hopes. If AKAR's
effect is small, the response should be to improve AKAR, not to cherry-pick results.

## 19. What to Keep

From current AKAR, these are worth keeping and building on:

| Component | Why Keep | Layer Relevance |
|---|---|---|
| Safety hooks + classifier | Directly prevents dangerous commands; proven on Node/Windows | Layers 0–6 |
| Project detection | Eliminates toolchain hallucination; works deterministically | Layers 0–6 |
| Verification discovery | Surfaces real commands without AI guesswork | Layers 0–6 |
| NEXT_RUN compiler | Best discipline artifact AKAR produces; content is correct | Layers 0–6 (delivery mechanism will change) |
| Structural contract validator | Prevents malformed guidance; fast, deterministic | Layers 0–6 |
| Diff baseline + postmortem | Only measurement AKAR has; foundation for Layer 3 metrics | Layers 0–6 |
| Doctor/status | Honest environment reporting; essential for trust | Layers 0–6 |
| Embedded hook templates | Works without source tree; proven in external dogfood | Layers 0–6 |
| Eval suite (28 cases) | Catches regressions; fast, reliable | Layers 0–6 |
| Test suite (508 tests) | Broad coverage; prevents silent breakage | Layers 0–6 |
| Dogfood audit discipline | Prevents self-deception; essential for honest progress | Layers 0–6 |

## 20. What to Stop

These activities should stop immediately:

| Activity | Why Stop |
|---|---|
| **Planning capsule architecture** | No evidence; crosses advisory→autonomous boundary; distracts from AI-facing delivery |
| **Designing token optimization** | No measurement baseline; no API access; premature |
| **Specifying Codex/OpenCode adapters** | Claude Code integration is barely proven; no multi-host demand |
| **Designing skill conflict resolver** | Skill semantics not modeled; wrong layer (belongs in AI, not AKAR) |
| **Planning autopilot modes** | Crosses advisory→autonomous boundary; auto-invocation ≠ auto-execution |
| **Designing memory engines** | Memory files are passive; prove AI reads them before optimizing |
| **Adding more CLI commands** | The problem is too many commands, not too few; each new command increases burden |

## 21. What to Postpone

These ideas have potential future value but no current evidence:

| Idea | When to Revisit | Prerequisite |
|---|---|---|
| Capsules | After Layer 2 (AI-facing delivery) | May become a packaging format for auto-delivered context |
| Token optimization | After Layer 3 (measured reduction) | Requires measurement baseline and delivery mechanism |
| Host adapters | After Layer 4 (host-enabled runtime) | Requires Claude Code integration to be solid first |
| Skill resolver | After Layer 5 (model-quality validation) | May not belong in AKAR at all — could be AI-level concern |
| Autopilot | Possibly never | Auto-invocation of advisory commands ≠ autonomous execution |
| Memory engine | After Layer 2 (AI-facing delivery) | Prove AI reads memory files; then optimize |
| Model/API calls | After Layer 5 | Would fundamentally change AKAR's risk profile |

## 22. What Must Be Measured Next

Before building anything new, these measurements are needed:

1. **Command count per task cycle** — exactly how many CLI commands does a
   disciplined user run per task? (Baseline: ~7)
2. **Time per command** — how long does each command take? Where is the friction?
3. **AI context presence** — in what fraction of dogfood sessions does the AI
   actually read and use NEXT_RUN.md content?
4. **User error rate** — how often does the user forget a command, run commands
   out of order, or skip postmortem?
5. **Dirty-tree frequency** — how often does `.akar/` cause a dirty-tree refusal
   on `preflight --snapshot`?
6. **Hook wiring time** — how long does a new user take to wire hooks manually?

These measurements will inform whether Layer 1 (friction reduction) or Layer 2
(AI-facing delivery) should come first. If dirty-tree friction dominates, Layer 1
is correct. If command count and manual relay dominate, jump to Layer 2 design.

## 23. Recommended Next Release

**v0.44.0 Manual-CLI Burden Reduction Design.**

### Why This, Not Dirty-Tree Recovery

The North Star gap matrix shows 9 Huge gaps. The largest cluster is **manual-CLI
trap** (dimensions 1, 2, 3, 6, 19). Dirty-tree recovery (v0.42 report's
recommendation) would add another CLI command — a diagnostic tool that the user
must still manually invoke. It would reduce one friction point (dirty-tree
confusion) while leaving the fundamental problem (too many manual commands)
untouched.

The North Star explicitly says: "AKAR should not be a manual checklist the user
repeatedly operates." Adding more checklist items, even helpful ones, moves away
from the North Star. The right next step is to design how AKAR stops being a
manual checklist.

### What v0.44.0 Should Produce

An audit/design report (no code changes) that answers:

1. **How can AKAR context reach the AI session without user copy/paste?**
   - CLAUDE.md injection? Session initialization hook? Filesystem path the AI
     always reads? Something else?
2. **How can advisory commands be auto-invoked without auto-executing project code?**
   - PreToolUse hook triggers? Session lifecycle events? File watcher?
3. **What is the minimum viable auto-invocation surface?**
   - Which commands must be auto-invoked? (candidates: preflight on session start,
     postmortem on session end, request on task change)
   - Which commands must remain manual? (candidates: init, hooks --install, doctor
     --fix, learn --resolve)
4. **What does the toggle mechanism look like?**
   - `akar enable` / `akar disable`? Environment variable? Config file?
5. **What are the safety boundaries?**
   - Auto-invocation must never auto-execute project code
   - Auto-invocation must never auto-modify Claude Code settings
   - Auto-invocation must never auto-commit or auto-push
   - Toggle-off must be immediate and complete
6. **What evidence is needed before implementing the design?**
   - External user feedback on the design
   - At least one manual simulation of the auto-invocation flow
   - Confirmation that the design works within Claude Code's extension model

### Hard Rules for v0.44.0
- Audit/design report only — no src/ modifications
- No auto-execution design — auto-invocation of advisory commands only
- No model/API calls in the design
- No token optimization in the design
- Design must work within documented Claude Code extension points
- Design must preserve AKAR's "never executes project code" guarantee

### Alternative: v0.44.0 Dirty-Tree Recovery Guidance

If measurement (section 22) shows dirty-tree friction is the dominant user pain
point, and command count is tolerable, then dirty-tree recovery is the right next
step. But the burden of proof is on this alternative — the North Star gap matrix
strongly suggests the manual-CLI trap is the bigger problem.

## 24. Honest Conclusion

AKAR v0.42.0 is a stable advisory alpha that does useful foundation work (safety
hooks, project detection, diff budgets, verification discovery) but has drifted
into a manual-CLI trap that contradicts the North Star. The user operates AKAR
as a manual checklist — exactly what the North Star says AKAR should not be.

The gap between current AKAR and the North Star is large (9 of 20 dimensions are
Huge gaps) but not infinite. The path is clear: solve AI-facing delivery (how AKAR
context reaches the AI session without user relay), then measure whether it actually
reduces negative behaviors, then validate across platforms and models. This path is
16–25 releases long.

The speculative ideas from recent planning (capsules, token optimization, adapters,
skill resolver, autopilot, memory engine) are not the path. They skip the
prerequisite step — AI-facing delivery — and several of them cross the
advisory→autonomous boundary without evidence that crossing it is necessary.

The most important thing AKAR can do next is not build more features. It is to
design how the features it already has can reach the AI session without the user
carrying them there manually. That is the root problem. AKAR means root.
