# AKAR v0.42.0 — Current Reality Re-Grounding Report

## 1. Executive Verdict

**AKAR is a stable advisory alpha, not beta, not v1-ready.** Six dogfood trials across
four project-type lanes (Rust, Node, Python, Unknown) plus live hook integration and
multi-task sessions confirm the CLI advisory loop works. Recent planning drifted into
speculative ideas (capsules, adapters, autopilot, token optimization) that are not
justified by current evidence. This report re-grounds AKAR to what it actually is today:
a local Rust CLI discipline layer that prepares scoped session instructions, measures
diffs, surfaces verification guidance, blocks dangerous shell commands through optional
hooks, and records local evidence — without executing project work itself.

## 2. One-Sentence Definition of AKAR Today

AKAR is a local Rust CLI discipline layer for AI coding sessions that prepares scoped
session instructions (NEXT_RUN.md), measures and budgets diffs (preflight/postmortem),
surfaces verification guidance (doctor/verify/discovery), blocks dangerous shell commands
through optional PreToolUse hooks (safety/hooks), and records local evidence (.akar/
state files and telemetry logs) without executing project work itself.

## 3. Baseline and Verification

| Check | Result |
|---|---|
| Commit | `e0bd5c4` — fix: polish AKAR fresh-user wording |
| Version | `akar 0.41.0` |
| Working tree | clean |
| Branch | master (ahead of origin/master by 36 commits) |
| `cargo test` | 508 passed, 0 failed |
| `cargo run -- eval` | 28/28 PASS |
| `cargo run -- doctor` | PASS (1 WARN: active split-rule in LEARNING_PATCHES.md) |
| `cargo run -- status` | HEALTHY, READY |
| `cargo run -- request "..."` | NORMAL mode |
| `cargo run -- request --check` | PASS (4/4 checks) |
| `cargo run -- governor --json --no-exit-code` | SPLIT_TASK (active split-rule; not a bug — known artifact) |
| `cargo run -- learn --list` | 8 entries, 1 active, 7 resolved |
| `cargo run -- hooks --check` | PASS (source-tree) |

## 4. Current CLI Surface

20+ CLI commands, all advisory and read-only except for state-file writes:

| Category | Commands |
|---|---|
| Onboarding | `init`, `init --claude`, `bootstrap` |
| Health | `doctor`, `doctor --fix`, `status` |
| Session prep | `preflight`, `preflight --snapshot`, `request`, `request --check` |
| Governance | `governor`, `governor --json`, `governor --one-line`, `governor --no-exit-code` |
| Verification | `verify`, `eval` |
| Safety | `safety <cmd>` |
| Hooks | `hooks`, `hooks --check`, `hooks --install` |
| Evidence | `postmortem`, `postmortem --diff --baseline`, `telemetry` |
| Learning | `learn`, `learn --list`, `learn --resolve` |
| Meta | `skills`, `calibrate`, `mission`, `run` |

All commands that could be confused with execution (`mission`, `run`, `verify`) are
explicitly labeled "ADVISORY ONLY" or "scaffold mode" and do not execute project code.

## 5. Current Local State Model

19 files under `.akar/`:

- **Session threading:** `NEXT_RUN.md`, `DIFF_BASELINE.json`
- **Telemetry:** `EVENT_LOG.jsonl`, `HOOK_EVENTS.jsonl`
- **Learning:** `LEARNING_PATCHES.md`
- **Project memory:** `PROJECT_DNA.md`, `STATE.md`, `DECISIONS.md`, `LESSONS.md`,
  `KNOWN_BUGS.md`, `TEST_DEBT.md`, `DESIGN_DNA.md`, `MODEL_PROFILE.md`,
  `VERIFY_RECIPE.md`, `SKILL_INVENTORY.md`
- **Historical archives:** `HOOK_EVENTS.before-v0.10.0.jsonl`,
  `HOOK_EVENTS.before-v0.11.0.jsonl`

All state is local, file-based, no database, no network, no cloud sync, no telemetry
exfiltration.

## 6. Current Hook Model

- Two PreToolUse hook templates embedded in the binary: `pre-tool-call.sh` (bash),
  `pre-tool-call.ps1` (PowerShell)
