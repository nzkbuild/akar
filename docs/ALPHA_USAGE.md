# AKAR Stable Advisory Alpha

## What AKAR is

AKAR (Adaptive Knowledge & Action Runtime) is a local CLI advisory tool for Claude Code sessions. It generates evidence-backed guardrails (NEXT_RUN prompt, diff budget, governor decisions) so you can run Claude Code manually with a safety net — baseline, budget, postmortem. AKAR does not execute commands, does not run Claude Code, and does not make decisions for you.

## Stable advisory alpha

AKAR is **stable advisory alpha** for the CLI loop. This means:
- The CLI commands (`init`, `doctor`, `status`, `preflight`, `request`, `postmortem`, etc.) are stable and tested.
- The NEXT_RUN prompt compiler is project-kind-aware and produces valid, verifiable output.
- The governor makes consistent, documented decisions based on local evidence.
- The diff budget and postmortem measurement are reliable.
- The hook templates are installable and checkable with an always-available embedded fallback.

This does **not** mean:
- Hook-integrated stability in live Claude Code sessions is proven end-to-end.
- Autonomous execution is supported (there is no `akar run` execution engine).
- AKAR is v1.0.0 or production-hardened.

## What stable advisory alpha supports

- Local CLI advisory loop (init, doctor, status, preflight, request, postmortem, learn)
- Project-kind-aware NEXT_RUN prompt compilation (Rust, Node, Python, Unknown)
- Doctor/status/governor health and readiness guidance
- Preflight snapshot with diff budget
- Postmortem diff budget measurement against baseline
- Manual Claude Code usage with AKAR-generated guardrails
- Local-only evidence files (`.akar/` directory, telemetry logs)
- Advisory hook template install and check (embedded fallback always available)
- Manual Claude Code PreToolUse hook wiring (AKAR provides templates and instructions, never edits `~/.claude/settings.json`)

## What stable advisory alpha does NOT yet support

- Live hook-integrated stability guarantee (not dogfooded in v0.33)
- Automatic hook registration in Claude Code (always manual, by design)
- One-command autonomous execution (`akar run` is advisory/scaffold, not an execution engine)
- Model routing or API calls (AKAR is local-only, no model integration)
- OpenCode/Codex adapters or multi-agent support
- Token cache or cost optimizer
- Background daemon or cloud telemetry
- Multi-task session proof (single-task dogfood only so far)
- Cross-platform stability guarantee (primary development on Windows)
- Python and Unknown project full dogfood proof (Rust and Node dogfooded)

## Supported stable workflow

### 1. Install

Build from source and add to PATH. See `docs/INSTALL.md`.

### 2. Enter your project

```bash
cd /path/to/your/project
```

### 3. Initialize AKAR

```bash
akar init
```

This creates `.akar/` with template files. Inspect `git status` afterward — AKAR does not decide whether `.akar/` belongs in `.gitignore` or should be committed. That choice is yours.

### 4. Install hook templates (optional)

```bash
akar hooks --install
```

This writes the embedded PreToolUse hook templates to `.akar/hooks/`. AKAR never edits `~/.claude/settings.json`. To wire the hooks into Claude Code, follow the manual instructions from `akar hooks` — copy the settings.json example and point it at `.akar/hooks/pre-tool-call.ps1` (Windows) or `.akar/hooks/pre-tool-call.sh` (POSIX).

### 5. Verify hook templates

```bash
akar hooks --check
```

### 6. Check readiness

```bash
akar doctor
akar status
```

### 7. Take a baseline snapshot

Ensure your working tree is clean (commit or intentionally handle any dirty files first). Then:

```bash
akar preflight --snapshot "<task description>"
```

AKAR refuses a dirty snapshot — this is by design. If `.akar/` itself is making the tree dirty, AKAR prints an advisory explaining the cause and your options (`.gitignore` or commit). AKAR will not auto-ignore, auto-delete, or auto-commit anything.

### 8. Generate the NEXT_RUN prompt

```bash
akar request "your actual task description"
```

### 9. Validate the prompt

```bash
akar request --check
```

### 10. Use the prompt with Claude Code

The file `.akar/NEXT_RUN.md` is the compiled guardrail prompt. Hand it to Claude Code manually. AKAR does not execute this prompt, does not launch Claude Code, and does not monitor the session.

### 11. Do the work manually

