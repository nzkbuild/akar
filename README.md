# AKAR — Adaptive Knowledge & Action Runtime

AKAR is a local runtime governance layer for Claude Code. It classifies tasks, enforces diff budgets, runs verification, records telemetry, and summarizes outcomes.

## Quickstart

```powershell
# 1. Build
cargo build --release

# 2. Verify
akar --version

# 3. Initialize your project
akar bootstrap
akar doctor

# 4. Run a task
akar preflight "fix the login button"
akar run "fix the login button"
akar postmortem
```

See [docs/INSTALL.md](docs/INSTALL.md) for full install instructions.

## What it does

- Classifies prompts into task contracts with diff budgets
- Runs honest verification (tests are evidence, not proof)
- Detects skill conflicts and recommends safe execution modes
- Records local telemetry and postmortem outcomes
- Proposes learning patches when missions degrade or fail
- Advises on request pressure strategy

## What it is not

- Not a local LLM, daemon, or vector DB
- Not a replacement for Claude Code
- Not an autonomous code editor (v0.2.x is scaffold/runtime mode)

## Requirements

- Rust 1.70+ / Cargo
- Windows (primary target), macOS/Linux supported

## Commands

| Command | Description |
|---|---|
| `akar status` | Full runtime health at a glance |
| `akar bootstrap` | Initialize project .akar/ with memory templates |
| `akar doctor` | Read-only health check |
| `akar doctor --fix` | Apply safe reversible fixes |
| `akar preflight "<task>"` | Strategy review before executing |
| `akar run "<task>"` | Stable workflow: preflight → mission → postmortem |
| `akar mission "<task>"` | Mission state machine (scaffold mode) |
| `akar telemetry` | Show local event log summary |
| `akar postmortem` | Review latest mission outcome |
| `akar learn` | Propose learning patch if degraded/failed |
| `akar skills` | Skill registry with conflict detection |
| `akar request` | Request pressure advisory |
| `akar eval` | Run eval harness (28 scenarios) |
| `akar verify` | Run verification recipe |
| `akar safety "<cmd>"` | Classify command risk |
| `akar calibrate` | Model/gateway profile |
| `akar hooks` | Hook install instructions |

## Test

```powershell
cargo test
```

## Docs

- [Install Guide](docs/INSTALL.md)
- [Operating Model](docs/OPERATING_MODEL.md)
- [Evaluation Plan](docs/EVALUATION_PLAN.md)
- [Release Checklist](docs/RELEASE_CHECKLIST.md)
- [Architecture](docs/architecture/AKAR_OS.md)
- [Roadmap](AKAR_MASTER_ROADMAP_v1.0_REVISED.md)

## Project layout

```
akar/
  src/                   # Rust CLI modules (21 modules)
  docs/
    architecture/        # AKAR OS, product roadmap, intelligence roadmap
    kernel/              # 12 kernel policy docs
    rfcs/                # RFC-0001 through RFC-0004
    INSTALL.md
    OPERATING_MODEL.md
    EVALUATION_PLAN.md
    RELEASE_CHECKLIST.md
  templates/             # Memory file templates
  .claude/commands/      # Claude Code slash commands
  hooks/                 # Pre-commit hook scripts
  .akar/                 # Project runtime state (gitignored artifacts)
  Cargo.toml
```

Current version: **v0.2.1** — Install + Operating Model + Evaluation Plan
