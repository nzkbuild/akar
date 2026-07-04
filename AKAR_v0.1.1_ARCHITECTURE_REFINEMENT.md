# AKAR v0.1.1 — Architecture Refinement Consolidation

**Project:** AKAR — Adaptive Knowledge & Action Runtime  
**New framing:** AKAR is evolving from a runtime into an **AI Engineering Operating System** hosted inside Claude Code.  
**Document status:** Next-source-of-truth for post-v0.1.0 work.  
**Target version:** `v0.1.1`  
**Purpose:** Consolidate the real lessons from the overnight autopilot run and define the next implementation mission before giving Claude Code a prompt.

---

## 1. Current Situation

AKAR v0.1.0 has a working baseline.

Verified state from the latest run:

```txt
cargo test: 121 passed, 0 failed
akar --version: akar 0.1.0
akar doctor: OK after doctor --fix
redact_pattern_ci exists and compile issue was repaired
```

The overnight run created a strong foundation, but it also exposed architectural gaps.

The biggest discovery:

> AKAR must not become another skill.  
> AKAR must become the operating layer that decides when skills, tools, memory, models, and strategies should be used.

---

## 2. Why v0.1.1, Not Phase 2

The next work should **not** be called Phase 2.

Reason:

```txt
Phase 2 sounds like "continue building more features."
But the real next step is architecture correction.
```

This is not expansion yet.  
This is not feature growth yet.  
This is not optimization yet.

This is:

```txt
v0.1.1 — Architecture Refinement
```

v0.1.1 exists to fix the shape of the system before AKAR grows bigger.

---

## 3. Version Strategy

### v0.1.0 — Foundation

Already built.

Scope:

```txt
Rust CLI
Doctor
Mission runtime
Task contract
Context pack
Verification
Event log
Memory templates
Safety
Design module
Skill registry
Model profile
Eval harness
Claude command files
Hooks scaffold
```

Status:

```txt
Functional baseline exists.
Tests pass.
Doctor OK after bootstrap.
```

---

### v0.1.1 — Architecture Refinement

Next.

Purpose:

```txt
Convert AKAR from "runtime with modules" into "AI Engineering OS architecture."
```

Main work:

```txt
Skill Intelligence
Request Intelligence
Learning Intelligence
Runtime Telemetry
RFC governance
Strategy routing
Architecture freeze rules
```

v0.1.1 should focus on understanding and controlling the system, not adding many features.

---

### v0.1.2 — Optimization

Later.

Purpose:

```txt
Make AKAR lighter, faster, cleaner, and less noisy.
```

Scope:

```txt
warning cleanup
RAM measurement
context pack size reduction
lazy loading
report lifecycle
log rotation tuning
unused code cleanup
IO optimization
```

---

### v0.2.0 — First Stable Runtime

Later.

Purpose:

```txt
Make AKAR usable as a reliable daily runtime.
```

Scope:

```txt
real bootstrap wiring
real Claude Code command integration test
real skill routing
real postmortem command
safe install/uninstall
better documentation
release packaging
```

---

### v0.3+ — Expansion

Only after the architecture is stable.

Possible areas:

```txt
SQLite index
visual report
cross-project intelligence
optional embeddings
advanced model routing
browser visual verification
```

---

### v1.0 — Battle Tested

v1.0 should mean:

```txt
AKAR has survived real projects, real failures, real quota pressure, real skill conflicts, and real recovery cycles.
```

Not merely:

```txt
all planned commands exist
```

---

## 4. Replace "Phases" with Layers

The old roadmap was too linear:

```txt
Phase 1 → Phase 2 → Phase 3
```

AKAR is not linear. It is an evolving operating layer.

Use this structure instead:

```txt
Foundation
Architecture
Optimization
Expansion
Hardening
Evolution
```

Each layer can be revisited.

Example:

```txt
Foundation can be patched.
Architecture can be refined.
Optimization can happen repeatedly.
Evolution learns from real failures.
```

---

## 5. Product Roadmap vs Intelligence Roadmap

AKAR needs two roadmaps.

