# AKAR v0.25.0 — Hook Template Discovery Report

**Date:** 2026-07-06
**Scope:** make the PreToolUse hook templates installable from the installed AKAR binary so `akar hooks --install`, `akar hooks --check`, and `akar doctor` work in a fresh external repo without the AKAR source tree. Also fix the `bootstrap` output honesty issue from the v0.24 dogfood trial. No new product features; no mission execution; no Claude settings mutation; no changes to governor rules/exit codes/telemetry, NEXT_RUN compiler/validator, safety classification, or hook behavior.
**Method:** embedded the two hook templates in the binary via `include_str!`; rewired `hooks --install` to write the embedded templates; rewired `hooks --check` and the doctor to discover templates from source-tree → exe-dir → project `.akar/hooks/` → embedded fallback; fixed `bootstrap` to report directory creation separately from template-file copy; rebuilt and ran the full verification matrix plus a fresh-external-repo check.

---

## 1. Baseline

Confirmed in Phase 0 (no files modified before verification):

| Check | Result |
|---|---|
| `git log --oneline -5` | HEAD = `a3c2af8 docs: record AKAR first external dogfood trial` |
| `git status` | working tree clean; 19 commits ahead of origin/master (unpushed) |
| `cargo run -- --version` | `akar 0.24.0` |
| `cargo test` | **411 passed; 0 failed; 0 ignored** |
| `cargo run -- doctor` | `doctor: WARN` (pre-existing split-rule entry) |
| `cargo run -- hooks --check` | `status: PASS` (AKAR repo has source-tree templates) |
| `cargo run -- eval` | **28/28 PASS** |

Baseline is v0.24.0, healthy, clean tree.

---

## 2. Dogfood blocker addressed

The v0.24 first external dogfood trial found one blocker: `akar hooks --check` and the doctor's hook-template check always **FAIL** on a fresh external repo because the hook templates live in the AKAR source tree (`templates/hooks/pre-tool-call.{sh,ps1}`), not alongside the installed binary. The `hooks --install` FAIL guidance pointed at a command that could not succeed without the source tree. This made `akar doctor` and `akar status` (DEGRADED) unusable on the exact use case the Real Doctor was built for.

v0.25.0 fixes this by **embedding the templates in the binary** and making them the install source and the check fallback. A fresh external repo now PASSES `hooks --check` and the doctor via the embedded fallback, and `hooks --install` writes real, valid templates into `.akar/hooks/` from the binary.

---

## 3. Embedded template behavior

Two `include_str!` constants bake the template contents into the binary at compile time:

- `EMBEDDED_HOOK_SH` ← `templates/hooks/pre-tool-call.sh`
- `EMBEDDED_HOOK_PS1` ← `templates/hooks/pre-tool-call.ps1`

The source-tree files remain the editable source; the embedded copies are regenerated on every build, so they cannot drift from the source tree. `embedded_template_content(name)` exposes them by name. Both embedded templates are validated by tests (`embedded_bash_template_is_nonempty`, `embedded_powershell_template_is_nonempty`) to be non-empty and to contain `akar safety`, `HOOK_EVENTS.jsonl`, and `exit 2`.

A `HookTemplateSource` enum (`SourceTree`, `ExeDir`, `ProjectInstalled`, `Embedded`) tags where each template came from, surfaced in `hooks --check` and `doctor` output.

---

## 4. hooks --install behavior

`akar hooks --install` now writes the **embedded** templates (not the discovered source-tree ones), so installation works in a fresh external repo with no source tree. Behavior:

- Ensures `.akar/hooks/` exists.
- For each expected template (`pre-tool-call.sh`, `pre-tool-call.ps1`):
  - dest missing → write it (`copied`)
  - dest exists and content is identical to the embedded template → skip (`unchanged`)
  - dest exists and content differs → back up the existing file, then overwrite (`copied` + `backed_up`)
