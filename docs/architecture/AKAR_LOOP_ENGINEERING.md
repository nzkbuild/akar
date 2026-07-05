# AKAR Loop Engineering

Date: 2026-07-05
Status: ACTIVE — applies to all AKAR development sessions

---

## What loop engineering is

Loop engineering is the practice of developing AKAR through small, scoped,
evidence-driven iterations. Each iteration has a frozen baseline, a single
narrow target, deterministic instructions, required verification, and a
recorded new baseline. The next iteration starts from that new baseline.

This discipline exists because Claude Code implements quickly and confidently.
Without a tight loop, sessions accumulate scope drift, overclaiming, and
untested assumptions. The loop is the human's main control mechanism.

---

## The loop

1. **Freeze current truth.**
   Before writing a prompt, document exactly what AKAR is now — what works,
   what is scaffold-only, what was removed, what tests pass. This is the
   baseline. Never start from memory.

2. **Choose one narrow release target.**
   Each release proves one thing. Not "improve AKAR." One specific claim:
   "hook templates can be installed with explicit confirmation" or
   "destructive commands are blocked." The target determines the scope.

3. **Write direct, deterministic instructions.**
   The prompt must list exact tasks. Not "handle edge cases." Not "improve
   the UX." Exact file names, exact behavior, exact output format. If a task
   can be interpreted two ways, pick one and say it explicitly.

4. **Avoid ambiguous branching instructions.**
   "Add X or Y if appropriate" is not an instruction — it is a delegation
   of architectural judgment. Every branch in the instructions produces a
   branch in the implementation. Keep instructions linear.

5. **Require verification commands.**
   Every prompt must end with exact verification commands and expected outputs.
   "cargo test" is not sufficient alone — state what passing means. Unexpected
   test counts, unexpected output, or unexpected behavior are blockers, not
   details to clean up later.

6. **Audit output against scope.**
   After Claude responds, read the diff. Not the summary — the diff. Summaries
   describe intent. Diffs describe what happened. Check for scope creep, added
   behavior, renamed invariants, or new dependencies.

7. **Record the new baseline.**
   After a passing session, update the baseline: what was added, what was
   removed, what is still scaffold-only, what the test count is, what the
   version is. This baseline becomes the first paragraph of the next prompt.

8. **Generate the next prompt from evidence.**
   The next prompt starts from the recorded baseline, not from memory. The
   target for the next release comes from the audit of the current one —
   what failed, what overclaimed, what is still missing.

---

## Prompt Rules

Rules for writing prompts to Claude for AKAR development:

- **No broad scope.** Never write "continue building AKAR" or "improve AKAR."
  These produce sessions that expand scope, version-bump without evidence, and
  build features that were not requested.

- **No ambiguous choices.** Never write "add X or Y." Never write "handle this
  if appropriate." Every optional item becomes a decision Claude makes without
  audit. Make the decision yourself before writing the prompt.

- **No hidden architecture expansion.** If a task would add a new module, a
  new dependency, a new command, or a new data file, name it explicitly. If
  the task does not require expansion, add a hard rule: "Do not add new
  commands. Do not expand architecture."

- **One milestone per prompt.** Each prompt targets one release. Not one
  session — one release. If a session is too large to fit in one prompt,
  split it into sequential prompts with dependencies made explicit.

- **Exact tasks only.** Each task must be completable with a deterministic
  yes/no check. "Make the UX clearer" is not a task. "Add the line 'advisory
  scaffold mode — AKAR does not execute fixes' to format_workflow_report" is
  a task.

- **Exact verification required.** State the exact commands and the exact
  expected output. If `cargo run -- safety "rm -rf /"` must show BLOCKED,
  say so. If `cargo test` must show a specific count, say so.

- **Final response format required.** Every prompt must specify the exact
  format of Claude's response. Without a format, the response contains
  summaries that obscure what actually happened. Require: files changed,
  what was removed, what was added, verification output, what remains
  scaffold-only.

---

## Release Loop

Rules for versioning AKAR releases:

- **Each v0.x release has one clear purpose.** The CHANGELOG entry must
  state what was proved, not just what was added. "Added hook templates" is
  not sufficient. "Proved Claude Code can call akar safety via a hook" is
  sufficient.

- **Patch versions (v0.x.y) are for cleanup.** Use patch versions for:
  removing dead code, fixing false claims in docs, fixing test failures,
  correcting safety classification gaps. No new behavior, no new commands.

- **Minor versions (v0.x.0) are for feature proof.** Use minor versions
  only when a new capability is proven to work end-to-end, not just
  implemented. "Implemented hook install" is not proof. "Ran akar hooks
  --install on a real project, hooks fired in Claude Code" is proof.

- **Failed verification blocks version bumps.** If `cargo test` fails,
  `cargo build --release` fails, or any required verification command
  produces unexpected output, do not bump the version. Fix first.

- **Scaffold-only items must stay labeled.** Every release must document
  what is still scaffold-only. If a previously-scaffold item becomes real,
  that is the proof that justifies the minor version bump.

---

## Human Audit

The human's role in loop engineering:

- **Claude implements quickly.** A well-scoped prompt produces a working
  implementation in seconds. Speed is not the bottleneck. Correctness and
  scope discipline are.

- **Human audit decides correctness.** Claude's summary says what it intended
  to do. The diff says what it did. Read the diff. Summaries are useful for
  orientation; they are not evidence. A confident summary of incorrect
  behavior is worse than no summary.

- **Evidence beats confident summaries.** "194 tests passing" is evidence.
  "The hooks are now fully integrated" is a claim. Claims require evidence.
  When Claude reports something works, verify it with the exact command
  from the verification section, not from Claude's description of what
  the command would show.

- **Scaffold-only items must remain visible.** Every release response must
  include a "Remains scaffold-only" section. If this section disappears
  from a response, that is a signal that Claude has either completed the
  scaffolding (verify it) or has stopped tracking it (a warning sign).
  Never accept "everything is working" without checking what is still
  scaffold-only.

- **Scope drift is slow and confident.** The most dangerous drift happens
  one small step at a time, each step individually justified. The audit
  must compare the current state to the original phase boundary, not just
  to the previous release.