---

### 5.1 Product Roadmap

This describes what users see.

```txt
v0.1.0 — Foundation CLI
v0.1.1 — Architecture Refinement
v0.1.2 — Optimization
v0.2.0 — First Stable Runtime
v0.3.0 — Expansion
v0.5.0 — Production Candidate
v1.0.0 — Battle-Tested Release
```

---

### 5.2 Intelligence Roadmap

This describes how smart AKAR itself becomes.

```txt
Level 0 — Prompt Executor
Level 1 — Mission Compiler
Level 2 — Adaptive Runtime
Level 3 — Skill Intelligence
Level 4 — Request Intelligence
Level 5 — Learning Intelligence
Level 6 — Runtime Self-Optimization
Level 7 — Self-Evolving Engineering OS
```

This is important because AKAR is not just software with features.  
It is an intelligence layer that should become better through real usage.

---

## 6. New Architecture Framing: AKAR OS

AKAR should be treated as an **AI Engineering OS**, not a skill pack.

Mapping:

```txt
AI model                 = CPU
Claude Code              = host runtime / hardware interface
AKAR Kernel              = operating system kernel
Mission Compiler         = process launcher
Task Contract            = process manifest
Context Pack             = memory paging
Skill Intelligence       = driver manager
Request Intelligence     = scheduler / resource manager
Learning Intelligence    = self-improvement layer
Doctor                   = recovery environment
Event Log                = system journal
Verification             = health checks
Safety Policy            = kernel permissions
Model Profile            = CPU profile
Gateway Profile          = network adapter profile
```

This framing helps prevent AKAR from becoming a pile of prompts.

---

## 7. Core Principle Change

Old assumption:

```txt
AKAR Runtime uses skills.
```

Better assumption:

```txt
AKAR OS decides whether a skill should be loaded at all.
```

Skills are not first-class authorities.

Skills are:

```txt
drivers
plugins
libraries
optional accelerators
```

The kernel remains the authority.

---

## 8. The Overnight Run as a Learning Event

The overnight run revealed:

```txt
Large autopilot mission
+ Superpower methodology active
+ GSD preference layer present
+ full autonomy
+ no skill intelligence
+ no request intelligence
+ no learning loop
= too many requests, too much repeated input context, unclear methodology hierarchy
```

Observed pattern:

```txt
Many phases claimed complete
but verification had to be checked manually after the fact
```

AKAR must learn from this.

Rule:

```txt
Every real failure becomes at least one of:
1. fixed code
2. fixed runtime behavior
3. documented limitation
4. future eval
```

---

## 9. Skill Intelligence

### 9.1 Why Skill Intelligence Exists

The user has at least two strong behavior layers:

```txt
Superpower
Get Shit Done / GSD
```

Both can affect how Claude behaves.

Superpower appears to be a methodology/planning/spec/TDD layer.  
GSD appears to be a command/profile/preference/execution helper.

The conflict risk:

```txt
Superpower wants process and methodology.
GSD wants aggressive completion.
Claude full-autonomy tries to obey both.
```

Without a router, Claude may activate too much.

---

### 9.2 Skill Intelligence Flow

```txt
Scan available skills/plugins/commands/hooks
↓
Read metadata / README / command docs
↓
Classify purpose
↓
Classify risk and cost
↓
Detect overlap and conflict
↓
Decide active / library-only / wrapped / disabled-candidate
↓
Use only when mission needs it
↓
Measure outcome
↓
Update skill profile
```

---

### 9.3 Skill Classes

```txt
kernel-level:
  belongs to AKAR itself

methodology:
  controls process/planning/spec/TDD

execution:
  pushes completion and action

support:
  helper skill for a narrow task

memory:
  recall, self-evolution, lessons

design:
  UI/UX/style

security:
  safety, threat modeling, secret handling

dangerous:
  can modify global config, shell, git, external systems

library-only:
  available for reference, not automatic control

disable-candidate:
  redundant, risky, broken, or too costly
```

---

### 9.4 Default Classification for Current Setup

