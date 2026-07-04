# RFC-0001: AKAR OS Architecture Framing

- Status: Accepted
- Date: 2026-07-04
- Author: AKAR Kernel Team

## Problem

AKAR was evolving into a skill pack: a collection of modules that Claude Code
loads, where any skill could claim control over methodology, request strategy,
or verification. There was no authority hierarchy. Skills were bosses, not
drivers. The overnight autopilot run confirmed this: Superpower and GSD both
claimed methodology control simultaneously, producing undefined behavior.

## Why Now

The overnight run exposed three concrete gaps:
1. Skill conflicts — two methodology controllers active at once, no resolution.
2. Request burn — no pressure-aware strategy; hit limits with no continuation.
3. Verification trust gaps — tasks marked done without observable evidence.

These are architectural, not implementation bugs. Patching them without an OS
framing would produce more patches, not a stable system.

## Goals

- Establish AKAR Kernel as the authority over all skills.
- Define skills as drivers/plugins/libraries, not controllers.
- Apply the OS metaphor consistently across all AKAR subsystems.
- Give every subsystem a clear OS analog so its role is unambiguous.

## Non-Goals

- AKAR is not a GUI, daemon, or background watcher.
- AKAR is not a database or persistent server process.
- This RFC does not define implementation; it defines framing and authority.

## Proposed Design

See `docs/architecture/AKAR_OS.md` for the full OS mapping table.

Authority hierarchy:

```
Safety Policy (hard floor)
  └─ AKAR Kernel
       ├─ Mission Compiler
       ├─ Skill Intelligence
       ├─ Request Intelligence
       ├─ Learning Intelligence
       └─ Skills (loaded on demand, scoped, evictable)
```

Key rules:
- Skills may suggest behavior. Kernel decides whether to apply it.
- No skill overrides kernel policy.
- No skill activates another skill.
- Methodology control requires explicit kernel grant.

## Alternatives Considered

**Remain a runtime.** Continue treating AKAR as a runtime with pluggable
modules, no authority hierarchy. Rejected: the overnight run proves this
produces unresolvable conflicts and undefined behavior at scale.

## Edge Cases

- A skill claims it IS the kernel: rejected at classification time.
- Two AKAR-native modules conflict: kernel logs the conflict and applies the
  lower-blast-radius option; files an LP entry.
- Claude Code updates break a kernel assumption: Doctor detects on next run.

## Risks

- Over-rigidity: kernel authority could block legitimate skill behavior.
  Mitigated by the library-only fallback (skills still provide content, just
  don't control methodology).
- Framing drift: future contributors ignore the OS model.
  Mitigated by ARCHITECTURE_FREEZE.md and RFC requirement for new engines.

## Acceptance Criteria

- [ ] OS mapping table published and linked from README.
- [ ] Skill Intelligence RFC (RFC-0002) references this authority model.
- [ ] No skill in the system claims methodology control without kernel grant.
- [ ] Doctor checks for authority violations on startup.

## Rollback

Revert to v0.1.0 runtime model. Remove Skill Intelligence classification.
Accept undefined behavior on skill conflicts. Not recommended.

## Decision

Accepted. Effective v0.1.1. All subsequent RFCs must respect this hierarchy.
