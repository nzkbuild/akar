# AKAR v0.21.0 — Current Reality Audit

**Date:** 2026-07-06
**Auditor:** post-v0.20.0 audit-only session
**Scope:** audit AKAR v0.20.0 against the original plan, the v1 architecture freeze, and the actual implementation. No fixes. No features. No runtime behavior changes.
**Method:** read every `src/*.rs` module, `main.rs` dispatch, all `docs/foundation/*`, `docs/architecture/*`, `docs/audits/*`, README, CHANGELOG, INSTALL, OPERATING_MODEL, and the master roadmap; ran the full Phase 0 verification matrix.

---

## 1. Executive verdict

AKAR v0.20.0 is an **honest, advisory-only local CLI** that does exactly what its
current README claims and nothing more. The drift and overclaims called out in
the v0.3.0 reality audit and the scope-drift report have, for the most part,
been **resolved** between v0.4.0 and v0.20.0. The system is coherent, well-tested
(401 unit tests, 28 evals, clean release build), and the one piece of real
runtime enforcement — the Claude Code PreToolUse safety hook — was genuinely
proven end-to-end across v0.8.0–v0.11.0.

The remaining problems are **not overclaims in the current README**. They are:

1. **Latent dead code in a live path.** `cmd_request` calls two NEXT_RUN.md
   writers back-to-back; the second unconditionally overwrites the first, so
   `request_intelligence::write_next_run` is shadowed and its "never overwrite"
   guard is moot. Not a correctness bug for users today (the compiled v0.19.0
   prompt is the one that survives, which is what we want), but it is dead code
   inside a runtime command path and a trap for future maintainers.
2. **Two aspirational subsystems that are wired only into tests, not runtime.**
   `safety::govern_dependency` and `safety::check_migration` are never called
   from `main.rs` — they exist only to satisfy evals #19 and #20. They are
   advertised in the master roadmap as Phase 13 deliverables.
3. **A stub doctor.** `doctor.rs` is still a directory-existence check. The
   Phase 5 promise (hook/skills/memory/project/known-failure checks) never
   landed. `akar doctor` cannot actually diagnose a broken hook or a corrupt
   event log — it only notices missing directories.
4. **Stale docs that were never refreshed after v0.1.1.** `PRODUCT_ROADMAP.md`
   still calls v0.1.1 "Current" and lists L6/L7 self-optimization, multi-model
   routing, and a self-evolving OS as upcoming. `INSTALL.md` says
   `Expected: akar 0.10.0`. `OPERATING_MODEL.md` references `/akar-preflight`
   and `/akar-doctor-fix` slash commands that do not exist as files. These are
   drift artifacts, not current-feature overclaims.
5. **A second hardcoded diff-budget table.** `diff_budget.rs` keeps its own
   numeric budget table despite a comment asserting "no second budget table."
   The two tables have not yet diverged, but the structure invites silent drift.

**Verdict:** v0.20.0 is **safe to dogfood in advisory mode on an external repo**
with the pre-tool-call hook active, provided the user understands that `akar run`
and `akar mission` print strategy and record telemetry but never execute the
task. It is **not** ready to be positioned as the master roadmap's "A5/A6 full
autonomy runtime" — that product was never built and the freeze correctly
forbids building it before a v1 design review.

---

## 2. Current baseline

Confirmed in Phase 0 (no files modified):

| Check | Result |
|---|---|
| `git log --oneline -5` | HEAD = `6ecb1a9 feat: validate AKAR next-run prompt contract` |
| `git status` | working tree clean; 15 commits ahead of origin/master (unpushed) |
| `cargo run -- --version` | `akar 0.20.0` |
| `cargo test` | **401 passed; 0 failed; 0 ignored** |
| `cargo run -- eval` | **28/28 PASS** |
| `cargo run -- status` | `status: HEALTHY`, readiness `READY`, governor `READY` |
| `cargo run -- governor --json --no-exit-code` | valid JSON, exit 0 |
| `cargo run -- request` | writes `.akar/NEXT_RUN.md`, prints governor block |
| `cargo run -- request --check` | `NEXT_RUN check: PASS` |
| `cargo run -- hooks --check` | `status: PASS`, both templates found |
| `cargo build --release` | clean, zero warnings |

Baseline is solid. The version is v0.20.0. Tests pass. Tree is clean.

---

## 3. Original plan vs current system

There are **two competing source documents**, and they describe different products:

### 3a. `AKAR_MASTER_ROADMAP_v1.0_REVISED.md` — the original full plan

This roadmap describes an **execution-capable autonomy runtime**: A5/A6 full
autopilot, a mission compiler that executes code, model/gateway drift detection,
circuit breakers, dependency governor, migration safety, an OS-style kernel
with skill authority hierarchy, L6 self-optimization, L7 self-evolving RFC
drafts. It has 37 phases and a "v1.0 Definition of Done" that includes
"diff budget prevents overcoding," "dangerous mode is safer," "rollback works."

### 3b. `docs/architecture/AKAR_V1_ARCHITECTURE_FREEZE_PROPOSAL.md` — the narrowed scope

The freeze (written at v0.3.0, after the scope-drift report) explicitly **forbids**
most of the roadmap: no code execution, no model API calls, no daemon, no
auto-apply of learning patches, no skill disabling, no settings.json mutation.
It narrows AKAR to a local advisory CLI that classifies, reports, records, and
suggests. It lists candidates for deletion (`SessionFingerprint`,
`calibrate_from_prompt`, `circuit_breaker.rs`, etc.) and a 3-release plan
(v0.4 honest scaffold, v0.5 real hooks, v0.6 real diff measurement).

### 3c. What actually happened (v0.3.0 → v0.20.0)

**The freeze won.** The implementation followed the freeze proposal, not the
master roadmap:

| Freeze proposal item | Status by v0.20.0 |
|---|---|
| Delete `circuit_breaker.rs` | ✅ Done in v0.4.1 |
| Remove `SessionFingerprint` / `detect_drift` / `calibrate_from_prompt` | ✅ Done in v0.4.0 (verified: zero source references) |
| Simplify `design.rs` to DNA-check only | ✅ Done in v0.4.0 |
| Deprecate event-count request pressure (require explicit used/limit) | ✅ Done in v0.4.0 |
| Scope skill conflict to project-local, not all 200 global skills | ✅ Done in v0.4.0 |
| Add LEARNING_PATCHES.md + NEXT_RUN.md to .gitignore | ✅ Done in v0.4.0 |
| Fix README overclaims ("enforces" → "reports", etc.) | ✅ Done in v0.4.0 |
| v0.5 real hook integration (templates + --check + --install) | ✅ Done in v0.5.0 |
| v0.6 real diff budget measurement (postmortem --diff) | ✅ Done in v0.6.0 |
| v0.8–v0.11 prove the baseline loop end-to-end with auto-hook | ✅ Done |
| No execution engine, no model API, no daemon before v1 review | ✅ Holds |

**The master roadmap was never updated to reflect the freeze.** It still
describes the unbuilt product. This is the single largest doc-vs-reality gap in
the project, and it is a *planning* gap, not a code gap. The code is honest;
the planning doc is stale.