```yaml
superpower:
  type: methodology
  default_mode: library-only
  allowed_when:
    - explicit planning/spec/TDD mission
    - user asks to use Superpower
    - AKAR Skill Intelligence selects it
  not_allowed_when:
    - simple bugfix
    - micro edit
    - routine verification
    - request budget pressure
  risk: high influence
  cost: high

gsd:
  type: execution/profile-helper
  default_mode: library-only
  allowed_when:
    - explicit execution sprint
    - user asks to use GSD
    - AKAR selects it for action bias
  not_allowed_when:
    - architecture decision
    - high-risk security/payment/db task
    - when Superpower already controls methodology
  risk: medium influence
  cost: medium
```

---

### 9.5 Skill Rules

```txt
AKAR kernel always wins.
Skills must earn activation.
No all-skills mode.
Default: use zero skills.
Normal task: max 1 primary skill + 1 support skill.
Only one methodology skill may control a mission.
If Superpower is active, GSD cannot also control methodology.
If GSD is active, Superpower is reference-only unless explicitly selected.
If skill conflict is detected, downgrade all non-AKAR skills to library-only.
```

---

## 10. Request Intelligence

### 10.1 Why Request Intelligence Exists

Hard caps create dead ends.

Bad rule:

```txt
Stop after N requests.
```

Better rule:

```txt
Adapt strategy as request pressure increases.
Stop only after writing a resumable continuation state.
```

---

### 10.2 Request Intelligence Tracks

```txt
requests used
requests remaining
requests per mission
requests per phase
requests per loop
tokens per request
input/output ratio
context pack size
retry count
model used
tool calls
hook context size
verification cost
```

---

### 10.3 Request Pressure Modes

```txt
NORMAL:
  Proceed normally.

SAVER:
  Batch reads, avoid repeated status checks, avoid full roadmap reload.

COMPACT:
  Drop cold context, use compact state, avoid long histories.

BOUNDARY:
  Finish current atomic step, verify minimally, do not start a new large step.

RESUME:
  Write NEXT_RUN.md with exact continuation prompt and stop cleanly.
```

---

### 10.4 Request Intelligence Rule

```txt
Request budget is not a wall.
It is a signal to change strategy.
```

---

## 11. Learning Intelligence

### 11.1 Why Learning Intelligence Exists

AKAR should not only repair code.  
AKAR should repair the way it works.

Flow:

```txt
Observe
↓
Measure
↓
Classify failure
↓
Adapt strategy
↓
Repair
↓
Verify
↓
Store reusable lesson
↓
Add eval
↓
Apply next run
```

---

### 11.2 Failure Taxonomy

```txt
compile_error
test_failure
weak_test
context_bloat
request_spike
loop_retry
model_drift
gateway_error
hook_error
memory_conflict
skill_conflict
ui_slop
overdiff
dependency_risk
migration_risk
security_risk
doctor_failure
verification_gap
```

---

### 11.3 Learning Patch Format

```yaml
learning_patch:
  id: LP-0001
  date: 2026-07-04
  source: real_run
  trigger: "Large autopilot run with skill conflict and request spike"
  failure_type:
    - request_spike
    - skill_conflict
    - verification_gap
  observed:
    - "many requests used"
    - "Superpower and GSD present"
    - "manual verification needed after claimed completion"
  rule:
    - "Do not allow multiple methodology/execution controllers by default."
    - "At request pressure, adapt strategy before stopping."
    - "At phase boundary, verify baseline or write unverified checkpoint."
  eval_added:
    - "skill-conflict-superpower-gsd"
    - "request-pressure-compaction"
    - "claimed-complete-but-unverified"
  status: accepted
```

---

## 12. Runtime Telemetry

AKAR needs local telemetry, not cloud telemetry.

It should record compact operational facts:

```txt
mission id
timestamp
model
gateway
task type
autonomy
cost mode
skills selected
requests estimated/used if available
tokens if available
files changed
verification result
failure class
lesson created
```

Telemetry must be:
- local only,
- redacted,
- compact,
- append-only,
- rotatable.

