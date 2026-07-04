# Kernel Policy: Architecture Freeze

## Purpose

Prevent architectural drift during v0.1.1 refinement. New engines, duplicate
systems, and global behavior changes require an RFC before any implementation.

## Freeze Rules

### No new engine without an RFC
A new engine is any module that owns a subsystem (Mission Compiler, Skill
Intelligence, Request Intelligence, etc.). Draft the RFC first. Get it to
Accepted before writing code.

### No duplicate engine
If a subsystem already exists, extend it. Do not create a parallel
implementation. Duplication without RFC is a violation.

### No module without owner, input, output, and eval
Every module must declare:
- Owner: which kernel subsystem governs it
- Input: what it receives
- Output: what it produces
- Eval: how its correctness is verified

Modules without these four fields are not accepted.

### No global behavior change without acceptance criteria
A global behavior change affects more than one mission type or more than one
kernel subsystem. It requires an RFC with explicit acceptance criteria before
any implementation touches production code.

### No hidden always-on process
AKAR does not run background watchers, daemons, or polling loops without
explicit user consent. Every active process must be visible in status output.

### No feature expansion during architecture refinement
v0.1.1 is refinement only. New user-facing features are deferred to v0.1.2
or later. Refinement means: document, classify, define policies, write RFCs.
Not: ship new capabilities.

## RFC Lifecycle

```
Draft → Challenge → Accepted | Rejected | Deferred → Implementation → Eval → Postmortem
```

| Stage          | Description                                                  |
|----------------|--------------------------------------------------------------|
| Draft          | Author writes RFC with problem, design, acceptance criteria  |
| Challenge      | Any kernel contributor may raise objections or alternatives  |
| Accepted       | Design is approved; implementation may begin                 |
| Rejected       | Design is not approved; rationale documented                 |
| Deferred       | Valid but not now; revisit at named future version           |
| Implementation | Code written against the accepted RFC                        |
| Eval           | Acceptance criteria verified by eval suite                   |
| Postmortem     | What worked, what didn't; lessons stored as LP entries       |

## Must

- File an RFC before implementing any new engine or duplicate subsystem.
- Include acceptance criteria in every RFC.
- Update RFC status when it moves through the lifecycle.
- Write a postmortem LP entry when an RFC closes.

## Never

- Implement before RFC reaches Accepted.
- Merge a module without declared owner/input/output/eval.
- Add an always-on background process without user consent.
- Expand features while the freeze is active.

## Effective

v0.1.1. Freeze lifts at v0.1.2 milestone completion.