- Prints `copied` / `unchanged` / `backed_up` lists, the paths written, and a clear manual wiring instruction for Claude Code PreToolUse.
- **Does not modify `~/.claude/settings.json`.** The output explicitly states this and shows the settings.json example for the user to apply manually.
- Requires the existing "INSTALL" confirmation (no new flag added).

Tests: `install_writes_embedded_templates_without_source_tree`, `install_skips_when_content_identical`, `install_backs_up_when_content_differs`, `install_cancelled_when_not_confirmed`, `install_copies_when_confirmed`, `install_creates_backup_before_overwrite`. The obsolete `install_fails_when_no_templates_found` test was removed (install always has embedded templates now).

---

## 5. hooks --check behavior

`akar hooks --check` discovers templates in priority order via `discover_hook_templates`:

1. **source-tree** — `<project_root>/templates/hooks/` (dev mode)
2. **exe-dir** — `<exe_dir>/templates/hooks/`
3. **project-installed** — `<project_root>/.akar/hooks/` (after `hooks --install`)
4. **embedded fallback** — the `include_str!` copies (always available)

The result reports `source` (which source was used) and PASS/FAIL. In a fresh external repo with no source-tree templates, `--check` PASSES via the embedded fallback and reports `source: embedded`. After `hooks --install`, it reports `source: project .akar/hooks`. In the AKAR repo, `source: source-tree`.

Output now includes a `source:` line. Test `check_passes_via_embedded_when_no_source_templates` locks the fresh-external-repo PASS.

---

## 6. Doctor behavior

The doctor's `check_hooks_section` calls the updated `hooks::check_hooks`, so it automatically accepts source-tree, project-installed, or embedded templates. It now reports:

- `[PASS] hook templates: valid (source: <source>): pre-tool-call.sh, pre-tool-call.ps1`
- `[PASS] Claude settings wiring: manual — AKAR does not edit ~/.claude/settings.json; run \`akar hooks\` for instructions`

The doctor remains **read-only**: it never installs templates, never creates `.akar/hooks/`, never modifies settings. Tests: `doctor_passes_via_embedded_when_no_source_templates` (fresh external repo PASSes via embedded), `doctor_accepts_installed_project_templates` (PASSes after install), `doctor_does_not_install_templates` (read-only guarantee). The obsolete `missing_hook_template_is_fail` test was removed — hook templates can no longer be "missing" since embedded is always available.

The AKAR-on-PATH warning remains in the environment section.

---

## 7. Bootstrap output fix

The v0.24 dogfood trial found `bootstrap: 0 created, 0 skipped` misleading — bootstrap created `.akar/` but the message counted only template files copied. v0.25 fixes this:

- `BootstrapResult` gained `akar_dir_created` and `global_dir_created` flags.
- `run_bootstrap` sets them by checking existence before `create_dir_all`.
- `format_bootstrap_report` header now reads, e.g., `bootstrap: .akar/ created, 0 template file(s) created, 0 skipped` (or `.akar/ already present, ...`). Directory creation is reported separately from template-file copy.

Tests: `format_bootstrap_report_distinguishes_dir_creation_from_template_copy`, `format_bootstrap_report_says_already_present_when_dir_existed`, plus the updated `format_bootstrap_report_shows_correct_counts`. Bootstrap still does not modify Claude settings.

---

## 8. Tests added/updated

411 → 420 tests (+9 net). Added:

- `embedded_bash_template_is_nonempty`, `embedded_powershell_template_is_nonempty`, `embedded_template_accessor_returns_correct_content`
- `check_passes_via_embedded_when_no_source_templates`
- `install_writes_embedded_templates_without_source_tree`, `install_skips_when_content_identical`, `install_backs_up_when_content_differs`
- `doctor_passes_via_embedded_when_no_source_templates`, `doctor_accepts_installed_project_templates`, `doctor_does_not_install_templates`
- `format_bootstrap_report_distinguishes_dir_creation_from_template_copy`, `format_bootstrap_report_says_already_present_when_dir_existed`

Removed (obsolete premises): `check_fails_when_templates_missing`, `install_fails_when_no_templates_found`, `missing_hook_template_is_fail`.