- Templates call `akar safety <cmd>` before tool execution
- `akar safety` classifies commands as Safe/Medium/High/Critical
- BLOCK exits 2, ALLOW/SKIP exits 0
- Hook events are logged to `.akar/HOOK_EVENTS.jsonl` in the session root project
- User must manually wire hooks into `~/.claude/settings.json` — AKAR never edits
  Claude Code settings
- Embedded fallback ensures hooks work without the AKAR source tree

## 7. Current Project Detection Model

`src/project_detection.rs` — shared module, marker-file based:
- Rust: `Cargo.toml`
- Node: `package.json`
- Python: `pyproject.toml`, `setup.py`, `requirements.txt`
- Unknown: none of the above

Detection feeds into: doctor (project kind check), NEXT_RUN compilation (appropriate
verification commands per language), verify (refuses automated execution for non-Rust),
verification discovery hints.

## 8. Current Verification Discovery Model

`src/verification_discovery.rs` — deterministic file-system-only scan:

| Source | Command | Confidence |
|---|---|---|
| `package.json` scripts.test | `npm test` | High |
| `pyproject.toml`/`pytest.ini`/`tests/` | `python -m pytest` | High |
| `Makefile` test: target | `make test` | Medium |
| `justfile` test: recipe | `just test` | Medium |
| `README.md` whitelisted commands | varies | Low–Medium |

Safety: blocklist filters dangerous patterns (curl, sudo, rm, etc.), max 5 hints,
deduplicated, stable-ordered, advisory only. All discovered commands are surfaced in
doctor, NEXT_RUN, and verify output with confidence and source annotation. AKAR never
executes discovered commands.

## 9. Current Governor Model

`src/loop_governor.rs` — reads local state files and produces decisions:

| Decision | Meaning |
|---|---|
| READY | Proceed with task |
| SNAPSHOT_NOW | Baseline needed first |
| RUN_POSTMORTEM | Postmortem needed before next task |
| COMMIT_CHECKPOINT | Commit before continuing |
| SPLIT_TASK | Task exceeded budget, split into smaller units |
| STOP_HOOK_BROKEN | Hook wiring broken, fix before continuing |
| STOP_REPEATED_BLOCK | Repeated safety blocks, reconsider approach |
| UNKNOWN | Cannot determine state |

Governor is read-only (except `--telemetry` opt-in event logging), never writes
NEXT_RUN.md, carries no task state of its own.

## 10. Current NEXT_RUN/Request Model

`akar request` compiles a discipline-tuned NEXT_RUN.md with 11 sections that must
pass `request --check` validation:

1. Current State — project kind, git status, task text
2. Objective — governor-decision-aware task framing
3. Hard Rules — budgets, file limits, no-commit-without-checkpoint
4. Safety Contract — command classification, hook expectations
5. Allowed Commands — project-appropriate commands (e.g., `npm test` for Node)
6. Verification Required — project-appropriate verification
7. Stop Conditions — when to stop (budget exceeded, dirty tree, etc.)
8. Evidence Required — postmortem expectations
9. Budget — file count and LOC limits
10. Autonomy Level — A5 (advisory only)
11. Consistency — decision consistency with governor

`request --check` validates section order, minimum content, safety contract, and
decision consistency. The validator is a structural contract, not an LLM judge.

## 11. Current Postmortem/Diff Model

`akar postmortem` measures the diff between current state and the preflight baseline:
- Reports files changed, lines added/removed, total LOC
- Compares against budget from NEXT_RUN.md
- PASS/FAIL on budget compliance
- `akar postmortem --diff --baseline` shows the measured diff
- Postmortem outcome feeds into `learn` for patch generation

## 12. Current Learn Model

`akar learn` generates learning patches from postmortem evidence:
- `learn --list` shows all patches (active and resolved)
- `learn --resolve` retires active patches
- Active split-rule patches affect governor decisions (SPLIT_TASK)
- Patches are stored in `.akar/LEARNING_PATCHES.md`
- Currently 8 entries (1 active split-rule, 7 resolved)

## 13. What AKAR Does Today