---

## 13. RFC System

AKAR needs an RFC process to avoid uncontrolled architecture growth.

Before adding major engines, Claude must write an RFC.

---

### 13.1 RFC Lifecycle

```txt
Draft
↓
Challenge
↓
Accepted / Rejected / Deferred
↓
Implementation Plan
↓
Eval
↓
Postmortem
```

---

### 13.2 RFC Template

```md
# RFC-000X — Title

## Problem

## Why Now

## Goals

## Non-Goals

## Proposed Design

## Alternatives Considered

## Edge Cases

## Risks

## Acceptance Criteria

## Tests / Evals

## Rollback Plan

## Decision
Accepted / Rejected / Deferred
```

---

### 13.3 First RFCs

```txt
RFC-0001 — AKAR OS Architecture
RFC-0002 — Skill Intelligence
RFC-0003 — Request Intelligence
RFC-0004 — Learning Intelligence
RFC-0005 — Runtime Telemetry
RFC-0006 — Architecture Freeze Rules
```

---

## 14. Architecture Freeze

AKAR must avoid becoming:

```txt
engine on engine on engine on engine
```

Architecture Freeze means:

```txt
No new engine without RFC.
No duplicate engine.
No module without owner, input, output, and eval.
No global behavior change without acceptance criteria.
No hidden always-on process.
No feature expansion during architecture refinement.
```

---

## 15. v0.1.1 Scope

v0.1.1 should be **small but real**.

It should not fully automate everything yet.

### Must create docs

```txt
docs/architecture/AKAR_OS.md
docs/architecture/PRODUCT_ROADMAP.md
docs/architecture/INTELLIGENCE_ROADMAP.md
docs/rfcs/RFC-0001-AKAR-OS.md
docs/rfcs/RFC-0002-SKILL-INTELLIGENCE.md
docs/rfcs/RFC-0003-REQUEST-INTELLIGENCE.md
docs/rfcs/RFC-0004-LEARNING-INTELLIGENCE.md
docs/kernel/SKILL_INTELLIGENCE.md
docs/kernel/REQUEST_INTELLIGENCE.md
docs/kernel/LEARNING_INTELLIGENCE.md
docs/kernel/RUNTIME_TELEMETRY.md
docs/kernel/ARCHITECTURE_FREEZE.md
```

### Should add or refine code

```txt
src/skill_registry.rs:
  improve skill classification

src/main.rs:
  add or expose skill inspection command if already consistent with CLI design

src/eval.rs:
  add eval cases for:
  - Superpower + GSD conflict
  - no all-skills mode
  - request pressure produces resume state
  - claimed complete requires verification
  - learning patch generated from failure
```

### Optional code only if simple

```txt
akar skills
akar skills --report
akar postmortem
akar telemetry
```

Do not overbuild these in v0.1.1.  
A report stub is acceptable if the architecture docs are strong.

---

## 16. v0.1.1 Non-Goals

Do not:

```txt
edit global ~/.claude
disable Superpower
disable GSD
auto-install hooks
build daemon
build GUI
add SQLite
add vector DB
add cloud telemetry
build full model router
implement full request accounting from provider logs
rewrite existing modules
start broad refactor
```

---

## 17. Acceptance Criteria

v0.1.1 is done when:

```txt
1. AKAR OS architecture is documented.
2. Product Roadmap and Intelligence Roadmap exist.
3. RFC system exists with at least four RFCs.
4. Skill Intelligence docs exist.
5. Request Intelligence docs exist.
6. Learning Intelligence docs exist.
7. Architecture Freeze docs exist.
8. Superpower and GSD are classified in a generated or documented skill inventory.
9. Eval harness includes cases for skill conflict, request pressure, and learning loop.
10. cargo test passes.
11. No global ~/.claude files are modified.
12. Final response clearly says what is still design-only vs implemented.
```

---

## 18. Prompt for Claude Code

Use this prompt after committing the current v0.1.0 baseline.