Updated: `format_hooks_help_contains_key_info`, `format_bootstrap_report_shows_correct_counts`, `format_bootstrap_report_shows_warnings_when_present`, and the `main.rs` `hooks_check_fail_output_includes_hook_broken_guidance` test (added `source: None`).

---

## 9. External-repo setup implications

A non-developer user can now set up AKAR on their own repo without the AKAR source tree:

1. `akar init` (bootstrap `.akar/`, with honest output)
2. `akar hooks --install` (writes embedded templates to `.akar/hooks/`)
3. manually wire `~/.claude/settings.json` (AKAR prints the example; never edits it)
4. `akar hooks --check` → PASS
5. `akar doctor` → OK/WARN (no longer FAIL on missing source-tree templates)

The v0.24 dogfood blocker is resolved. The remaining v0.24 friction findings (generic NEXT_RUN objective, `Cargo.lock` clean-tree friction) are documented for v0.26 and unchanged here.

---

## 10. Verification

| Command | Result |
|---|---|
| `cargo build --release` | clean, **zero warnings** |
| `cargo test` | **420 passed; 0 failed; 0 ignored** |
| `cargo run -- --version` | `akar 0.25.0` (after bump) |
| `cargo run -- hooks --check` | `source: source-tree`, `status: PASS` |
| `cargo run -- doctor` | `doctor: WARN` (pre-existing split-rule; hooks section PASS with source) |
| `cargo run -- status` | `status: HEALTHY` |
| `cargo run -- request` / `request --check` | writes NEXT_RUN.md / `NEXT_RUN check: PASS` |
| `cargo run -- governor --json --no-exit-code` | valid JSON, exit 0 |
| `cargo run -- eval` | **28/28 PASS** |

**External-repo verification** (temp repo outside AKAR, using the release binary):

- `akar hooks --install` → wrote both embedded templates to `.akar/hooks/`, did not modify `~/.claude/settings.json`
- `akar hooks --check` → `source: embedded` (before install) / `source: project .akar/hooks` (after install), `status: PASS`
- `akar doctor` → hook-template check PASS via embedded fallback (no longer FAILs because source-tree templates are absent)

Release build is zero-warning. Test count is 420 (+9). Eval count is 28/28.

---

## 11. Honest conclusion

AKAR v0.25.0 fixes the v0.24 dogfood blocker: hook templates are now installable and checkable from the installed binary, so `akar hooks --install`, `akar hooks --check`, and `akar doctor` work in a fresh external repo without the AKAR source tree. The embedded templates are regenerated from the source-tree files on every build, so they cannot drift. The `bootstrap` output is now honest about what it created. The doctor and `hooks --check` report which template source they used.

What v0.25.0 did **not** do: no new commands, no execution, no auto-run, no auto-apply, no Claude settings mutation, no auto-install of hooks into Claude, no changes to governor rules/exit codes/telemetry, no changes to NEXT_RUN compiler/validator, no changes to safety classification or hook *behavior* (the templates' content is unchanged; only how they are discovered and installed changed). The v1 architecture freeze holds completely.

The setup path that failed in v0.24 now works. The discipline path was already working. AKAR is one step closer to a non-developer "first real try."

---

## 12. Next recommended release

**v0.26.0 — NEXT_RUN task threading.** Accept the user's task prompt in `request` and thread it into the compiled objective/suggested-prompt so NEXT_RUN is self-describing (the v0.24 dogfood's "generic objective" finding). Pair with tolerating/handling `Cargo.lock` in `preflight --snapshot` (the v0.24 clean-tree friction).

**v0.27.0 — Honest Enums.** Scoped deletion of the dead `contract`/`skill_registry` enum variants and `model_profile` vestigial fields deferred since v0.22.

**v1.0 design review** — after v0.26 + a second external-repo dogfood trial (ideally non-Rust) confirms the setup path is friction-free and the hook can be activated end-to-end on an external repo.

*End of report. v0.25.0 Hook Template Discovery. No new features. No execution. Advisory-only, frozen, honest.*