### Always Available CLI Behavior
- Detects project kind from marker files (Rust/Node/Python/Unknown)
- Classifies shell commands by risk level (Safe/Medium/High/Critical)
- Compiles discipline-tuned NEXT_RUN.md with project-appropriate commands
- Validates NEXT_RUN.md against a structural contract
- Produces loop governor decisions from local state
- Measures and budgets diffs via preflight baseline + postmortem
- Surfaces verification guidance from discovery hints
- Runs a 28-case eval suite (all passing)

### Optional Hook-Integrated Behavior
- Classifies commands in real-time via PreToolUse hooks
- Blocks Critical-class commands (exit 2)
- Logs all classified commands to HOOK_EVENTS.jsonl
- Requires manual user wiring into Claude Code settings

### Local State Behavior
- Maintains 19 files under `.akar/` — all local, no network
- Records telemetry in EVENT_LOG.jsonl (3801 events) and HOOK_EVENTS.jsonl
- Tracks learning patches in LEARNING_PATCHES.md
- Stores project memory files (DNA, STATE, DECISIONS, LESSONS, etc.)

### Dogfood-Proven Behavior
- Rust single-task lane (v0.24)
- Node single-task lane (v0.33)
- Python single-task lane (v0.37)
- Unknown-project lane with discovery hints (v0.39)
- Anchored live hook lane (v0.36)
- Multi-task Node lane (v0.40)

### Advisory-Only Behavior
- `akar mission` — scaffold mode, does not execute
- `akar run` — walks state machine, does not execute
- `akar verify` — runs build/test for Rust only; manual-only for all other projects
- `akar preflight` — writes baseline, does not modify project code
- `akar request` — writes NEXT_RUN.md, does not execute the task
- `akar postmortem` — measures diff, does not commit or revert
- All verification discovery hints — surfaced, never executed

### What AKAR Itself Does vs. What Claude/User Does

| Action | AKAR | Claude/User |
|---|---|---|
| Classify commands | Yes (safety) | No |
| Block dangerous commands | Yes (hooks) | No |
| Prepare session instructions | Yes (NEXT_RUN.md) | Reads and follows them |
| Measure diffs | Yes (preflight/postmortem) | No |
| Detect project kind | Yes (marker files) | No |
| Surface verification guidance | Yes (discovery) | Decides whether to run |
| Execute project code | **Never** | Yes |
| Edit project files | **Never** | Yes |
| Run tests | Rust build/test via `akar verify` | All other projects |
| Commit code | **Never** | Yes |
| Wire hooks | **Never** (advisory only) | Yes (manual) |
| Decide .gitignore | **Never** (advisory only) | Yes |

## 14. What AKAR Does Not Do Today

- Does not execute missions — `akar mission` is scaffold mode
- Does not edit project code by itself
- Does not auto-run discovered verification commands
- Does not install dependencies
- Does not auto-modify Claude Code settings
- Does not manage models or call model APIs
- Does not optimize tokens automatically
- Does not select skills or resolve skill conflicts
- Does not support Codex/OpenCode as real adapters
- Does not guarantee bug-free AI output
- Does not replace tests or code review
- Does not run as a daemon
- Does not provide cloud sync or telemetry exfiltration
- Does not auto-commit or auto-push
- Does not have a capsule system
- Does not have an autopilot mode
- Does not have a memory optimization engine
- Does not have a multi-agent orchestrator
- Does not execute `preflight --snapshot` or `postmortem` automatically
- Does not decide `.akar/` tracking policy for the user

## 15. Dogfood Evidence Matrix

| Trial | Version | Project | Hook | Multi-Task | Verdict |
|---|---|---|---|---|---|
| First external | v0.24 | Rust | No | No | PASS — loop works |
| Second external | v0.27 | Node | Yes (misrouted) | No | PASS — loop works; hook routing bug found |
| Third external | v0.33 | Node | No | No | PASS — project-kind awareness correct |
| Live hook | v0.35 | Node | Yes (session-anchoring found) | No | CONDITIONAL PASS |
| Anchored live hook | v0.36 | Node | Yes (correctly routed) | No | PASS — hook pipeline proven |
| Python external | v0.37 | Python | No | No | PASS — Python lane proven |
| Unknown (Makefile) | v0.39 | Unknown | No | No | PASS — discovery hints work |
| Unknown (no hint) | v0.39 | Unknown | No | No | PASS — fallback correct |
| Multi-task Node | v0.40 | Node | No | Yes (3 tasks) | PASS — state transitions correct |

