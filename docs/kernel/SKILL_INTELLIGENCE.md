# Kernel Policy: Skill Intelligence

## Must

- Classify every skill by role before activation.
- Enforce single Methodology controller per mission.
- Log all downgrades to EVENT_LOG.jsonl.
- Generate a Learning Patch for every skill_conflict failure.

## Should

- Default external methodology skills to LibraryOnly.
- Prefer lower-blast-radius resolution when ambiguity exists.
- Report active skill roles in the Mission Compiler output.

## Never

- Allow two Methodology controllers active simultaneously.
- Let a skill override kernel policy.
- Let a skill activate another skill.
- Activate a Dangerous skill without explicit Task Contract opt-in.

## Role Definitions

| Role          | Control Scope                                         |
|---------------|-------------------------------------------------------|
| Kernel        | Full; AKAR-native only                               |
| Methodology   | Controls plan/verify/loop cadence                    |
| Execution     | Performs concrete tasks (write, test, deploy)         |
| Support       | Read-only augmentation                                |
| Memory        | State read/write only                                 |
| Design        | UI/visual output only                                 |
| Security      | Audit only; never modifies                           |
| Dangerous     | High blast radius; requires explicit opt-in           |
| LibraryOnly   | Content provider; no behavioral control               |

## Conflict Resolution

1. Kernel conflicts: log as anomaly (should never occur).
2. Methodology conflicts (>1): downgrade all to LibraryOnly.
3. Dangerous without opt-in: block activation, log.
4. All downgrades emit a Learning Patch.

## Default Classifications

- `superpowers:*` → LibraryOnly (methodology skills; conflict risk)
- `gsd:*` → LibraryOnly (methodology skills; conflict risk)
- `akar-*` → Kernel (native; authored by AKAR)
- All others → evaluated at load time per mission context

## Effective

v0.1.1. Implementation target: v0.1.2. See RFC-0002.
