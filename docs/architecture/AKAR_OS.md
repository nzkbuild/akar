# AKAR OS — Architecture Framing

## Thesis

AKAR is not a runtime with modules. It is an AI Engineering Operating System
hosted inside Claude Code. This framing has concrete design consequences.

## OS Mapping

| OS Concept          | AKAR Equivalent         | Role                                              |
|---------------------|-------------------------|---------------------------------------------------|
| CPU                 | AI Model                | Raw compute; executes instructions                |
| Host Runtime        | Claude Code             | Hardware abstraction; tool and file I/O           |
| OS Kernel           | AKAR Kernel             | Policy enforcement; resource governance           |
| Process Launcher    | Mission Compiler        | Parses intent, emits Task Contract                |
| Process Manifest    | Task Contract           | Declares scope, constraints, success criteria     |
| Memory Paging       | Context Pack            | Loads only what a task needs; evicts stale state  |
| Driver Manager      | Skill Intelligence      | Classifies, loads, and conflicts-checks skills    |
| Scheduler           | Request Intelligence    | Allocates request budget; adapts under pressure   |
| Self-Improvement    | Learning Intelligence   | Observes failures; patches behavior               |
| Recovery Env        | Doctor                  | Diagnoses and repairs broken AKAR state           |
| System Journal      | Event Log               | Append-only telemetry; local only                 |
| Health Checks       | Verification            | Proves completion before closing a task           |
| Kernel Permissions  | Safety Policy           | Hard limits; cannot be overridden by skills       |
| CPU Profile         | Model Profile           | Declares model capabilities and cost tier         |
| Network Adapter     | Gateway                 | Routes model calls; handles auth and fallback     |

## Core Principle

AKAR OS decides whether a skill should be loaded at all.
Skills are drivers, plugins, or libraries — not bosses.

A skill may suggest behavior. AKAR Kernel decides whether to apply it.
No skill can override kernel policy. No skill can activate another skill.
No skill can claim exclusive methodology control without kernel authorization.

## Authority Hierarchy

```
Safety Policy (hard floor)
  └─ AKAR Kernel
       ├─ Mission Compiler  (process launch)
       ├─ Skill Intelligence (driver selection)
       ├─ Request Intelligence (scheduler)
       ├─ Learning Intelligence (self-repair)
       └─ Skills (loaded on demand, scoped, evictable)
```

## What This Changes

- Skills are evaluated before activation, not trusted by default.
- Methodology skills (Superpower, GSD) default to library-only unless kernel
  explicitly grants methodology-controller role.
- Only one methodology controller may be active per mission.
- Kernel conflicts → all non-AKAR skills downgraded to library-only.
- Doctor is a recovery environment, not a feature; it runs when the OS is sick.
- Verification is a health check gate, not an optional step.

## Status

Adopted in v0.1.1. See RFC-0001-AKAR-OS.md.