**Proven lanes:** Rust, Node, Python, Unknown — all single-task. Node multi-task
(3 sequential tasks). Live hook integration (Node, anchored session).

**Not yet proven:** Cross-platform (all trials on Windows), hook integration with
non-Node projects, multi-task with non-Node projects, >3 task sequences, dirty-tree
recovery workflows, concurrent sessions, large-repo performance.

## 16. Negative Behavior Reduction Today

| Behavior | Status | Evidence |
|---|---|---|
| Fewer destructive commands | **Supported** | hooks/safety blocks Critical commands |
| Smaller diffs | **Partially supported** | budgets/postmortem measure but don't enforce |
| Clearer task boundaries | **Supported** | NEXT_RUN.md defines scope explicitly |
| Less stale task context | **Supported** | NEXT_RUN task threading + request --check |
| Less verification guessing | **Supported** | doctor/verify/discovery hint surfaces commands |
| Fewer invented project commands | **Supported** | project-kind-aware NEXT_RUN uses real toolchain |
| Less fresh-user confusion | **Partially supported** | v0.41 wording polish helped; still manual-heavy |
| Fewer repeated unsafe commands | **Supported** | hooks block on every invocation |
| Less hallucinated project type | **Supported** | marker-file detection, not LLM guess |
| Less over-broad work | **Partially supported** | budget measured but split-rule only advisory |
| Fewer user prompts | **Not supported** | CLI is manual; no auto-invocation |
| Lower token consumption | **Not supported** | no token optimization exists |
| Lower output verbosity | **Not supported** | AKAR does not control AI output style |
| Better skill/tool choice | **Not supported** | skill registry is inventory-only, no selection |
| Better self-awareness of environment | **Partially supported** | NEXT_RUN provides context but user must relay it |
| Better memory handling | **Not supported** | no memory engine or cross-session optimization |

**Summary:** AKAR reliably reduces 9 categories of negative AI behavior through its
advisory discipline loop. It has partial effect on 3 categories (diff size, fresh-user
confusion, over-broad work) where it provides measurement but not enforcement. It has
no effect on 4 categories (prompt count, token consumption, output verbosity,
skill/tool/memory optimization) because these require integration AKAR does not have.

## 17. Current User Burden

### Install/Setup Burden
- `cargo install --path .` or equivalent (Rust toolchain required)
- Manual PATH configuration
- Running `akar init` in each project
- Manual `.gitignore` decision for `.akar/`

### Manual Claude Hook Wiring Burden
- Running `akar hooks --install`
- Editing `~/.claude/settings.json` to register PreToolUse hook
- Understanding the hook template syntax
- This is a one-time-per-project cost but not zero

### Per-Session Manual Burden
- Running `akar preflight --snapshot "<task>"` before each task
- Running `akar request "<task>"` to compile NEXT_RUN.md
- Running `akar request --check` to validate
- Inspecting NEXT_RUN.md or relaying it to Claude
- Running `akar postmortem --diff --baseline` after each task
- Running `akar learn --list` / `akar learn --resolve` as needed

### What v0.41 Improved
- Bootstrap "templates directory not found" no longer sounds like failure
- Init `.akar/ notice` explains dirty-tree situation before user encounters it
- Doctor fresh-project checks (missing NEXT_RUN/baseline) are now PASS with
  action-oriented wording instead of WARN

### What Burden Remains
- **No auto-invocation** — user must remember and run each command manually
- **No dirty-tree auto-detection** — `.akar/` dirties tree on first use; user must
  discover and decide `.gitignore` policy themselves
- **No hook auto-wiring** — user must edit JSON config manually
- **No cross-session memory** — AKAR's own memory files exist but don't auto-inject
  into Claude sessions
- **No postmortem auto-trigger** — user must remember to run it after each task
- **Governor is passive** — produces decisions but doesn't enforce them
- **CLI is verbose** — 7+ commands per task cycle in full advisory mode

