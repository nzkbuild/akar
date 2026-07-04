# RFC-0002: Skill Intelligence

- Status: Accepted
- Date: 2026-07-04
- Author: AKAR Kernel Team

## Problem

Skills in v0.1.0 were uncontrolled. Any loaded skill could influence methodology,
request strategy, or verification behavior. The overnight run produced a concrete
failure: Superpower (methodology) and GSD (methodology) were both active,
resulting in conflicting control signals with no resolution mechanism.

## Why Now

Skill conflicts are the highest-priority gap from the overnight run. Without
classification and conflict detection, every multi-skill mission risks undefined
behavior.

## Goals

- Classify every skill by role before activation.
- Detect and resolve conflicts automatically.
- Enforce single methodology controller per mission.
- Default external methodology skills to library-only.

## Non-Goals

- Does not rewrite or remove existing skills.
- Does not prevent skills from being loaded; prevents them from being trusted
  with roles they haven't earned.

## Proposed Design

### SkillRole Enum

| Role          | Description                                              |
|---------------|----------------------------------------------------------|
| Kernel        | AKAR-native; always wins; cannot be downgraded           |
| Methodology   | Controls how work is done (plan/verify/loop cadence)     |
| Execution     | Performs concrete tasks (write, test, deploy)            |
| Support       | Augments without controlling (memory, research)          |
| Memory        | Reads/writes persistent state only                       |
| Design        | UI/visual output only                                    |
| Security      | Audits; never modifies                                   |
| Dangerous     | High blast radius; requires explicit mission opt-in      |
| LibraryOnly   | Content provider; no behavioral control                  |

### Default Classifications

| Skill              | Default Role     | Rationale                              |
|--------------------|------------------|----------------------------------------|
| superpowers:*      | LibraryOnly      | Methodology skills; conflict risk high |
| gsd:*              | LibraryOnly      | Methodology skills; conflict risk high |
| akar-*             | Kernel           | Native; authored by AKAR               |
| design-taste-*     | Design           | Visual only                            |
| security-review    | Security         | Audit only                             |
| deep-research      | Support          | Read-only augmentation                 |
| memory-maintain    | Memory           | State only                             |

### Conflict Detection Rules

1. Only one Methodology controller may be active per mission.
2. If two Methodology skills are loaded, both downgrade to LibraryOnly.
3. Kernel role always wins; cannot be downgraded by any rule.
4. Dangerous role requires explicit `allow_dangerous: true` in Task Contract.
5. Conflict events are logged to EVENT_LOG.jsonl with failure_class:
   `skill_conflict`.

### Resolution Order

1. Check for Kernel conflicts (should never occur; log if detected).
2. Count Methodology roles. If > 1, downgrade all to LibraryOnly.
3. Check Dangerous roles against Task Contract opt-in.
4. Log all downgrades as LP entries.

## Acceptance Criteria

- [ ] Skill registry classifies roles at load time.
- [ ] Eval catches Superpower + GSD conflict and logs it.
- [ ] No mission runs with two active Methodology controllers.
- [ ] Downgrade events appear in EVENT_LOG.jsonl.

## Rollback

Remove role classification. Accept all skills as uncontrolled. Reintroduces
overnight-run conflict behavior.

## Decision

Accepted. Effective v0.1.1 (documented). Implementation target: v0.1.2.