### 3d. Net

AKAR v0.20.0 is **the freeze proposal's vision, largely realized**, sitting
inside a repo whose master roadmap still describes a different, unbuilt system.
The audit's job is to record that the code follows the freeze, the docs lag, and
the roadmap's execution-era promises must not be silently inherited by future
sessions as if they were commitments.

---

## 4. Command surface reality table

Every command dispatched in `main.rs`, audited against actual behavior. "Real"
= performs a side effect beyond printing to stdout. "Advisory" = prints a
report only.

| Command | Real behavior | Advisory behavior | Reads files | Writes files | Mutates git | Depends on Claude Code | Safe for dogfood | Notes |
|---|---|---|---|---|---|---|---|---|
| `--version` / `-V` | Prints `akar <version>` | — | no | no | no | no | ✅ | Trivial |
| `status` | Prints health snapshot | Aggregates doctor/skills/request/governor/readiness | yes (.akar/, git status) | no | no (read-only git) | no | ✅ | Does NOT write NEXT_RUN.md (only prints governor) |
| `doctor` | Checks 3 dirs exist (`config::validate`) | — | yes (dir existence) | no | no | no | ✅ | **Still a stub** — no hook/skill/memory/project checks |
| `doctor --fix` | Creates missing dirs; copies missing templates (backs up first) | — | yes | yes (.akar/, .bak backups) | no | no | ✅ | `SafeFix::NormalizePath` variant never constructed — dead |
| `bootstrap` | Creates `.akar/` + `~/.claude/akar/`; copies 9 `.md` templates idempotently | — | yes (templates/) | yes (.akar/*.md, never overwrites) | no | no | ✅ | Solid, well-tested |
| `init` | bootstrap + doctor + shell detect + next-steps guide | — | yes | yes (via bootstrap) | no | no | ✅ | `--claude` only prints instructions, does not wire hooks |
| `verify` | **Spawns `cargo build`/`cargo test` (or npm) via `std::process::Command`** | — | yes (Cargo.toml/package.json) | no | no | no | ⚠️ | **The only command that executes subprocesses other than git.** Real execution. |
| `eval` (no arg) | Runs 28 behavioral checks across modules | — | yes | no | no | no | ✅ | Several self-fulfilling evals (see §9) |
| `eval "<prompt>"` | — | Classifies prompt, prints contract | no | no | no | no | ✅ | Same as `akar preflight` minus the extras |
| `safety "<cmd>"` | Classifies risk; **exits 2** for BLOCKED commands (this is what the hook reads) | Prints safe-alternative playbook | no | no | no | no (but the hook calls it) | ✅ | `govern_dependency`/`check_migration` NOT reachable from here |
| `skills` | Scans global + project `.claude/commands/`; writes SKILL_INVENTORY.md | Reports role/conflicts/recommended mode | yes | yes (.akar/SKILL_INVENTORY.md) | no | no | ✅ | Enforcement = zero; report only |
| `calibrate` | — | Reads env vars (ANTHROPIC_MODEL etc.), prints hardcoded model profile | no | no | no | no | ✅ | Never persists; `gateway` field always "unknown" (minor bug) |
| `preflight "<task>"` | — | Combines contract + request + skills + verify-recipe into strategy report | yes | no | no | no | ✅ | `needs_execution` hardcoded false |
| `preflight --snapshot "<task>"` | Writes `.akar/DIFF_BASELINE.json` (refuses if tree dirty) | — | yes (git HEAD) | yes (DIFF_BASELINE.json) | no (read-only git) | no | ✅ | Refuses dirty tree — good |
| `request` | Writes `.akar/NEXT_RUN.md` (compiled 11-section prompt, always overwrites) | Prints request mode + governor block | yes | yes (NEXT_RUN.md) | no | no | ⚠️ | **Dual-writer bug**: `write_next_run` then `write_governor_next_run` — first is shadowed (see §7) |
| `request --check` | — | Validates NEXT_RUN.md against contract; exits 0/non-zero | yes | no | no | no | ✅ | Read-only; never writes. v0.20.0 addition |
| `governor` | — | Prints governor decision (human) | yes | no | no | no | ✅ | Does NOT write NEXT_RUN.md |
| `governor --one-line` | — | `DECISION<TAB>SUGGESTED_PROMPT` | yes | no | no | no | ✅ | |
| `governor --json` | — | Single JSON object | yes | no | no | no | ✅ | std-only JSON escaping |
| `governor --telemetry` | Appends one event to EVENT_LOG.jsonl (opt-in) | — | yes | yes (EVENT_LOG.jsonl) | no | no | ✅ | Default writes nothing; suggested prompt NOT logged |
| `governor --no-exit-code` | Forces exit 0 | Same output | yes | no | no | no | ✅ | Composable with --one-line/--json/--telemetry |
| `mission "<prompt>"` | Appends one JSONL event to EVENT_LOG.jsonl | Runs state machine; Execute/Verify/MemoryUpdate are scaffold (log only) | yes | yes (EVENT_LOG.jsonl) | no | no | ⚠️ | **No execution.** Verify state calls `detect_recipe` (file inspection), never `run_recipe` |
| `run "<task>"` | Appends mission event (via mission.rs) | Chains doctor→preflight→mission→postmortem; prints "advisory scaffold mode" | yes | yes (EVENT_LOG.jsonl) | no | no | ⚠️ | End-to-end report-only. Explicit "not done by AKAR" block in output |
| `postmortem` | — | Reads EVENT_LOG.jsonl, classifies latest outcome | yes | no | no | no | ✅ | Hand-rolled JSON parser; outcome coupled to mission summary format |
| `postmortem --diff` | Appends learning patch to LEARNING_PATCHES.md on EXCEEDED | Measures `git diff HEAD --numstat`; compares to budget | yes | yes (LEARNING_PATCHES.md) | no (read-only git) | no | ✅ | Reports only — does not enforce/block/revert |
| `postmortem --diff --baseline` | Appends learning patch on EXCEEDED | Measures diff from saved baseline HEAD; PASS/EXCEEDED/UNKNOWN | yes | yes (LEARNING_PATCHES.md) | no | no | ✅ | Proven end-to-end in v0.8–v0.11 |
| `learn` | Writes a generic learning patch stub | — | yes | yes (LEARNING_PATCHES.md) | no | no | ✅ | Patch is boilerplate, not failure analysis; never emits SPLIT_RULE_MARKER |
| `learn --list` | — | Prints active/resolved patch counts | yes | no | no | no | ✅ | Read-only |
| `learn --resolve` | Marks all active patches `status: resolved` with timestamp | — | yes | yes (LEARNING_PATCHES.md) | no | no | ✅ | Leaves file in place; does not delete |
| `hooks` | — | Prints hook paths + manual install instructions | yes | no | no | no | ✅ | |
| `hooks --check` | — | Verifies both templates exist and contain `akar safety` call | yes | no | no | no | ✅ | |
| `hooks --install` | Copies templates into `.akar/hooks/` after "INSTALL" confirmation (backs up first) | — | yes | yes (.akar/hooks/) | no | no | ✅ | Does NOT modify ~/.claude/settings.json |
| `telemetry` | — | Prints EVENT_LOG.jsonl summary | yes | no | no | no | ✅ | **Not in audit's required list but is a real command** |

**Key surprises:**

- **`akar verify` is the only command that runs non-git subprocesses** (`cargo`/`npm`).
  Every other "real" side effect is local file writes or read-only `git`. The
  README's "does not execute code changes" framing is true for the *project*,
  but `verify` does run build/test commands. This is intentional and honest
  (it's a verification runner), but it is the one command whose blast radius
  extends beyond `.akar/`.
- **`akar telemetry` exists as a command** but was omitted from the audit's
  required command list. It is real, read-only, and safe.
- **`governor` never writes NEXT_RUN.md**; only `request` does. This is correct
  and tested, but easy to misremember.

---

## 5. Activation / on-off matrix

How each subsystem is turned on/off, who owns the switch, and how it fails.

| Subsystem | Default state | How to turn on | How to turn off | Owner of the switch | Evidence command | Failure mode |
|---|---|---|---|---|---|---|
| Claude Code PreToolUse hook | OFF | User manually registers hook in `~/.claude/settings.json` pointing at `pre-tool-call.{sh,ps1}` | Remove the settings.json entry | User (AKAR never edits settings.json) | `akar hooks --check` (verifies template validity only — cannot tell if registered) | If `akar` not on subprocess PATH → hook logs ALLOW and skips safety (fails open, by design) |
| AKAR safety classification | ON (always, when `akar safety` is called) | Call `akar safety "<cmd>"` | Don't call it | Caller (user or hook) | `akar safety "git push --force"` | Substring matching — can false-positive (e.g. "cat" in "concatenate") or be evaded |
| Hook blocking | ON when hook is registered + `akar` on PATH | Hook script exits 2 | Hook not registered, or `akar` absent from PATH | Hook script + exit code 2 | `.akar/HOOK_EVENTS.jsonl` (decision=BLOCK) | Fails open if `akar` not found (ALLOW + warning) |
| Governor decisions | ON (computed on every `status`/`governor`/`request` call) | Call any of those commands | Don't call them | Caller | `akar governor` | UNKNOWN (exit 30) if git unavailable |
| Governor exit codes | ON for `akar governor` only | Run `akar governor` (not --no-exit-code) | `--no-exit-code` forces 0 | `main.rs` `cmd_governor` | `echo $?` / `$LASTEXITCODE` | Does NOT apply to `status` or `request` (they keep existing behavior) |
| Governor telemetry | OFF by default | `--telemetry` flag OR `AKAR_GOVERNOR_TELEMETRY=1` env var | Omit both | User (opt-in) | `.akar/EVENT_LOG.jsonl` (`event:"governor"`) | Returns None if `.akar/` absent; suggested prompt never logged |
| NEXT_RUN compiler | ON when `akar request` is called | Run `akar request` | Don't run it (or delete `.akar/NEXT_RUN.md`) | Caller | `.akar/NEXT_RUN.md` | Overwrites unconditionally — see dual-writer note §7 |
| NEXT_RUN validator | ON when `akar request --check` is called | Run `akar request --check` | Don't run it | Caller | exit 0/1 | Returns FAIL (exit 1) if file missing or malformed |
| Learning patches (write) | ON when `learn` runs, or `postmortem --diff` EXCEEDED | Run those commands | Don't run them | Caller | `.akar/LEARNING_PATCHES.md` | `learn` patches are boilerplate; only `--diff` EXCEEDED emits the split-rule marker |
| Learning patch lifecycle (active/resolved) | ON (governor reads it) | `learn --resolve` marks resolved | Edit file to `status: active` | User / `learn --resolve` | `akar learn --list` | Old statusless entries treated as active; resolved entries stop affecting governor |
| Diff baseline | OFF until snapshot | `akar preflight --snapshot "<task>"` (requires clean tree) | Delete `.akar/DIFF_BASELINE.json` | User | `akar status` (readiness section) | Refuses to snapshot a dirty tree |
| Diff measurement (postmortem) | OFF until `postmortem --diff` | Run `akar postmortem --diff [--baseline]` | Don't run it | Caller | `akar postmortem --diff --baseline` | UNKNOWN if git fails; reports only, never enforces |
| mission/run scaffold | ON (scaffold mode — no execution) | Run `akar mission`/`akar run` | n/a (always scaffold) | Code (freeze) | EVENT_LOG.jsonl | Cannot execute even if asked — by design |
| model/gateway behavior | OFF (display only) | `akar calibrate` reads env vars | Unset env vars | User env | `akar calibrate` | `gateway` always "unknown"; no persistence, no API call |
| skill scanner | ON when `akar skills`/`status` called | Run `akar skills` | Don't run it | Caller | `.akar/SKILL_INVENTORY.md` | Enforcement = zero; report only |
| eval suite | OFF until invoked | `akar eval` | Don't run it | Caller | `akar eval` | Several self-fulfilling checks (see §9) |
| verify recipe execution | OFF until `akar verify` | Run `akar verify` | Don't run it | Caller | `akar verify` | Spawns cargo/npm — the one real execution surface |

**Net activation finding:** AKAR's default posture is **everything off until
explicitly invoked**. The only always-on-at-runtime enforcement (the PreToolUse
hook) is owned by the user, not AKAR, and AKAR correctly refuses to install it
automatically. This matches the freeze. There is no hidden always-on process.

---

## 6. Module wiring audit

For each `src/*.rs` module: is it called from `main.rs`, called by another
runtime module, test-only, or dead? Recommendation for v1.

| Module | Called from main.rs | Called by runtime modules | Test-only paths | Wiring | Keep / Simplify / Delete / Postpone | Reason |
|---|---|---|---|---|---|---|
| `main.rs` | (dispatch) | — | 8 tests | Fully wired | Keep | Command hub |
| `config.rs` | yes (every cmd) | nearly all | — | Fully wired | Keep | `discover`/`redact`/`home_dir` are load-bearing. Solid. |
| `event_log.rs` | yes (status, telemetry, postmortem, learn, governor) | mission, request_intelligence, learn, loop_governor, skill_registry | — | Fully wired | Keep | Append-only JSONL, rotation, redaction. Freeze-listed. |
| `bootstrap.rs` | yes (cmd_bootstrap, init) | init | — | Fully wired | Keep | Idempotent, no-overwrite. Freeze-listed. |
| `backup.rs` | no (indirectly via safe_fix) | safe_fix | — | Fully wired (indirect) | Keep | backup/restore. Freeze-listed. |
| `safe_fix.rs` | yes (cmd_doctor --fix) | — | `NormalizePath` variant never constructed | Partially wired | Simplify | Delete the `NormalizePath` dead variant; keep `CreateMissingDir`/`CreateMissingTemplate` |
| `doctor.rs` | yes (cmd_doctor, status, init) | init | — | **Stub** | Simplify (postpone full Phase 5) | Still delegates to `config::validate` (3 dir checks). Either build the real checks or rename to "dir-check" honestly |
| `verify.rs` | yes (cmd_verify) | preflight, eval | `run_recipe` not called from mission scaffold | Fully wired (but `run_recipe` only reachable via `akar verify`, not via `run`/`mission`) | Keep | The one real execution surface. `detect_recipe` is file-inspection only. |
| `eval.rs` | yes (cmd_eval) | — | — | Fully wired | Keep (improve honesty — see §9) | 28 checks; several self-fulfilling |
| `contract.rs` | yes (cmd_eval, preflight, mission, request) | mission, preflight, eval | many dead enum variants | Fully wired | Simplify | `autonomy` always A5, `cost_mode` always Balanced, `confidence` always Medium — the variant enums are aspirational dead code. `verification_commands` hardcoded to cargo. |
| `safety.rs` | yes (cmd_safety) | hooks (tests only) | `govern_dependency`/`check_migration` never called from main | **Partially dead** | Simplify | `classify_command`/`check_secrets` are real and hook-critical. `govern_dependency`/`check_migration` are test+eval-only — either wire them or move to a clearly-labeled "policy library" |
| `skill_registry.rs` | yes (cmd_skills, status) | — | `format_registry`, `detect_duplicates` dead; `ClaudeBundled`/non-Active `SkillStatus`/`LibraryOnly` never produced | Partially wired | Simplify | `scan_multi`/`build_skill_report`/`write_skill_inventory` are live. Trim the dead enums/helpers. |
| `context_pack.rs` | no (not directly) | eval (#15), workflow? | — | **Weakly wired** | Simplify or postpone | Enumerates paths only (never reads contents). Only eval #15 exercises it. Not called from `run`/`mission` meaningfully. Either read contents (v1) or reduce to a path enumerator. |
| `model_profile.rs` | yes (cmd_calibrate) | — | — | Fully wired (display-only) | Simplify | Drift code removed ✅. But `gateway` always "unknown", `last_calibrated` always "never", `known_failure_patterns` always empty — vestigial fields. |
| `design.rs` | no (not from main) | eval (#16) | — | **Weakly wired** | Simplify | Simplified to DNA-check in v0.4.0 ✅. Only eval #16 calls it. Not in any user-facing command path except indirectly via eval. |
| `mission.rs` | yes (cmd_mission, via workflow) | workflow | — | Fully wired (scaffold) | Keep (frozen) | Execute/Verify/MemoryUpdate are scaffold by freeze design. Do not expand before v1 review. |
| `workflow.rs` | yes (cmd_run) | eval (#26,#27,#28) | — | Fully wired | Keep | Orchestrates the advisory chain. Honest "scaffold mode" labeling. |
| `preflight.rs` | yes (cmd_preflight) | workflow | — | Fully wired | Keep | `needs_execution` hardcoded false (stale v0.1.9 comment). |
| `postmortem.rs` | yes (cmd_postmortem, status) | workflow, request_intelligence, learn | — | Fully wired | Keep | Hand-rolled JSON parser; outcome coupled to mission summary format. |
| `learn.rs` | yes (cmd_learn) | loop_governor (has_active_split_rule_entry) | — | Fully wired | Keep | Lifecycle (active/resolved) is real. `build_patch` is boilerplate, not failure analysis. |
| `diff_budget.rs` | yes (cmd_status, cmd_postmortem, cmd_preflight) | loop_governor | `BudgetVerdict::as_str` dead | Fully wired | Simplify | **Second hardcoded budget table** (lines 214-225) despite "no second budget table" comment. Consolidate with contract.rs. |
| `hooks.rs` | yes (cmd_hooks) | — | — | Fully wired | Keep | `check_hooks`/`install_hooks`/`parse_hook_event`/`hook_decision` all real. |
| `loop_governor.rs` | yes (cmd_status, cmd_governor, cmd_request) | — | many tests (116) | Fully wired | Keep | The most-tested module. `write_governor_next_run` + `compile_next_run_prompt` + `validate_next_run` all live. |
| `foundation.rs` | yes (cmd_safety, cmd_status, cmd_postmortem, cmd_hooks) | loop_governor, diff_budget, hooks | `snapshot_required_playbook`/`repeated_block_playbook` marked dead_code but ARE called by loop_governor | Fully wired | Simplify | Remove the stale `#[allow(dead_code)]` annotations on functions that are actually used. |

**Cross-cutting wiring findings:**

1. **No module is fully dead.** Every module has at least one runtime or eval
   caller. The v0.4.0 cleanup removed the genuinely dead code
   (`circuit_breaker.rs`, drift detection). What remains is *partially* dead:
   dead enum variants, dead helper functions, and two policy functions
   (`govern_dependency`, `check_migration`) reachable only via evals.

2. **The weakest-wired modules are `context_pack.rs` and `design.rs`.** Both
   exist primarily to satisfy a single eval check (#15, #16) and are not on any
   user-facing command's critical path. They are not dead, but they are
   load-bearing only for the eval count.

3. **`verify.rs::run_recipe` is reachable but not from the `run`/`mission`
   chain.** It runs only when the user explicitly calls `akar verify`. The
   mission Verify state calls `detect_recipe` (file inspection) and explicitly
   does NOT call `run_recipe`. So "AKAR runs your tests for you during a
   mission" is false; "AKAR can run your tests if you ask it to" is true.

4. **`std::process::Command` appears in exactly two modules:** `verify.rs`
   (cargo/npm, only via `akar verify`) and `diff_budget.rs` (read-only git
   subcommands). No module spawns anything else. No network. No model API.
   This is the strongest evidence that the freeze holds.

---

## 7. Dead, partial, and drifted components

### 7a. Dead code (never reachable from a runtime command path)

| Item | Location | Status | Evidence |
|---|---|---|---|
| `SafeFix::NormalizePath` | `safe_fix.rs:24` | Dead variant | Never constructed anywhere; `apply_safe_fix` returns `"ok"` no-op. Only tested. |
| `safety::govern_dependency` | `safety.rs:219` | Dead in runtime | Never called from `main.rs`. Only eval #19 and tests call it. |
| `safety::check_migration` | `safety.rs:258` | Dead in runtime | Never called from `main.rs`. Only eval #20 and tests call it. |
| `skill_registry::format_registry` | `skill_registry.rs:275` | Dead in runtime | `#[allow(dead_code)]`; `cmd_skills` uses `format_skill_report` instead. Test-only. |
| `skill_registry::detect_duplicates` | `skill_registry.rs:253` | Dead in runtime | `#[allow(dead_code)]`; never called from runtime. Test-only. |
| `BudgetVerdict::as_str` | `diff_budget.rs:289` | Dead | `#[allow(dead_code)]`; never used. |
| `contract::TaskType::{Research,Answer,Inspect,Greenfield,Repair,Release}` | `contract.rs` | Dead variants | Never produced by `classify_prompt`. Only some TaskType variants are emitted. |
| `contract::Autonomy::{A0,A1,A2,A3,A4,A6}` | `contract.rs` | Dead variants | `autonomy` always hardcoded A5. |
| `contract::CostMode::{Fast,Deep,Autopilot,Emergency}` | `contract.rs` | Dead variants | `cost_mode` always Balanced. |
| `contract::Confidence::{Low,High}` | `contract.rs` | Dead variants | `confidence` always Medium. |
| `skill_registry::SkillSource::ClaudeBundled` | `skill_registry.rs:11` | Dead variant | `collect_skills` only ever produces Superpower/Project/Custom. |
| `skill_registry::SkillStatus` (non-Active) | `skill_registry.rs:18` | Dead variants | Scanner hardcodes `Active` for every skill. |
| `skill_registry::SkillRole::LibraryOnly` | `skill_registry.rs:39` | Dead variant | Never produced by `classify_role`. |

### 7b. Partial components (exist but do less than their name/roadmap implies)

| Component | What it claims (roadmap/doc) | What it actually does | Gap |
|---|---|---|---|
| `doctor` | Phase 5: config/hook/skills/memory/project/known-failure checks | Checks 3 directories exist | No hook health, no skills check, no memory validity, no known-failure patterns |
| `mission` (Execute/Verify/MemoryUpdate) | Roadmap state machine with execution | Logs "skipped in scaffold mode" | By freeze design — but the names imply more |
| `learn` (build_patch) | "Learning Intelligence: observes failures, patches behavior" | Maps a 4-variant Outcome enum to 2 canned rule strings | No root-cause analysis; never emits the split-rule marker that the governor reads |
| `context_pack` | "Hot/warm/cold tier context pack" | Enumerates file paths; never reads contents | No content, no eviction, no actual context delivery |
| `calibrate` / `model_profile` | "Model profile + gateway routing" | Reads env vars, returns hardcoded heuristic table | No persistence, no drift detection (removed), `gateway` always "unknown" |
| `verify` (in mission path) | "VERIFY: run verification recipe" | Calls `detect_recipe` (file inspection), never `run_recipe` | Mission never runs tests; only `akar verify` does |
| `skills` enforcement | "AKAR kernel beats skills, broken skills can be disabled" | Reports conflicts; recommends modes | Zero enforcement — by freeze design |

### 7c. Drifted components (code and a nearby comment/doc disagree)

1. **`diff_budget.rs` second budget table.** A comment at line 211 asserts
   "Uses existing contract.rs budget tiers — no second budget table," but
   `budget_for_task_name` (lines 214-225) maintains its own numeric table.
   The tables have not diverged yet, but the comment is false and the structure
   invites silent drift. **Real drift risk.**

2. **`foundation.rs` stale `#[allow(dead_code)]`.** `snapshot_required_playbook`
   and `repeated_block_playbook` are annotated `#[allow(dead_code)]` but are
   actually called by `loop_governor.rs` at runtime. The annotations are
   misleading — they suggest the functions are unused when they are live.

3. **`request_intelligence::write_next_run` is shadowed in `cmd_request`.**
   `cmd_request` (main.rs:817-827) calls `write_next_run` (which writes a
   continuation prompt only if NEXT_RUN.md does NOT exist), then immediately
   calls `write_governor_next_run` (which **unconditionally overwrites**
   NEXT_RUN.md with the compiled 11-section prompt). The first writer's output
   is always destroyed; its "never overwrite" guard is moot. The first writer
   is only reachable in Resume mode and even then its result is overwritten.
   **Latent dead code inside a live command path.** Not a user-facing bug
   (the compiled prompt is the one we want to survive), but a maintainer trap.

4. **`preflight.rs` stale version comment.** `needs_execution = false` carries
   a "not recommended automatically in v0.1.9" comment. The code is correct;
   the version string is stale.

5. **`model_profile.rs` vestigial fields.** `gateway` is always "unknown"
   (even though `detect_model` returns a gateway — the field is never populated
   from it; minor wiring bug), `last_calibrated` always "never",
   `known_failure_patterns` always empty. The struct shape implies more than
   the code delivers.

---

## 8. Docs vs reality

Findings classified as true / partially true / false / obsolete / missing-caveat.

### 8a. `README.md` — **largely true** (corrected in v0.4.0)

| Claim | Verdict |
|---|---|
| "A local advisory CLI" | True |
| "It does not write your code. It does not execute fixes. It does not edit project files." | Partially true — `akar verify` runs cargo/npm; `doctor --fix`/`bootstrap`/`hooks --install` write files (in `.akar/` and templates). The spirit (no project source edits) is true. |
| "Reports a diff budget so you know the expected scope (not enforced yet)" | True |
| "Detects skill conflicts ... and reports them" | True (report only) |
| "Records local-only telemetry after each mission" | True |
| "Writes generic learning notes if something degraded or failed" | True (generic — accurate) |
| "Prints hook installation instructions (does not install hooks automatically)" | True |
| Governor 8-decision table | True (matches `loop_governor.rs`) |
| Governor exit-code table | True |
| Next-run prompt 11 sections | True (matches `compile_next_run_prompt`) |
| `request --check` validator description | True |
| Learning patch lifecycle (active/resolved) | True |
| Example output shows `akar 0.3.0` | Stale — should be 0.20.0 (cosmetic) |
| Commands table omits `governor` row | Missing command in the table. `telemetry` is present. |

### 8b. `INSTALL.md` — **stale version string**

- Line 45: `Expected: akar 0.10.0` — **false**, should be 0.20.0. (The v0.11.0
  changelog claims this was fixed once; it has drifted again.)
- Version-compatibility table stops at v0.9.x — missing v0.10–v0.20 entries.
- Otherwise accurate.

### 8c. `OPERATING_MODEL.md` — **partially obsolete**

- Section G lists `/akar-preflight` and `/akar-doctor-fix` slash commands —
  **these files do not exist** in `.claude/commands/`. Only `akar-bootstrap`,
  `akar-doctor`, `akar-eval`, `akar-mission`, `akar-status`, `akar-verify`
  exist. **Overclaim.**
- Section G describes `hooks/pre-commit-akar.{sh,ps1}` running `akar doctor`
  before each commit. These files exist in `hooks/` but are the **older v0.1-era
  design**; the current, proven hook is `templates/hooks/pre-tool-call.{sh,ps1}`
  (PreToolUse, not pre-commit). The doc does not mention the pre-tool-call hook.
  **Obsolete.**
- Section E says "In v0.2.x, AKAR is in scaffold mode ... Real execution is
  planned for v0.3+." — **obsolete** (we are at v0.20.0; execution is still
  scaffold by freeze, not by version lag).
- Section H "Circuit breaker: Removed in v0.4.1" — accurate.
- Section F "passive runtime ... STATE.md is updated after each session" —
  AKAR does not automatically update STATE.md after a session; it is a
  template the user edits. Slightly overclaimed.

### 8d. `AKAR_MASTER_ROADMAP_v1.0_REVISED.md` — **largely obsolete/aspirational**

This is the largest drift. The roadmap describes an execution-capable OS that
was never built and is explicitly forbidden by the freeze. Specific overclaims
vs. reality:

- "v1.0 = reliable enough for daily A5/A6 full-autonomy use" — AKAR cannot
  execute anything; it is advisory. The autonomy modes A0–A6 are dead enum
  variants (always A5).
- "Mission Compiler handles multi-step missions with branching" (v0.2.0 item)
  — never built; mission is a linear scaffold state machine.
- "L6 Runtime Self-Optimization," "L7 Self-Evolving Engineering OS" — not built.
- "Multi-model routing (cost-aware fallback chains)" — not built; `model_profile`
  is display-only.
- "Rollback works" (v1.0 DoD) — no rollback command exists; only `backup.rs`
  file-level backup/restore for `doctor --fix`.
- "Doctor can recover from all documented failure classes" — doctor checks 3 dirs.
- "Circuit breakers" (section 27) — deleted in v0.4.1.

The roadmap should be treated as a **historical aspirational document**, not a
commitment list. **Recommendation: mark it superseded by the freeze + this
audit, or move it to a `docs/archive/` folder.**

### 8e. `AKAR_V1_ARCHITECTURE_FREEZE_PROPOSAL.md` — **accurate and current**

This is the document the code actually follows. Its "next 3 releases" plan
(v0.4/v0.5/v0.6) was executed. Its deletion/simplification candidates were
acted on. Its "must not build before v1 design review" list still holds. This
doc is the de facto source of truth.

### 8f. `docs/foundation/*` — **accurate**

The five playbooks match `foundation.rs`'s six static functions. The
integration claims (safety BLOCKED includes safe alternative, etc.) are true
and tested.

### 8g. `docs/architecture/AKAR_OS.md` — **aspirational framing**

The OS mapping (CPU=Model, Kernel=AKAR, Driver Manager=Skill Intelligence, etc.)
is a metaphor, not a description of running code. It is marked "Adopted in
v0.1.1" and was never revised. Harmless as framing; misleading if read as
implementation status.

---

## 9. Test honesty audit

401 unit tests, 28 evals. Distribution is uneven: `loop_governor.rs` alone has
116 tests (29%); `context_pack.rs` has 3, `doctor.rs` has 2, `model_profile.rs`
has 4.

### 9a. Meaningful behavior tests (test real correctness)

- `config::redact` — 7 tests, genuine secret-redaction coverage. Freeze-listed.
- `safety::classify_command` — risk classification incl. BLOCKED for destructive wipes.
- `event_log` — append/read/rotate/json-escape.
- `bootstrap` — idempotency + no-overwrite guarantee (real invariant).
- `verify` — recipe detection, failure classification, `run_recipe` execution.
- `backup` — backup/restore cycle, find-latest.
- `contract` — classification across task types.
- `skill_registry` — role classification, conflict detection, scan_multi.
- `diff_budget` — readiness ready/blocked, budget comparison, baseline round-trip.
- `loop_governor` — decision priority, exit codes, NEXT_RUN compilation,
  validator PASS/FAIL, telemetry opt-in, repeated-block window. (Well-tested.)
- `hooks` — JSON parsing, hook_decision, exit-2 enforcement.
- `learn` — patch lifecycle (active/resolved), has_active_split_rule_entry.

### 9b. Smoke / non-panic tests (prove it runs, not that it's correct)

- `init::detect_shell_returns_a_value` — asserts it doesn't panic.
- `eval::eval_helper_constructs_correctly` — tests a struct constructor.
- `eval::run_evals_returns_28_results` — counts, doesn't verify correctness.
- `model_profile` tests — mostly format non-empty.

### 9c. Self-fulfilling / tautological evals

- `vague_prompt_contract` — `passed = true` unconditionally (`classify_prompt`
  always returns a contract). Proves nothing.
- `context_pack_build` — `passed = pack.total_files as isize >= 0`. Always true.
- `design_check` — `passed = true` (passes if it didn't panic).
- `doctor_check` — `passed = true` (passes if it didn't panic).
- `request_pressure_compaction` — `passed = "compact" != "stop"`. Tautology.
- `no_all_skills_mode` — passes because a nonexistent dir returns empty.
  Real but trivial.

These six evals inflate the 28/28 PASS count. They are regression smoke checks,
not behavior proofs. The other 22 evals are meaningful.

### 9d. Tests that prove formatting, not behavior

- `format_*` tests across modules assert that output contains a substring
  ("skills:", "recommended:", "overall: PASS"). Useful for output-contract
  stability; not correctness of the underlying logic.

### 9e. Command behavior verified manually but not unit-tested end-to-end

- The **dirty-tree refusal** path (`preflight --snapshot` on dirty tree) is
  structurally tested via `diff_budget::readiness_blocked_when_tree_is_dirty`,
  but the full `cmd_preflight` refusal (exit 1 + message) is not exercised
  end-to-end in a unit test.
- The **hook blocking** behavior is proven by real Claude Code sessions
  (v0.9–v0.11 audit reports) and by `hooks.rs` unit tests on `hook_decision`,
  but there is no integration test that pipes PreToolUse JSON through the
  actual `.ps1`/`.sh` template.
- `akar verify` actually running `cargo` is tested only via `detect_recipe`
  (file inspection); `run_recipe`'s live subprocess execution is not unit-tested
  (sensibly — it would be slow/flaky).

### 9f. Untested runtime paths

- `cmd_request`'s dual-writer interaction (section 7c.3) has no test asserting
  which writer wins. (The v0.19.0 test `request_path_still_writes_next_run`
  asserts the compiled format survives, which indirectly covers it, but does
  not assert the first writer is shadowed.)
- `doctor --fix` end-to-end (issue then fix then backup then resolve) is tested
  in pieces (`safe_fix` tests) but not as a full `cmd_doctor(true)` flow.
- `govern_dependency`/`check_migration` have unit tests but no runtime path
  tests (because there is no runtime path).

### 9g. Net test-honesty verdict

The suite is **genuinely strong where it matters** (redaction, bootstrap
invariants, governor logic, hook decisions, diff measurement). It is **weak
where the code is weak** (doctor, context_pack, design, model_profile) —
consistent with those modules being stubs. The 28/28 eval PASS is **slightly
inflated** by 6 self-fulfilling checks; the honest eval-pass count is ~22/28
meaningful, plus 6 smoke. This is not dishonesty — each self-fulfilling eval is
labeled in code — but the headline "28/28 PASS" reads as stronger than it is.

---

## 10. Dogfood readiness

### Can AKAR be used today on a real external repo?

**Yes, in advisory mode.** The honest answer is: AKAR v0.20.0 can be used as a
read-only discipline layer around a real Claude Code session on an external
repo, and the one piece of real enforcement (the PreToolUse safety hook) works.
It cannot be used as an autonomy runtime, because it does not execute tasks.

### What exact mode is safe?

**Advisory + hook mode:**

1. `akar init` in the external repo (creates `.akar/`, copies templates).
2. `akar status` to confirm HEALTHY and READY.
3. `akar preflight --snapshot "<task>"` to record a baseline (requires clean tree).
4. User manually wires the PreToolUse hook into `~/.claude/settings.json`
   pointing at `templates/hooks/pre-tool-call.ps1` (or `.sh`), with `akar` on
   the subprocess PATH.
5. Run the Claude Code session. The hook blocks destructive commands
   (`safety` BLOCKED → exit 2) and logs every Bash call to
   `.akar/HOOK_EVENTS.jsonl`.
6. `akar postmortem --diff --baseline` to measure the actual diff against budget.
7. `akar learn --list` / `akar learn --resolve` to manage learning patches.

### What command sequence should be used?

```
akar init
akar status                         # confirm READY
akar preflight --snapshot "<task>"  # baseline (clean tree required)
<run Claude Code session with PreToolUse hook active>
akar postmortem --diff --baseline   # measure
akar learn --list
```

### What must not be trusted yet?

- **`akar run "<task>"` / `akar mission "<task>"`** — these print strategy and
  record telemetry but **do not execute the task**. A user who types
  `akar run "fix the bug"` and walks away will find nothing fixed. The output
  says "scaffold mode," but the command name implies action. **Do not trust
  `run`/`mission` to do work.**
- **`akar doctor`** — only checks directory existence. Do not trust it to
  detect a broken hook, a corrupt event log, or a missing template beyond the
  3 dirs. Use `akar hooks --check` for hook-template validity.
- **`akar learn`** — the patch is generic boilerplate, not failure analysis.
  Do not trust it to have diagnosed what went wrong.
- **`akar calibrate`** — display only. `gateway` is always "unknown." Do not
  trust it to reflect real model/gateway state.
- **`akar verify` inside a mission** — the mission Verify state does NOT run
  tests. Only `akar verify` (standalone) runs cargo/npm.
- **Skill conflict enforcement** — `akar skills` reports conflicts but
  disables nothing. Do not trust it to have changed skill state.

### What would be the first dogfood failure?

**The `akar run` expectation gap.** The first time a real user runs
`akar run "fix the login button"` on an external repo, they will expect the
bug to be fixed. It will not be. AKAR will print a strategy report, append a
telemetry event, and exit. The user will think AKAR is broken. This is the
v0.3 audit's #1 failure mode and it is **still the #1 dogfood risk** at v0.20.0,
because the command name `run` has never been changed to reflect that it does
not run anything.

The second likely failure: **the hook fails open if `akar` is not on the
subprocess PATH.** The v0.9.0 changelog documents exactly this. A user who
installs the hook but whose `akar.exe` is not on the PATH Claude Code uses for
hook subprocesses will get no blocking — every command ALLOW'd — with only a
warning in stderr. The hook logs `akar not found`, the governor would surface
`STOP_HOOK_BROKEN`, but only if the user runs `akar status`/`governor` and
reads it.

### What is the smallest safe dogfood trial?

A single external Rust repo, one trivial doc-typo task (1 file, 1-2 LOC), with
the PreToolUse hook active. This is exactly the v0.8.0–v0.11.0 proof shape,
already demonstrated on the AKAR repo itself. The smallest *new* trial is to
repeat it on a repo that is not AKAR — confirming that `bootstrap`'s template
copy, `preflight --snapshot`'s clean-tree check, the hook's PATH resolution,
and `postmortem --diff --baseline`'s measurement all work outside the AKAR
repo. Risk: low. Expected outcome: PASS.

---

## 11. Corrected roadmap to first real try

A roadmap built from **current reality**, not the aspirational master roadmap.

### Must freeze now (do not change before v1 design review)

- `mission.rs` Execute/Verify/MemoryUpdate scaffold behavior (freeze proposal)
- `learn.rs` auto-apply (no automatic patch application)
- `skill_registry.rs` enforcement (no skill disabling)
- `model_profile.rs` persistence (no model profile storage)
- No model API calls, no daemon, no cloud telemetry, no DB
- The advisory-only posture of every command

### Must delete or simplify before v1

- `SafeFix::NormalizePath` — delete dead variant
- `safety::govern_dependency` / `check_migration` — either wire into a real
  command path or move to a clearly-labeled "policy library" module (not
  advertised as a runtime governor)
- `skill_registry::format_registry` / `detect_duplicates` — delete (test-only)
- `BudgetVerdict::as_str` — delete
- Dead `contract` enum variants (A0-A4/A6, Fast/Deep/Autopilot/Emergency,
  Low/High confidence, unused TaskType variants) — delete or gate behind
  `#[allow(dead_code)]` with a "future v1" comment
- `request_intelligence::write_next_run` — delete from `cmd_request` path (it
  is shadowed by `write_governor_next_run`); either remove the call or remove
  the function
- `diff_budget.rs` second budget table — consolidate with `contract.rs`
- `foundation.rs` stale `#[allow(dead_code)]` on live functions — remove
- Stale docs: `INSTALL.md` version string, `OPERATING_MODEL.md` slash-command
  list + pre-commit hook section, `PRODUCT_ROADMAP.md` status

### Must dogfood before new features

- Repeat the v0.8–v0.11 baseline-loop proof on **at least 2 external repos**
  (one Rust, one non-Rust) to confirm `verify` recipe detection and
  `postmortem --diff` generalize.
- Run a real Claude Code session on an external repo with the hook active and
  confirm `STOP_HOOK_BROKEN` / `STOP_REPEATED_BLOCK` governor decisions surface
  correctly when provoked.
- Confirm `akar request` → `akar request --check` round-trip on an external
  repo (NEXT_RUN.md compiles and validates).

### Must not build before v1 design review

- Any code execution in the mission state machine
- Any model API call
- Any automatic settings.json mutation
- Any daemon / background watcher
- Any DB or vector store
- Any GUI
- Any "autopilot" loop that acts without a human/hook in the loop

### First real try criteria

AKAR is ready for a "first real try" (defined as: a non-developer user can
safely install and use it on their own repo without hand-holding) when:

1. The `akar run` expectation gap is closed — either rename the command,
   restructure its output to lead with "THIS IS ADVISORY — NOTHING WILL BE
   EXECUTED," or document the limitation in `akar init` output.
2. `doctor` checks something real beyond directory existence (at minimum: hook
   template presence + EVENT_LOG.jsonl parseability + template-file presence).
3. The hook PATH-failure mode is documented in `akar init` / `akar hooks`
   output so a user knows to verify `akar` is on the subprocess PATH.
4. The dogfood trials above have PASSed on external repos.
5. The stale docs are corrected so a new user is not misled.

### v1.0 definition (corrected from reality)

AKAR v1.0 = **a stable, documented, advisory-only local CLI that reliably
classifies tasks, reports diff budgets, measures actual diffs, records local
telemetry, suggests the next safe loop action, and blocks destructive shell
commands via a Claude Code PreToolUse hook — with honest output that never
implies it executed the task.**

This is a **narrower** v1.0 than the master roadmap's. It drops A5/A6 autonomy,
model routing, circuit breakers, self-optimization, and execution. It matches
what the freeze proposal actually committed to and what the code actually does.

---

## 12. v1.0 boundary check

Checking the freeze proposal's "must not build before v1 design review" list
against v0.20.0:

| Freeze prohibition | v0.20.0 status |
|---|---|
| Model API calls of any kind | ✅ None. Verified: no network, no HTTP, no model client. |
| Automatic skill disabling/enabling | ✅ None. Skills report-only. |
| Automatic memory patch application | ✅ None. Patches advisory; `learn --resolve` only changes status. |
| Daemon or background process | ✅ None. All commands synchronous, exit. |
| Cloud telemetry | ✅ None. All telemetry local in `.akar/`. |
| SQLite / database dependency | ✅ None. JSONL + Markdown only. |
| Vector DB / embeddings | ✅ None. |
| GUI / web UI | ✅ None. |
| Modifying `~/.claude/settings.json` without explicit confirmation | ✅ None. `hooks --install` copies to `.akar/hooks/` only; requires "INSTALL" confirmation. |

**The freeze holds completely.** v0.20.0 has not crossed any v1 boundary. The
only boundary-adjacent behavior is `akar verify` running `cargo`/`npm`, which
is verification execution (allowed) not task execution (forbidden), and is
explicitly user-invoked.

The risk is **not** that v0.20.0 crossed a boundary. The risk is that **future
sessions, reading the master roadmap, may believe they are authorized to build
the execution engine, model routing, or autonomy modes** that the freeze
forbids. The roadmap and the freeze disagree; the freeze must win, and the
roadmap must be marked superseded.

---

## 13. Recommended next release

**v0.21.0 — this audit (audit-only, no code changes).**

**v0.22.0 — Honest Edges.** A small, no-new-features release that closes the
expectation gaps this audit found, without expanding scope:

1. **Lead `akar run` / `akar mission` output with an explicit advisory banner**
   ("ADVISORY ONLY — AKAR did not execute the task. Run the task in Claude
   Code; AKAR records strategy and telemetry."). Do not rename commands (blast
   radius); do not add execution.
2. **Fix `INSTALL.md` version string** (0.10.0 → 0.20.0+) and extend the
   compatibility table.
3. **Fix `OPERATING_MODEL.md`**: remove nonexistent slash commands, replace
   the pre-commit-hook section with the pre-tool-call hook, drop "v0.2.x
   scaffold" tense.
4. **Mark `AKAR_MASTER_ROADMAP_v1.0_REVISED.md` superseded** by the freeze +
   this audit at the top, or move to `docs/archive/`.
5. **Delete the confirmed dead code** (§7a) — `NormalizePath`, dead enum
   variants, `format_registry`/`detect_duplicates`, `BudgetVerdict::as_str`,
   the shadowed `write_next_run` call in `cmd_request`.
6. **Remove stale `#[allow(dead_code)]`** on live `foundation.rs` functions.
7. **Document the hook PATH-failure mode** in `akar init` / `akar hooks` output.
8. **Either wire `govern_dependency`/`check_migration` into a real path or
   relabel them** as a policy library, so they are not mistaken for a runtime
   dependency/migration governor.

No new commands. No execution. No runtime behavior change beyond the advisory
banner text and dead-code removal.

**v0.23.0 — Real Doctor.** Build the Phase 5 doctor honestly: check hook
template presence, EVENT_LOG.jsonl parseability, template-file presence,
`.akar/` writeability. Still read-only. This is the one stub that most
undercuts the "AKAR diagnoses itself" claim.

**v1.0 design review** — only after v0.22 + v0.23 + the external-repo dogfood
trials. The design review decides whether v1.0 stays advisory-only (recommended)
or authorizes a bounded execution path.

---

## 14. Delete / simplify / freeze list

### Delete (confirmed dead)

- `safe_fix::SafeFix::NormalizePath` + its match arm
- `safety::govern_dependency` / `check_migration` (unless wired in v0.22)
- `skill_registry::format_registry`, `detect_duplicates`
- `diff_budget::BudgetVerdict::as_str`
- Dead `contract` enum variants (A0-A4, A6; Fast/Deep/Autopilot/Emergency; Low/High; unused TaskType)
- `skill_registry::SkillSource::ClaudeBundled`, non-Active `SkillStatus`, `SkillRole::LibraryOnly`
- `request_intelligence::write_next_run` call in `cmd_request` (shadowed)

### Simplify

- `diff_budget.rs` — consolidate the second budget table with `contract.rs`
- `doctor.rs` — either build real checks (v0.23) or rename honestly to "dir check"
- `context_pack.rs` — either read contents (v1 feature) or reduce to a path enumerator
- `model_profile.rs` — drop vestigial fields or populate them
- Stale `#[allow(dead_code)]` annotations across `foundation.rs`, `contract.rs`

### Freeze (do not change before v1 design review)

- `mission.rs` scaffold behavior
- `learn.rs` no-auto-apply
- `skill_registry.rs` no-enforcement
- `model_profile.rs` no-persistence
- Advisory-only posture of all commands
- No model API / no daemon / no cloud / no DB
- The freeze proposal's full "must not build" list

### Keep (load-bearing, well-tested)

- `config.rs`, `event_log.rs`, `bootstrap.rs`, `backup.rs`, `safe_fix.rs` (minus NormalizePath)
- `contract.rs` (minus dead variants), `safety.rs` (classify_command/check_secrets)
- `verify.rs`, `diff_budget.rs` (minus second table), `loop_governor.rs`, `hooks.rs`
- `foundation.rs`, `learn.rs` (lifecycle), `postmortem.rs`, `preflight.rs`, `workflow.rs`
- The `.gitignore` policy for `.akar/` runtime artifacts

---

## 15. Honest conclusion

AKAR v0.20.0 is **honest about what it is**. The current README and the v1
architecture freeze proposal describe the same advisory-only CLI, and the code
matches both. The drift and overclaims that justified the v0.3.0 audit were
acted on between v0.4.0 and v0.11.0: dead code was deleted, hooks were made
real, diff measurement was made real, the baseline loop was proven end-to-end,
and the foundation/governor/next-run stack was built on top of that honest
foundation. 401 tests pass, 28 evals pass, the release build is clean, and the
working tree is clean.

The remaining problems are **not lies**. They are:

- a stub doctor that never grew past directory checks,
- two policy functions wired only into evals,
- dead enum variants left over from aspirational design,
- one shadowed writer inside `cmd_request`,
- a second budget table that duplicates the first,
- and a set of stale planning docs (most importantly the master roadmap) that
  describe a different, unbuilt product.

None of these change runtime behavior in a way that endangers a user. The
advisory posture is intact. The freeze holds. No v1 boundary was crossed.

The biggest risk is **not technical**. It is that a future session reads the
master roadmap, believes AKAR is supposed to be an A5/A6 autonomy runtime, and
starts building the execution engine the freeze explicitly forbids — undoing
the discipline that makes v0.20.0 trustworthy. This audit exists to anchor the
**real** definition of AKAR — advisory, local, honest, frozen — so that does
not happen.

**Recommended next release: v0.22.0 Honest Edges** (close the expectation gaps,
delete dead code, fix stale docs, no new features). Then **v0.23.0 Real
Doctor**. Then a v1.0 design review that reaffirms advisory-only as the v1.0
definition.

AKAR v0.20.0 is safe to dogfood in advisory mode today. It is not, and was
never meant to be, the runtime the master roadmap describes.

---

*End of audit. This document is audit-only. No source code was modified. No
runtime behavior was changed. Verification commands and git housekeeping follow
per the release instructions.*