```txt
Mission: AKAR v0.1.1 Architecture Refinement.

Do not continue broad feature development.
Do not treat this as Phase 2.
Do not use Superpower as a methodology controller for this task.
Do not use Get Shit Done / GSD as a methodology controller for this task.
Do not edit global ~/.claude.
Do not disable any installed skill or plugin.
Do not add a daemon, SQLite, vector DB, GUI, or cloud telemetry.
Do not rewrite the whole codebase.

Context:
AKAR v0.1.0 baseline is complete enough to refine architecture:
- cargo test passes with 121 tests
- doctor is OK after doctor --fix
- CLI scaffold exists
- skill_registry, model_profile, mission, event_log, safety, eval modules exist
- overnight autopilot revealed skill conflict / request burn / verification trust issues

Goal:
Convert AKAR from a linear runtime roadmap into an AI Engineering OS architecture with Product Roadmap + Intelligence Roadmap + RFC governance.

Implement v0.1.1 only.

Required docs:
1. docs/architecture/AKAR_OS.md
2. docs/architecture/PRODUCT_ROADMAP.md
3. docs/architecture/INTELLIGENCE_ROADMAP.md
4. docs/rfcs/RFC-0001-AKAR-OS.md
5. docs/rfcs/RFC-0002-SKILL-INTELLIGENCE.md
6. docs/rfcs/RFC-0003-REQUEST-INTELLIGENCE.md
7. docs/rfcs/RFC-0004-LEARNING-INTELLIGENCE.md
8. docs/kernel/SKILL_INTELLIGENCE.md
9. docs/kernel/REQUEST_INTELLIGENCE.md
10. docs/kernel/LEARNING_INTELLIGENCE.md
11. docs/kernel/RUNTIME_TELEMETRY.md
12. docs/kernel/ARCHITECTURE_FREEZE.md

Required implementation:
1. Improve skill registry classification if needed.
2. Add support for identifying methodology/execution/support/library-only skill roles if it does not already exist.
3. Add eval cases for:
   - Superpower + GSD conflict should not activate both as controllers.
   - No all-skills mode.
   - Request pressure should compact/resume, not hard-dead-end.
   - Claimed completion requires verification evidence.
   - Real failure should produce a learning patch or future eval.
4. Keep implementation small and aligned with existing code style.
5. Do not add dependencies unless strictly necessary.
6. Run cargo test.

Optional only if naturally small:
- Add `akar skills` or `akar skills --report`.
- Add `akar postmortem` stub.
- Add `akar telemetry` stub.

Important architecture rules:
- AKAR kernel always wins.
- Skills are drivers/plugins/libraries, not bosses.
- Skills must earn activation.
- No all-skills mode.
- Only one methodology skill may control a mission.
- Superpower defaults to methodology/library-only.
- GSD defaults to execution/profile-helper/library-only.
- Request budget is a signal to adapt strategy, not just stop.
- Every real failure becomes fixed code, fixed runtime behavior, documented limitation, or future eval.

Verification:
- cargo test
- cargo run -- doctor
- cargo run -- eval
- If commands are added, smoke-test them.

Final response format:
Done.

Changed:
- ...

Verified:
- ...

Not verified:
- ...

Design-only:
- ...

Implemented:
- ...

Next:
- one best next task only
```

---

## 19. Immediate Human Workflow

Before running the prompt:

```powershell
cargo test
git add .
git commit -m "Initial AKAR runtime baseline"
```

Then run the v0.1.1 prompt.

After Claude finishes:

```powershell
cargo test
cargo run -- doctor
cargo run -- eval
git diff --stat
```

Then decide whether to commit:

```powershell
git add .
git commit -m "Refine AKAR OS architecture and intelligence roadmap"
```

---

## 20. Final Reminder

AKAR is not a bigger prompt.

AKAR is not a better skill.

AKAR is not Superpower replacement only.

AKAR is becoming:

```txt
An AI Engineering Operating System
that uses Claude Code as host
and treats models as replaceable CPUs.
```

The next mission is not to add more power.

The next mission is to teach AKAR how to decide **which power should be used, when, why, and at what cost**.