## 18. Current AI-Facing Usefulness

| Question | Answer | Detail |
|---|---|---|
| Does AKAR make Claude aware of project kind? | **Yes, but indirectly** | NEXT_RUN.md includes project kind; user must relay it |
| Does AKAR make Claude aware of verification commands? | **Yes, but indirectly** | NEXT_RUN.md includes project-appropriate commands |
| Does AKAR make Claude aware of task objective? | **Yes, but indirectly** | NEXT_RUN.md includes task-threaded objective |
| Does AKAR make Claude aware of stop conditions? | **Yes, but indirectly** | NEXT_RUN.md includes budget and stop rules |
| Does AKAR reduce hallucinated commands? | **Yes, indirectly** | Project-kind-aware allowed commands lists prevent Cargo leakage into Node projects |
| Does AKAR force concise output? | **No** | No mechanism to control AI output style |
| Does AKAR force smaller code? | **Partially** | Budget in NEXT_RUN.md is advisory; split-rule from postmortem adds pressure |
| Does AKAR guide tool choice? | **No** | No skill/tool selection mechanism |
| Does AKAR automatically inject context into Claude? | **No** | User must manually relay NEXT_RUN.md or Claude must read it |
| Does AKAR require user copy/paste/manual invocation? | **Yes** | All AKAR output requires user action to reach the AI |

**Honest assessment:** AKAR today is **CLI-useful but not AI-auto-injected.** It
prepares excellent discipline-tuned context that, when followed, meaningfully reduces
negative AI behaviors. But the user must actively bridge the gap between AKAR's output
and Claude's input. AKAR does not automatically inject its guidance into Claude Code
sessions — the user either copy-pastes NEXT_RUN.md content or relies on Claude to
discover and read `.akar/NEXT_RUN.md` from the filesystem.

This is by design (AKAR does not edit Claude Code settings or modify session state),
but it means AKAR's AI-facing usefulness depends entirely on user diligence. In
dogfood trials where the user was disciplined about using the full advisory loop,
results were consistently good. In a hypothetical scenario where the user runs
`akar init` once and never touches it again, AKAR provides zero ongoing value.

## 19. Current Limitations

1. **No auto-injection into AI sessions** — NEXT_RUN.md content must be manually
   relayed or discovered by the AI from the filesystem
2. **No enforcement** — budgets, split rules, and governor decisions are all advisory;
   the AI can ignore them
3. **No cross-session memory optimization** — memory files exist but don't auto-load
4. **No dirty-tree recovery guidance** — user must figure out `.gitignore`/commit
   policy themselves; advisory exists but no guided workflow
5. **Manual CLI overhead** — 7+ commands per task cycle
6. **Windows-only validation** — all dogfood trials on Windows
7. **No concurrent session support** — state files assume single-user, single-session
8. **No large-repo performance data** — all trials on tiny fixtures (<20 files)
9. **Hook wiring is manual and error-prone** — JSON config editing required
10. **No non-English project support** — README.md scanning assumes English command
    literals

## 20. What Is Proven

- The advisory discipline loop (baseline → budget → postmortem) works across 4
  project types
- Project-kind-aware NEXT_RUN compilation eliminates hallucinated toolchain commands
- Verification discovery hints correctly surface real commands from local files
- Live PreToolUse hooks correctly classify and block dangerous commands
- Hook events correctly route to the session-root project's log
- Multi-task sessions maintain correct state across task boundaries
- The eval suite (28 cases) reliably catches regressions
- The test suite (508 tests) provides broad coverage of CLI behavior
- Embedded hook templates make AKAR usable without the source tree
- Fresh-user wording (v0.41) reduces first-run friction

## 21. What Is Unproven

- Cross-platform behavior (Linux, macOS)
- Hook integration with Python, Rust, or Unknown projects in live sessions
- Multi-task sessions with non-Node projects
- Sessions longer than 3 sequential tasks
- Large-repo performance (baseline size, scan speed, hook latency)
- Concurrent or parallel sessions
- Non-English project discovery
- Real-world adoption by users other than the AKAR author
- Whether users outside the author will tolerate the manual CLI overhead
- Whether the advisory-only approach scales to complex multi-file refactors

