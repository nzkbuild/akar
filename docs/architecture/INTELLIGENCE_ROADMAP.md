# AKAR Intelligence Roadmap

## Levels

### L0 — Prompt Executor (Done)
Raw model calls with no runtime awareness. AKAR did not exist at this level;
this is the baseline Claude Code session without any harness.

### L1 — Mission Compiler (Done)
Parses natural language intent into a structured Task Contract. Knows what the
mission is, what done looks like, and what constraints apply before touching code.

### L2 — Adaptive Runtime (v0.1.0)
Selects gateway, model profile, and cost mode based on mission type. Loads
skills from a fixed list without conflict detection. Basic telemetry recording.

### L3 — Skill Intelligence (v0.1.1)
Classifies every loaded skill by role before activation. Detects and resolves
methodology conflicts. Enforces the rule: only one methodology controller per
mission. Skills are drivers; kernel decides whether they load.

### L4 — Request Intelligence (v0.1.1 — documented, v0.1.2 — live)
Treats the request budget as a strategy signal, not a wall. Detects pressure
levels (Normal, Saver, Compact, Boundary, Resume) and adapts behavior at each
threshold. Writes NEXT_RUN.md before stopping; never dead-ends a mission.

### L5 — Learning Intelligence (v0.1.1 — documented, v0.1.2 — live)
Observes real failures, classifies them by type, and emits Learning Patches
(LP-NNNN). Patches fix code, fix behavior, document limitations, or add evals.
Every real failure produces at least one artifact. Nothing is silently absorbed.

### L6 — Runtime Self-Optimization (v0.2+)
AKAR tunes its own skill selection, context load strategy, and request pacing
based on accumulated telemetry. Proposes configuration changes as draft RFCs.
Does not self-modify without human review of the proposed change.

### L7 — Self-Evolving Engineering OS (v1.0)
AKAR drafts its own RFCs when it detects structural gaps, runs its own eval
suite to validate proposed changes, and applies accepted patches autonomously.
Human review remains required for Safety Policy changes and kernel authority
modifications. The OS improves itself within defined bounds.

## Progression Rules

- Each level must be functionally complete before the next is built.
- Documentation of a level is not the same as implementation.
- L4 and L5 are documented in v0.1.1 and implemented in v0.1.2.
- No level may be skipped. Intelligence compounds; gaps become liabilities.
