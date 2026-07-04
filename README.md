# AKAR — Adaptive Knowledge & Action Runtime

AKAR is a lightweight, model-agnostic, self-healing engineering runtime for Claude Code. It turns casual user intent into disciplined engineering missions.

## What it does

AKAR gives Claude Code a control layer:
- compiles casual prompts into structured task contracts
- enforces diff budgets to prevent overcoding
- runs honest verification (tests are evidence, not proof)
- self-heals broken config and tooling
- evolves memory compactly without pollution

## What it is not

- Not a local LLM
- Not a daemon
- Not a vector DB
- Not a replacement for Claude Code

## Requirements

- Rust 1.70+ / Cargo
- Windows (primary target), macOS/Linux supported

## Build

```powershell
cargo build --release
```

## Run

```powershell
# Check version
cargo run -- --version

# Show help
cargo run -- --help
```

## Commands

| Command | Description |
|---|---|
| `akar status` | Runtime health, context pack, design check |
| `akar doctor` | Read-only health check of config and memory |
| `akar doctor --fix` | Apply safe reversible fixes |
| `akar bootstrap` | Initialize missing AKAR memory files |
| `akar verify` | Run verification recipe honestly |
| `akar eval` | Run all 20 eval scenarios |
| `akar eval "<prompt>"` | Classify a prompt into a task contract |
| `akar mission "<prompt>"` | Run full mission state machine |
| `akar safety "<cmd>"` | Classify a shell command's risk level |
| `akar skills` | List registered skills, check kernel conflicts |
| `akar calibrate` | Show model/gateway profile |
| `akar hooks` | Show hook paths and install instructions |

## Test

```powershell
cargo test
```

## Project layout

```
akar/
  src/
    main.rs            # CLI entry point and command dispatch
    config.rs          # Path discovery and secret redaction
    contract.rs        # Task contract and prompt classifier
    context_pack.rs    # Context pack builder (HOT/WARM/COLD)
    verify.rs          # Verification recipe and test intelligence
    doctor.rs          # Read-only health checks
    backup.rs          # File backup and restore
    safe_fix.rs        # Safe reversible fixes
    event_log.rs       # JSONL event log with rotation
    mission.rs         # Mission state machine
    design.rs          # Design quality module
    safety.rs          # Command risk and secret detection
    skill_registry.rs  # Skill registry and kernel priority checks
    model_profile.rs   # Model/gateway drift detection
    eval.rs            # 20-scenario eval harness
    circuit_breaker.rs # Runaway retry prevention
  docs/
    kernel/            # 12 kernel policy docs
    AKAR_ADOPTION_NOTES.md
  templates/           # Memory file templates (PROJECT_DNA, STATE, etc.)
  .claude/commands/    # Claude Code slash commands
  hooks/               # Pre-commit hook scripts (bash + powershell)
  evals/               # Eval scenario files
  examples/            # Usage examples
  tests/               # Integration tests
  Cargo.toml
```

## Roadmap

See `AKAR_MASTER_ROADMAP_v1.0_REVISED.md` for the full phased plan.

Current phase: **v1.0 Scaffold Complete**