## 22. What Recent Planning Drifted Into

After v0.41.0, planning discussions drifted into speculative architectural ideas:

- **AI-first capsule** — a self-contained task unit with auto-execution
- **Token optimization** — automatic context window management
- **Host adapters for Codex/OpenCode** — multi-platform coding agent support
- **Skill conflict resolver** — automatic skill selection and conflict resolution
- **Bounded autopilot** — limited autonomous task execution
- **Memory optimization** — cross-session knowledge compression and retrieval
- **Model/API calls** — LLM integration for classification or decision-making

**Why this was drift:** None of these ideas have evidence from dogfood trials
supporting them. The current advisory loop has proven value without any of them.
Adding execution, autonomy, model calls, or multi-agent orchestration would
fundamentally change AKAR's risk profile from "advisory discipline layer" to
"autonomous coding agent" — a category change that requires different safety
guarantees and evidence standards.

**What should have happened instead:** The planning should have stayed focused on
closing proven gaps — dirty-tree recovery, cross-platform validation, hook wiring
simplification — rather than inventing new product categories.

## 23. Ideas Valid but Not Accepted Yet

### AI-First Capsule
- **Why attractive:** Would package task context, budget, and verification into a
  single unit the AI could consume without user relay
- **Why not current AKAR:** AKAR does not execute anything; a capsule that
  auto-executes would cross the advisory→autonomous boundary. No evidence that
  auto-execution is needed — the manual loop works.
- **Evidence required before building:** At least 3 external users reporting that
  manual NEXT_RUN relay is a blocking friction point; at least one dogfood trial
  showing that auto-injection (not auto-execution) improves outcomes without
  increasing risk.

### Token Optimization
- **Why attractive:** Could reduce AI API costs and improve context utilization
- **Why not current AKAR:** AKAR has no visibility into token consumption, no model
  API access, and no mechanism to trim or restructure AI context. This requires
  integration AKAR does not have.
- **Evidence required before building:** Token waste measured and quantified in
  AKAR-guided sessions vs. unguided sessions; a specific optimization strategy with
  measured improvement.

### Host Adapters for Codex/OpenCode
- **Why attractive:** Would make AKAR's discipline model available across multiple
  coding agents
- **Why not current AKAR:** The hook model is Claude Code-specific (PreToolUse JSON
  schema). Supporting other agents requires understanding their hook/plugin APIs,
  which are not yet stable or documented. Zero dogfood evidence that multi-agent
  support is needed.
- **Evidence required before building:** At least one user actively using both Claude
  Code and another coding agent who reports that AKAR's absence in the other agent
  causes measurable degradation.

### Skill Conflict Resolver
- **Why attractive:** AKAR has a skill inventory but doesn't help the AI choose
  skills or resolve conflicts
- **Why not current AKAR:** Skill conflict resolution requires understanding of skill
  semantics and task context that AKAR doesn't model. The skill registry is an
  inventory, not a resolver.
- **Evidence required before building:** Documented cases where skill conflicts caused
  measurable harm in AKAR-guided sessions; a specific resolvable conflict pattern.

### Bounded Autopilot
- **Why attractive:** Could reduce manual CLI overhead by auto-running preflight,
  request, postmortem in sequence
- **Why not current AKAR:** Autopilot implies AKAR making decisions about when to
  snapshot, when to measure, and when to stop — crossing from advisory to executive.
  No evidence that manual overhead is the primary adoption blocker.
- **Evidence required before building:** External user data showing that manual CLI
  overhead is the #1 reason for non-adoption; a bounded autopilot design with clear
  safety boundaries that doesn't auto-execute project code.

### Memory Optimization
- **Why attractive:** Could make AKAR's project memory files more useful across
  sessions
- **Why not current AKAR:** Memory files exist but are passive — the AI discovers
  them from the filesystem or not at all. Optimization without auto-injection is
  premature.
- **Evidence required before building:** Evidence that AI models are reading and
  using AKAR memory files today; a specific memory staleness or bloat problem.

## 24. Evidence-Backed v1.0 Boundary

### What Must Be True Before v1.0
1. **Cross-platform validation** — at least one full dogfood trial each on Linux
   and macOS
