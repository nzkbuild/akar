# AKAR

<p align="center">
  <img src="assets/branding/akar.png" alt="AKAR" width="180" />
</p>

**A local CLI companion for AI-assisted software engineering.**

AKAR sits alongside Claude Code and helps add structure to AI coding sessions — preflight checks, diff budgets, skill awareness, local telemetry, postmortems, and learning notes.

It does not write your code. It helps slow the agent down before it does something weird.

---

## What it does

- Classifies your task before the agent touches anything
- Enforces a diff budget so a small fix stays small
- Detects skill conflicts (e.g. two methodology controllers active at once)
- Records local-only telemetry after each mission
- Summarises what happened and whether it went well
- Proposes a learning note if something degraded or failed

## What it does not do

- It is not an AI model
- It does not replace Claude Code
- It does not write or execute code changes (v0.2.x is scaffold/runtime mode)
- It does not send data anywhere — everything stays in `.akar/` on your machine
- It is not a benchmark, cloud service, or plugin marketplace

---

## Quick start

```powershell
# Build
cargo build --release

# Verify
akar --version

# Initialise a project
akar bootstrap
akar doctor
```

See [docs/INSTALL.md](docs/INSTALL.md) for full install instructions.

---

## Normal workflow

```
akar bootstrap          # one-time project setup
akar doctor             # confirm health
akar preflight "task"   # review strategy before acting
akar run "task"         # full workflow in one command
akar postmortem         # review what happened
akar learn              # propose a learning note if needed
```

---

## Commands

| Command | Description |
|---|---|
| `akar status` | Runtime health at a glance |
| `akar bootstrap` | Initialise `.akar/` with memory templates |
| `akar doctor` | Read-only health check |
| `akar doctor --fix` | Apply safe reversible fixes |
| `akar preflight "<task>"` | Strategy review before acting |
| `akar run "<task>"` | Full workflow: preflight → mission → postmortem |
| `akar mission "<task>"` | Mission state machine (scaffold mode) |
| `akar telemetry` | Show local event log |
| `akar postmortem` | Review latest outcome |
| `akar learn` | Propose learning patch if degraded or failed |
| `akar skills` | Skill registry with conflict detection |
| `akar request` | Request pressure advisory |
| `akar eval` | Run eval harness (28 scenarios) |
| `akar verify` | Run verification recipe |
| `akar safety "<cmd>"` | Classify command risk level |
| `akar calibrate` | Model/gateway profile |
| `akar hooks` | Hook install instructions |

---

## Example output

```
$ akar status
status: HEALTHY
  runtime:    akar 0.2.2
  doctor:     OK
  bootstrap:  OK
  telemetry:  42 event(s)
  postmortem: clean
  skills:     OK
  request:    NORMAL

$ akar preflight "fix the login button"
preflight:
  task:         Bugfix
  risk:         Low
  diff_budget:  1-3 files, 5-60 LOC
  request_mode: NORMAL
  skills:       zero-skill mode (AKAR kernel only)
  verification:
    - run: cargo build
    - run: cargo test
  recommendation: Proceed — low risk task. Stay within diff budget
```

---

## Project state

- **Maturity:** early, local-first, scaffold mode
- **Code execution:** not yet — AKAR classifies and records, does not edit files
- **Data:** everything stays in `.akar/` on your machine, gitignored by default
- **Global config:** AKAR does not edit `~/.claude/` unless you explicitly run `akar bootstrap`

---

## Docs

- [Install Guide](docs/INSTALL.md)
- [Operating Model](docs/OPERATING_MODEL.md)
- [Evaluation Plan](docs/EVALUATION_PLAN.md)
- [Changelog](CHANGELOG.md)
- [Architecture](docs/architecture/AKAR_OS.md)
- [Roadmap](docs/architecture/PRODUCT_ROADMAP.md)

---

## License

License decision pending. Not yet open for redistribution.

---

## Requirements

- Rust 1.70+ / Cargo
- Windows (primary), macOS/Linux supported