Follow the NEXT_RUN rules: stay within the diff budget, obey the allowed/forbidden commands, stop on the listed stop conditions. Run your project tests yourself (`cargo test`, `npm test`, `python -m pytest`, etc.). AKAR does not run project tests for Node or Python projects — `akar verify` automated execution is Rust/Cargo only.

### 12. Run postmortem

After the task is done and tests pass:

```bash
akar postmortem --diff --baseline
```

### 13. Check learning patches

```bash
akar learn --list
```

### 14. Commit intentionally

Commit only the changes you intended. Do not commit `.akar/` generated files unless you've decided to track them. Do not commit secrets or test artifacts.

## Stable guarantees

These guarantees hold for stable advisory alpha and will not be silently removed:

- **Local-only by default.** AKAR reads and writes only within the project's `.akar/` directory and `~/.claude/akar/`. No network calls, no cloud telemetry, no external services.
- **No model API calls.** AKAR does not call any LLM API. It is a deterministic CLI tool.
- **No Claude Code settings mutation.** AKAR never edits `~/.claude/settings.json` or any Claude Code configuration file. Hook wiring is always manual.
- **No source-code edits by AKAR.** AKAR generates `.md` and `.json` files only. It never modifies your project's source code.
- **No destructive git operations.** AKAR never runs `git reset`, `git clean`, `git stash`, `git checkout`, or `git push`. It only runs read-only git commands (`git status`, `git diff`, `git log`) plus `git rev-parse` for HEAD detection.
- **No auto-apply of learning patches.** `akar learn` lists patches; it never applies them automatically.
- **Hook templates are always installable and checkable.** The embedded fallback ensures `akar hooks --install` and `akar hooks --check` work even without the AKAR source tree.
- **`request --check` validates NEXT_RUN structure.** The validator checks section count, minimum content, safety contract presence, and governor decision consistency.
- **Postmortem measures diff budget against baseline.** The diff measurement (`git diff --stat` against the baseline HEAD) is reliable and honest.
- **Doctor reports project kind honestly.** Rust, Node, Python are PASS; Unknown is WARN. The check is named "project kind," not "cargo project."

## Known limitations

These limitations are acknowledged and scoped for future releases:

- **Live Claude Code hook telemetry was not tested in v0.33.** The third external dogfood trial was CLI-only. Hook behavior in a live Claude Code session with wired PreToolUse needs a dedicated dogfood trial (targeted for v0.35.0).
- **Hooks require manual Claude settings wiring.** AKAR provides templates and instructions but never automates the registration step. This is a design choice, not a bug — but it means the hook pipeline is not "one command to activate."
- **Hook behavior depends on user PATH and session setup.** The PreToolUse hook calls `akar safety` via the user's PATH. If `akar` is not on PATH in the hook's execution environment, the hook will fail.
- **Python and Unknown projects need full dogfood trials.** Node (v0.33) and Rust (v0.24) have been dogfooded. Python and Unknown fixtures need the same end-to-end proof (targeted for v0.36.0).
- **Cross-platform behavior needs broader validation.** Primary development and dogfooding is on Windows. macOS and Linux have not been independently dogfooded.
- **Multi-task sessions need dogfood.** All dogfood trials have been single-task (one baseline, one fix, one postmortem). A session with multiple sequential tasks (preflight → fix → postmortem → preflight → fix → postmortem) needs proof (targeted for v0.37.0).
- **`akar verify` automated execution is Rust/Cargo only.** For Node, Python, and Unknown projects, `akar verify` reports "(no automated checks)" and directs the user to manual verification. This is by design — AKAR does not run `npm test` or `pytest`.
- **`akar run` and `akar mission` are advisory scaffolds, not execution engines.** They write prompts and record telemetry but do not drive Claude Code or execute tasks.
- **Cost/token optimization is future work.** No token counting, cache warming, or cost estimation exists today.
- **Multi-agent support is future work.** AKAR is designed for single-agent Claude Code sessions. Multi-agent orchestration patterns are not yet designed or scoped.

## Version roadmap

| Version | Scope |
|---|---|
| v0.34.0 | Stable advisory alpha freeze (this release) |
| v0.35.0 | Live Hook Dogfood Trial |
| v0.36.0 | Python External Dogfood Trial |
| v0.37.0 | Multi-task Session Dogfood Trial |
| v1.0.0 | Release Candidate review |

Multi-agent support, token optimization, OpenCode/Codex adapters, and background services are deferred to post-v1.0.0 design work. The priority is proving the single-agent advisory loop is solid before adding scope.