2. **All four project lanes proven with live hooks** — currently only Node has live
   hook evidence (v0.36); Rust, Python, Unknown need hook-integrated dogfood
3. **Dirty-tree recovery guidance** — user can run a command that diagnoses why the
   tree is dirty, distinguishes AKAR-state from real changes, and provides clear
   next steps (currently only advisory text exists)
4. **At least one external (non-author) user** has completed the full advisory loop
   and reported results
5. **No regressions** — 508+ tests, 28/28 eval, all dogfood lanes re-verified

### What Must Remain Out of Scope for v1.0
- Auto-execution of any kind (mission, verify, discovered commands)
- Model/API calls
- Token optimization
- Multi-agent orchestration
- Codex/OpenCode adapters
- Skill selection/resolution
- Memory optimization engines
- Autopilot or autonomous modes
- Cloud sync or telemetry exfiltration

### What Should Be Postponed to v1.1+
- Capsule system (needs auto-injection design first)
- Token optimization (needs measurement baseline)
- Host adapters (needs multi-agent demand evidence)
- Skill resolver (needs conflict pattern evidence)
- Autopilot (needs safety boundary design)
- Memory engine (needs auto-injection design first)

### What User Experience v1.0 Should Promise
- `akar init` works on any project (Rust/Node/Python/Unknown) on any platform
- `akar doctor` honestly reports environment status without false alarms
- `akar request "<task>"` produces project-appropriate, validated NEXT_RUN.md
- `akar hooks --install` + manual wiring blocks dangerous commands in live sessions
- `akar preflight --snapshot` + `akar postmortem` measures and budgets diffs
- `akar status` accurately reflects session state
- `akar eval` runs 28+ contract tests passing
- Dirty-tree situations have clear recovery guidance

### What User Experience v1.0 Should NOT Promise
- "AKAR will run your tests for you"
- "AKAR will automatically keep your AI on track"
- "AKAR will reduce your token costs"
- "AKAR will choose the right tools"
- "AKAR will work with any coding agent"
- "AKAR is a set-and-forget solution"

## 25. Recommended Next Release

**v0.43.0 Dirty-Tree Recovery Guidance.**

Reasoning:
- Dirty-tree handling is the most-repeated friction across all external dogfood trials
  (v0.24, v0.27, v0.33, v0.39, v0.40)
- The v0.28 AKAR-state advisory and v0.41 `.akar/ notice` provide text guidance but
  no structured workflow
- A `dirty-tree` diagnostic command that categorizes every dirty/untracked entry
  (AKAR-state, build-artifact, source-change, unknown) and provides category-specific
  guidance would directly reduce user burden
- This is a read-only diagnostic — no new execution risk
- It closes a proven gap rather than opening a speculative new direction
- It moves AKAR closer to v1.0 trust hardening

Alternative allowed only if justified by new evidence: **v0.43.0 Cross-Platform
Hook Validation** (Linux and macOS dogfood trials).

**Explicitly NOT recommended:** capsules, Codex/OpenCode adapters, skill resolver,
token optimizer, autopilot, memory engine, model integration.

## 26. Honest Conclusion

AKAR v0.41.0 is a **stable advisory alpha** that does exactly what it claims: prepares
scoped session instructions, measures diffs, surfaces verification guidance, blocks
dangerous commands through optional hooks, and records local evidence. Six dogfood
trials across four project types prove the advisory loop works. Live hook integration
is proven for Node.

AKAR does not execute project code, does not call models, does not auto-inject context
into AI sessions, and requires significant manual CLI discipline from the user. These
are design choices, not bugs — but they mean AKAR's value depends entirely on user
diligence.

The path to v1.0 is clear: close proven gaps (dirty-tree recovery, cross-platform
validation, external-user evidence) without adding speculative features. The recent
planning drift into capsules, adapters, and autopilot was premature — those ideas may
have merit someday but are not justified by current evidence.

AKAR's core insight — that a local, read-mostly, project-aware discipline layer can
meaningfully reduce negative AI coding behaviors without executing anything itself —
is sound and proven. The next release should harden what exists, not invent what
doesn't.
