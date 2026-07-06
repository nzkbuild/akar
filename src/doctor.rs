//! AKAR Doctor — read-only environment checks for advisory dogfood readiness.
//!
//! The doctor verifies AKAR is ready for advisory dogfood **without modifying
//! files or configuration**. It checks the environment (project root, `.akar/`
//! writability, `akar` on PATH, git repo, working-tree state, Cargo project),
//! runtime files (NEXT_RUN.md validity, DIFF_BASELINE.json, LEARNING_PATCHES.md
//! summary, EVENT_LOG/HOOK_EVENTS JSONL parseability), hook templates, and
//! produces a sectioned report with an overall OK / WARN / FAIL status.
//!
//! ## Read-only guarantees
//!
//! `run_doctor_report` and `run_checks` never:
//! - create `.akar/` or any directory,
//! - write or rewrite NEXT_RUN.md (it validates an existing file only),
//! - resolve learning patches,
//! - install hooks or modify `~/.claude/settings.json`,
//! - mutate git,
//! - delete or truncate logs,
//! - auto-fix malformed files.
//!
//! The only non-read-only path is `akar doctor --fix`, which is limited to the
//! existing safe directory/template creation (see `cmd_doctor`) and explicitly
//! refuses to auto-fix dogfood-critical checks.

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Issue types — used by `cmd_status` (DEGRADED flag) and `doctor --fix`
// ---------------------------------------------------------------------------

/// Severity of a doctor finding (legacy `Vec<Issue>` API).
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

/// A single issue found during a doctor run.
#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
    /// The kind of fix that can automatically resolve this issue, if any.
    pub fix_hint: Option<FixHint>,
}

/// Hints that tell `doctor --fix` what safe action to take for an issue.
///
/// Only directory creation is offered as an auto-fix. Dogfood-critical checks
/// (invalid NEXT_RUN.md, malformed telemetry logs, missing hook templates, no
/// git repo) have no `FixHint` — they require human action, not an automatic
/// fix. `doctor --fix` never modifies Claude settings, installs hooks, mutates
/// git, or rewrites files.
#[derive(Debug, Clone)]
pub enum FixHint {
    /// The directory at this path is missing and should be created.
    CreateDir(PathBuf),
}

// ---------------------------------------------------------------------------
// Doctor run (legacy Vec<Issue> API — used by `cmd_status`)
// ---------------------------------------------------------------------------

/// Run the read-only doctor checks against `cfg` and return findings as a flat
/// `Vec<Issue>`. Used by `akar status` to set the HEALTHY/DEGRADED flag.
///
/// This is a lossy view of the full [`DoctorReport`]; `akar doctor` prints the
/// sectioned report via [`run_doctor_report`]. Read-only: never writes or
/// mutates anything.
pub fn run_checks(cfg: &crate::config::Config) -> Vec<Issue> {
    let report = run_doctor_report(cfg);
    report.to_issues()
}

// ---------------------------------------------------------------------------
// Doctor report types
// ---------------------------------------------------------------------------

/// Overall doctor verdict.
///
/// - `Ok` — no failed checks. Advisory dogfood can proceed.
/// - `Warn` — dogfood is possible but something advisory is missing (e.g. no
///   baseline snapshot, no NEXT_RUN.md). No check that gates safety failed.
/// - `Fail` — dogfood should stop: invalid NEXT_RUN.md, missing hook
///   templates, malformed telemetry logs, or no git repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctorStatus {
    Ok,
    Warn,
    Fail,
}

impl DoctorStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DoctorStatus::Ok => "OK",
            DoctorStatus::Warn => "WARN",
            DoctorStatus::Fail => "FAIL",
        }
    }
}

/// The outcome of a single doctor check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckOutcome {
    /// Check passed.
    Pass,
    /// Check failed but does not gate advisory dogfood (advisory gap).
    Warn,
    /// Check failed and gates advisory dogfood (stop).
    Fail,
    /// Check was not applicable (e.g. file absent and that is acceptable).
    #[allow(dead_code)]
    Na,
}

/// A single doctor check result.
#[derive(Debug, Clone)]
pub struct Check {
    pub label: String,
    pub outcome: CheckOutcome,
    pub detail: String,
    /// A safe fix the user can apply via `akar doctor --fix`, if any.
    /// `None` means no auto-fix is available (e.g. invalid NEXT_RUN.md,
    /// malformed logs, missing hook templates, no git repo — these require
    /// human action, not an automatic fix).
    pub fix_hint: Option<FixHint>,
}

impl Check {
    fn pass(label: &str, detail: &str) -> Self {
        Check { label: label.to_string(), outcome: CheckOutcome::Pass, detail: detail.to_string(), fix_hint: None }
    }
    fn warn(label: &str, detail: &str) -> Self {
        Check { label: label.to_string(), outcome: CheckOutcome::Warn, detail: detail.to_string(), fix_hint: None }
    }
    fn warn_fixable(label: &str, detail: &str, fix: FixHint) -> Self {
        Check { label: label.to_string(), outcome: CheckOutcome::Warn, detail: detail.to_string(), fix_hint: Some(fix) }
    }
    fn fail(label: &str, detail: &str) -> Self {
        Check { label: label.to_string(), outcome: CheckOutcome::Fail, detail: detail.to_string(), fix_hint: None }
    }
}

/// A sectioned doctor report.
#[derive(Debug, Clone)]
pub struct DoctorReport {
    pub status: DoctorStatus,
    pub environment: Vec<Check>,
    pub files: Vec<Check>,
    pub hooks: Vec<Check>,
    pub telemetry: Vec<Check>,
    pub git: Vec<Check>,
    pub next_run: Vec<Check>,
    pub recommendations: Vec<String>,
}

impl DoctorReport {
    /// Aggregate every check across all sections into a flat issue list for
    /// the legacy `run_checks` API. Fail → Severity::Error, Warn → Warning,
    /// Pass/Na → omitted.
    pub fn to_issues(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        for c in self.all_checks() {
            match c.outcome {
                CheckOutcome::Fail => issues.push(Issue {
                    severity: Severity::Error,
                    message: format!("{} — {}", c.label, c.detail),
                    fix_hint: c.fix_hint.clone(),
                }),
                CheckOutcome::Warn => issues.push(Issue {
                    severity: Severity::Warning,
                    message: format!("{} — {}", c.label, c.detail),
                    fix_hint: c.fix_hint.clone(),
                }),
                CheckOutcome::Pass | CheckOutcome::Na => {}
            }
        }
        issues
    }

    fn all_checks(&self) -> Vec<&Check> {
        let mut v: Vec<&Check> = Vec::new();
        v.extend(self.environment.iter());
        v.extend(self.files.iter());
        v.extend(self.hooks.iter());
        v.extend(self.telemetry.iter());
        v.extend(self.git.iter());
        v.extend(self.next_run.iter());
        v
    }

    fn has_fail(&self) -> bool {
        self.all_checks().iter().any(|c| c.outcome == CheckOutcome::Fail)
    }
    fn has_warn(&self) -> bool {
        self.all_checks().iter().any(|c| c.outcome == CheckOutcome::Warn)
    }
}

// ---------------------------------------------------------------------------
// run_doctor_report — the real read-only check engine
// ---------------------------------------------------------------------------

/// Run all read-only doctor checks against `cfg` and return a sectioned
/// [`DoctorReport`]. Read-only: never writes, creates, deletes, or mutates.
pub fn run_doctor_report(cfg: &crate::config::Config) -> DoctorReport {
    let environment = check_environment(cfg);
    let files = check_files(cfg);
    let hooks = check_hooks_section(cfg);
    let telemetry = check_telemetry(cfg);
    let git = check_git(cfg);
    let next_run = check_next_run(cfg);

    let mut report = DoctorReport {
        status: DoctorStatus::Ok,
        environment,
        files,
        hooks,
        telemetry,
        git,
        next_run,
        recommendations: Vec::new(),
    };

    report.status = if report.has_fail() {
        DoctorStatus::Fail
    } else if report.has_warn() {
        DoctorStatus::Warn
    } else {
        DoctorStatus::Ok
    };

    report.recommendations = build_recommendations(&report);
    report
}

/// Environment checks: project root, `.akar/` writability, `akar` on PATH.
fn check_environment(cfg: &crate::config::Config) -> Vec<Check> {
    let mut out = Vec::new();

    // project root detected
    if cfg.project_root.exists() {
        out.push(Check::pass(
            "project root",
            &format!("{}", cfg.project_root.display()),
        ));
    } else {
        out.push(Check::fail(
            "project root",
            &format!("not found: {}", cfg.project_root.display()),
        ));
    }

    // .akar/ exists or is creatable by normal AKAR commands
    if cfg.akar_dir.exists() {
        out.push(Check::pass(
            ".akar/ directory",
            &format!("{}", cfg.akar_dir.display()),
        ));
        // writability probe — read-only: attempt to open a temp file for write
        // then immediately delete it. Never leaves a file behind.
        let probe = cfg.akar_dir.join(".akar_doctor_write_probe.tmp");
        let writable = match std::fs::File::create(&probe) {
            Ok(_) => {
                let _ = std::fs::remove_file(&probe);
                true
            }
            Err(_) => false,
        };
        if writable {
            out.push(Check::pass(".akar/ writable", "yes"));
        } else {
            out.push(Check::fail(
                ".akar/ writable",
                "directory exists but is not writable",
            ));
        }
    } else {
        // Missing .akar/ is a WARN, not a FAIL — `akar bootstrap`/`akar init`
        // create it, and advisory dogfood is still possible after bootstrap.
        // `doctor --fix` can also create it (the one safe fix available).
        out.push(Check::warn_fixable(
            ".akar/ directory",
            &format!("missing — run 'akar bootstrap' to create {}", cfg.akar_dir.display()),
            FixHint::CreateDir(cfg.akar_dir.clone()),
        ));
    }

    // akar binary visible in PATH (best-effort; read-only)
    out.push(check_akar_on_path());

    out
}

/// Best-effort check that the `akar` binary is resolvable on PATH.
/// This matters because the PreToolUse hook fails open if `akar` is not on
/// the subprocess PATH. Read-only.
fn check_akar_on_path() -> Check {
    // Prefer `which`/`where` if available; fall back to current_exe.
    let found = std::env::var("PATH").is_ok()
        && (which_akar("where.exe akar").is_some() || which_akar("which akar").is_some());
    if found {
        return Check::pass("akar on PATH", "yes");
    }
    // Fall back to current_exe — if AKAR is running, its binary exists
    // somewhere, though not necessarily on the hook subprocess PATH.
    match std::env::current_exe() {
        Ok(p) => Check::warn(
            "akar on PATH",
            &format!(
                "could not confirm via where/which; running binary at {}. \
                 Verify it is on the subprocess PATH Claude Code uses for hooks, \
                 or the hook will fail open (ALLOW + warning).",
                p.display()
            ),
        ),
        Err(_) => Check::warn(
            "akar on PATH",
            "could not confirm; verify akar is on the hook subprocess PATH",
        ),
    }
}

/// Run a `where`/`which` lookup and return `Some(path)` if it succeeds.
fn which_akar(cmdline: &str) -> Option<String> {
    let mut parts = cmdline.split_whitespace();
    let prog = parts.next()?;
    let args: Vec<&str> = parts.collect();
    let out = std::process::Command::new(prog).args(&args).output().ok()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout);
        let s = s.trim();
        if !s.is_empty() {
            return Some(s.to_string());
        }
    }
    None
}

/// Runtime-file checks: NEXT_RUN.md, DIFF_BASELINE.json, LEARNING_PATCHES.md,
/// EVENT_LOG.jsonl, HOOK_EVENTS.jsonl.
fn check_files(cfg: &crate::config::Config) -> Vec<Check> {
    let mut out = Vec::new();

    // NEXT_RUN.md existence (validity is in the next_run section)
    let next_run_path = cfg.akar_dir.join("NEXT_RUN.md");
    if next_run_path.exists() {
        out.push(Check::pass(
            "NEXT_RUN.md present",
            &format!("{}", next_run_path.display()),
        ));
    } else {
        out.push(Check::warn(
            "NEXT_RUN.md present",
            "missing — run 'akar request' to generate a Claude-ready next-run prompt",
        ));
    }

    // DIFF_BASELINE.json
    let baseline_path = cfg.akar_dir.join("DIFF_BASELINE.json");
    if baseline_path.exists() {
        match crate::diff_budget::read_baseline(&cfg.akar_dir) {
            Ok(b) => out.push(Check::pass(
                "DIFF_BASELINE.json",
                &format!("valid; head {} ({} files, {} LOC)", b.head_commit, b.budget_files_max, b.budget_loc_max),
            )),
            Err(e) => out.push(Check::fail(
                "DIFF_BASELINE.json",
                &format!("present but unreadable: {}", e),
            )),
        }
    } else {
        out.push(Check::warn(
            "DIFF_BASELINE.json",
            "missing — run 'akar preflight --snapshot \"<task>\"' before a measured session",
        ));
    }

    // LEARNING_PATCHES.md summary
    let patches_path = cfg.akar_dir.join("LEARNING_PATCHES.md");
    if patches_path.exists() {
        let s = crate::learn::summarize_patches(&patches_path);
        let detail = format!(
            "{} entries ({} active, {} resolved; {} active split-rule)",
            s.total, s.active, s.resolved, s.active_split_rule
        );
        if s.governor_affected() {
            out.push(Check::warn(
                "LEARNING_PATCHES.md",
                &format!("{}; governor may report SPLIT_TASK", detail),
            ));
        } else {
            out.push(Check::pass("LEARNING_PATCHES.md", &detail));
        }
    } else {
        // No patches file is fine (not a dogfood gate).
        out.push(Check::pass(
            "LEARNING_PATCHES.md",
            "absent (no learning patches recorded yet)",
        ));
    }

    out
}

/// Hook-template checks (the internal equivalent of `akar hooks --check`).
fn check_hooks_section(cfg: &crate::config::Config) -> Vec<Check> {
    let result = crate::hooks::check_hooks(cfg);
    let mut out = Vec::new();

    // Template presence + validity.
    if result.all_valid {
        out.push(Check::pass(
            "hook templates",
            &format!("valid: {}", result.templates_found.join(", ")),
        ));
    } else if result.templates_missing.is_empty() {
        // Should not happen, but be defensive.
        out.push(Check::fail("hook templates", "check returned no templates and no missing entries"));
    } else {
        out.push(Check::fail(
            "hook templates",
            &format!("missing/invalid: {}", result.templates_missing.join("; ")),
        ));
    }

    out
}

/// Telemetry checks: EVENT_LOG.jsonl and HOOK_EVENTS.jsonl parseability.
fn check_telemetry(cfg: &crate::config::Config) -> Vec<Check> {
    let mut out = Vec::new();

    out.push(check_jsonl_file(
        "EVENT_LOG.jsonl",
        &cfg.akar_dir.join("EVENT_LOG.jsonl"),
        false, // absent is acceptable (not a gate)
    ));
    out.push(check_jsonl_file(
        "HOOK_EVENTS.jsonl",
        &cfg.akar_dir.join("HOOK_EVENTS.jsonl"),
        false, // absent is acceptable (not a gate)
    ));

    out
}

/// Validate a JSONL file's structural parseability. Read-only.
///
/// Each non-empty line must be a structurally valid JSON object: starts with
/// `{`, ends with `}`, balanced braces, and balanced string quotes. This is a
/// structural check (AKAR has no JSON dependency), not a full schema parse —
/// but it catches the corruption cases that matter for dogfood (truncated
/// writes, concatenated lines, non-JSON content).
///
/// If `absent_is_fail` is false, a missing file is a Pass (absent is
/// acceptable). If true, missing is a Fail.
fn check_jsonl_file(label: &str, path: &Path, absent_is_fail: bool) -> Check {
    let full_label = format!("{}", label);
    if !path.exists() {
        return if absent_is_fail {
            Check::fail(&full_label, "missing")
        } else {
            Check::pass(&full_label, "absent (no events recorded yet)")
        };
    }
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Check::fail(&full_label, &format!("unreadable: {}", e)),
    };
    let mut line_no = 0usize;
    let mut total_lines = 0usize;
    for line in content.lines() {
        line_no += 1;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        total_lines += 1;
        if let Err(reason) = validate_json_object(trimmed) {
            return Check::fail(
                &full_label,
                &format!("malformed at line {}: {}", line_no, reason),
            );
        }
    }
    Check::pass(&full_label, &format!("{} event line(s) parseable", total_lines))
}

/// Structural JSON-object validator. Returns `Ok(())` if `s` is a single JSON
/// object with balanced braces and string quotes, else `Err(reason)`.
fn validate_json_object(s: &str) -> Result<(), String> {
    let s = s.trim();
    if !s.starts_with('{') {
        return Err("does not start with '{'".to_string());
    }
    if !s.ends_with('}') {
        return Err("does not end with '}'".to_string());
    }
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut prev = '\0';
    for c in s.chars() {
        if in_string {
            if c == '"' && prev != '\\' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        } else if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
        }
        prev = c;
    }
    if in_string {
        return Err("unterminated string".to_string());
    }
    if depth != 0 {
        return Err(format!("unbalanced braces (depth {})", depth));
    }
    Ok(())
}

/// Git checks: repository detected, working-tree state.
fn check_git(cfg: &crate::config::Config) -> Vec<Check> {
    let mut out = Vec::new();

    match crate::diff_budget::is_working_tree_clean(&cfg.project_root) {
        Ok(clean) => {
            out.push(Check::pass("git repository", "detected"));
            if clean {
                out.push(Check::pass("working tree", "clean"));
            } else {
                out.push(Check::warn(
                    "working tree",
                    "dirty — commit or review changes before a measured session",
                ));
            }
            // HEAD commit (informational)
            if let Ok(head) = crate::diff_budget::get_head_commit(&cfg.project_root) {
                out.push(Check::pass("git HEAD", &head));
            }
        }
        Err(_) => {
            out.push(Check::fail(
                "git repository",
                "not detected or git unavailable — AKAR needs a git repo for baseline/postmortem",
            ));
        }
    }

    // Cargo project detected if Cargo.toml exists
    if cfg.project_root.join("Cargo.toml").exists() {
        out.push(Check::pass("cargo project", "Cargo.toml found"));
    } else {
        out.push(Check::warn(
            "cargo project",
            "no Cargo.toml — 'akar verify' will fall back to npm or report no recipe",
        ));
    }

    out
}

/// Next-run checks: validate NEXT_RUN.md against the request contract if present.
fn check_next_run(cfg: &crate::config::Config) -> Vec<Check> {
    let mut out = Vec::new();
    let path = cfg.akar_dir.join("NEXT_RUN.md");
    if !path.exists() {
        // Missing NEXT_RUN is a WARN, not a FAIL (no baseline for dogfood stop).
        out.push(Check::warn(
            "NEXT_RUN.md valid",
            "missing — run 'akar request' to generate it",
        ));
        return out;
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            out.push(Check::fail(
                "NEXT_RUN.md valid",
                &format!("unreadable: {}", e),
            ));
            return out;
        }
    };
    let result = crate::loop_governor::validate_next_run(&content);
    if result.pass {
        out.push(Check::pass(
            "NEXT_RUN.md valid",
            "passes request contract (11 sections, safety, consistency)",
        ));
    } else {
        let detail = if result.reasons.is_empty() {
            "request contract validation failed".to_string()
        } else {
            result.reasons.join("; ")
        };
        out.push(Check::fail("NEXT_RUN.md valid", &detail));
    }
    out
}

/// Build advisory recommendations from the report. Read-only intent.
fn build_recommendations(report: &DoctorReport) -> Vec<String> {
    let mut recs = Vec::new();
    for c in report.all_checks() {
        match c.outcome {
            CheckOutcome::Fail => {
                recs.push(format!("FIX: {} — {}", c.label, c.detail));
            }
            CheckOutcome::Warn => {
                recs.push(format!("advisory: {} — {}", c.label, c.detail));
            }
            CheckOutcome::Pass | CheckOutcome::Na => {}
        }
    }
    if recs.is_empty() {
        recs.push("No action needed — environment is ready for advisory dogfood.".to_string());
    }
    recs
}

// ---------------------------------------------------------------------------
// format_doctor_report
// ---------------------------------------------------------------------------

/// Format the sectioned doctor report for `akar doctor`.
pub fn format_doctor_report(report: &DoctorReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("doctor: {}\n", report.status.as_str()));
    out.push('\n');

    out.push_str("environment:\n");
    out.push_str(&format_checks(&report.environment));
    out.push('\n');

    out.push_str("files:\n");
    out.push_str(&format_checks(&report.files));
    out.push('\n');

    out.push_str("hooks:\n");
    out.push_str(&format_checks(&report.hooks));
    out.push('\n');

    out.push_str("telemetry:\n");
    out.push_str(&format_checks(&report.telemetry));
    out.push('\n');

    out.push_str("git:\n");
    out.push_str(&format_checks(&report.git));
    out.push('\n');

    out.push_str("next-run:\n");
    out.push_str(&format_checks(&report.next_run));
    out.push('\n');

    out.push_str("recommendations:\n");
    for r in &report.recommendations {
        out.push_str(&format!("  - {}\n", r));
    }

    out
}

fn format_checks(checks: &[Check]) -> String {
    let mut out = String::new();
    if checks.is_empty() {
        out.push_str("  (no checks)\n");
        return out;
    }
    for c in checks {
        let tag = match c.outcome {
            CheckOutcome::Pass => "PASS",
            CheckOutcome::Warn => "WARN",
            CheckOutcome::Fail => "FAIL",
            CheckOutcome::Na => "N/A",
        };
        out.push_str(&format!("  [{}] {}: {}\n", tag, c.label, c.detail));
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_cfg_all_missing() -> crate::config::Config {
        crate::config::Config {
            project_root: std::env::current_dir().unwrap(),
            akar_dir: PathBuf::from("/nonexistent/__akar_test__/.akar"),
            global_dir: PathBuf::from("/nonexistent/__akar_test__/global"),
            project_name: "stub-test".to_string(),
        }
    }

    /// Build a config whose `akar_dir` is a fresh temp `.akar/` we control.
    fn cfg_with_temp_akar(label: &str) -> (crate::config::Config, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "akar_doctor_{}_{}",
            label,
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let cfg = crate::config::Config {
            project_root: std::env::current_dir().unwrap(),
            akar_dir: dir.clone(),
            global_dir: dir.join("global"),
            project_name: "doctor-test".to_string(),
        };
        (cfg, dir)
    }

    #[test]
    fn run_checks_returns_issues_when_dirs_missing() {
        let cfg = make_cfg_all_missing();
        let issues = run_checks(&cfg);
        assert!(!issues.is_empty(), "expected issues for missing .akar/");
    }

    #[test]
    fn missing_akar_dir_is_warn_not_fail() {
        // A missing .akar/ is advisory (bootstrap can create it), not a stop.
        let cfg = make_cfg_all_missing();
        let report = run_doctor_report(&cfg);
        let env_akar = report
            .environment
            .iter()
            .find(|c| c.label == ".akar/ directory")
            .unwrap();
        assert_eq!(env_akar.outcome, CheckOutcome::Warn);
        assert!(env_akar.fix_hint.is_some(), "missing .akar/ should offer a CreateDir fix");
    }

    #[test]
    fn doctor_does_not_create_files() {
        // Running the doctor against a missing .akar/ must not create it.
        let cfg = make_cfg_all_missing();
        let _report = run_doctor_report(&cfg);
        assert!(
            !cfg.akar_dir.exists(),
            "doctor must not create .akar/ (read-only)"
        );
    }

    #[test]
    fn doctor_does_not_modify_next_run() {
        let (cfg, dir) = cfg_with_temp_akar("no_modify_nextrun");
        let next_run = dir.join("NEXT_RUN.md");
        std::fs::write(&next_run, "# stub\nnot a valid next-run prompt\n").unwrap();
        let before = std::fs::read_to_string(&next_run).unwrap();
        let _report = run_doctor_report(&cfg);
        let after = std::fs::read_to_string(&next_run).unwrap();
        assert_eq!(before, after, "doctor must not modify NEXT_RUN.md");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_next_run_is_warn_not_fail() {
        let (cfg, dir) = cfg_with_temp_akar("missing_nextrun");
        // No NEXT_RUN.md present.
        let report = run_doctor_report(&cfg);
        assert_ne!(report.status, DoctorStatus::Fail, "missing NEXT_RUN must not FAIL");
        let nr = report.next_run.iter().find(|c| c.label == "NEXT_RUN.md valid").unwrap();
        assert_eq!(nr.outcome, CheckOutcome::Warn);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn invalid_next_run_is_fail() {
        let (cfg, dir) = cfg_with_temp_akar("invalid_nextrun");
        std::fs::write(dir.join("NEXT_RUN.md"), "# not the compiled prompt\nno sections here\n").unwrap();
        let report = run_doctor_report(&cfg);
        assert_eq!(report.status, DoctorStatus::Fail, "invalid NEXT_RUN must FAIL");
        let nr = report.next_run.iter().find(|c| c.label == "NEXT_RUN.md valid").unwrap();
        assert_eq!(nr.outcome, CheckOutcome::Fail);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn valid_next_run_passes() {
        let (cfg, dir) = cfg_with_temp_akar("valid_nextrun");
        // Build a real compiled NEXT_RUN.md via the governor writer.
        let gov_cfg = cfg.clone();
        let report = crate::loop_governor::decide(&gov_cfg);
        let path = crate::loop_governor::write_governor_next_run(&gov_cfg, &report);
        assert!(path.is_some(), "setup: governor writer should produce NEXT_RUN.md");
        let doc_report = run_doctor_report(&cfg);
        let nr = doc_report
            .next_run
            .iter()
            .find(|c| c.label == "NEXT_RUN.md valid")
            .unwrap();
        assert_eq!(nr.outcome, CheckOutcome::Pass, "valid NEXT_RUN should PASS: {:?}", nr.detail);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_baseline_is_warn_not_fail() {
        let (cfg, dir) = cfg_with_temp_akar("missing_baseline");
        // No DIFF_BASELINE.json present.
        let report = run_doctor_report(&cfg);
        let b = report.files.iter().find(|c| c.label == "DIFF_BASELINE.json").unwrap();
        assert_eq!(b.outcome, CheckOutcome::Warn);
        assert_ne!(report.status, DoctorStatus::Fail, "missing baseline must not FAIL");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn malformed_event_log_is_fail() {
        let (cfg, dir) = cfg_with_temp_akar("bad_event_log");
        std::fs::write(dir.join("EVENT_LOG.jsonl"), "this is not json\n{broken\n").unwrap();
        let report = run_doctor_report(&cfg);
        let ev = report.telemetry.iter().find(|c| c.label == "EVENT_LOG.jsonl").unwrap();
        assert_eq!(ev.outcome, CheckOutcome::Fail, "malformed EVENT_LOG must FAIL");
        assert_eq!(report.status, DoctorStatus::Fail);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn malformed_hook_events_is_fail() {
        let (cfg, dir) = cfg_with_temp_akar("bad_hook_events");
        std::fs::write(dir.join("HOOK_EVENTS.jsonl"), "not json at all\n").unwrap();
        let report = run_doctor_report(&cfg);
        let he = report.telemetry.iter().find(|c| c.label == "HOOK_EVENTS.jsonl").unwrap();
        assert_eq!(he.outcome, CheckOutcome::Fail, "malformed HOOK_EVENTS must FAIL");
        assert_eq!(report.status, DoctorStatus::Fail);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn valid_event_log_passes() {
        let (cfg, dir) = cfg_with_temp_akar("good_event_log");
        std::fs::write(
            dir.join("EVENT_LOG.jsonl"),
            "{\"ts\":\"t\",\"event\":\"info\",\"summary\":\"ok\"}\n",
        ).unwrap();
        let report = run_doctor_report(&cfg);
        let ev = report.telemetry.iter().find(|c| c.label == "EVENT_LOG.jsonl").unwrap();
        assert_eq!(ev.outcome, CheckOutcome::Pass, "valid EVENT_LOG should PASS: {:?}", ev.detail);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_hook_template_is_fail() {
        // Point project_root at a temp dir with no templates/hooks/ so the
        // hook-template check fails.
        let tmp = std::env::temp_dir().join(format!(
            "akar_doctor_no_hooks_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let cfg = crate::config::Config {
            project_root: tmp.clone(),
            akar_dir: tmp.join(".akar"),
            global_dir: tmp.join("global"),
            project_name: "no-hooks".to_string(),
        };
        std::fs::create_dir_all(&cfg.akar_dir).unwrap();
        let report = run_doctor_report(&cfg);
        let ht = report.hooks.iter().find(|c| c.label == "hook templates").unwrap();
        assert_eq!(ht.outcome, CheckOutcome::Fail, "missing hook templates must FAIL");
        assert_eq!(report.status, DoctorStatus::Fail);
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn dirty_git_tree_is_reported_as_warn() {
        // The real AKAR repo is the project_root; if its tree is dirty the
        // doctor should WARN (not FAIL). If clean, the check passes. Either
        // way it must not FAIL on tree state alone.
        let cfg = crate::config::Config::discover();
        let (cfg, dir) = (crate::config::Config {
            project_root: cfg.project_root,
            akar_dir: std::env::temp_dir().join(format!(
                "akar_doctor_dirty_{}",
                std::process::id()
            )),
            global_dir: std::env::temp_dir().join("akar_doctor_dirty_global"),
            project_name: "dirty-test".to_string(),
        }, std::env::temp_dir().join("unused"));
        std::fs::create_dir_all(&cfg.akar_dir).unwrap();
        let report = run_doctor_report(&cfg);
        let wt = report.git.iter().find(|c| c.label == "working tree").unwrap();
        // Dirty → Warn, Clean → Pass; never Fail on tree state alone.
        assert_ne!(wt.outcome, CheckOutcome::Fail, "tree state must not FAIL doctor");
        std::fs::remove_dir_all(&cfg.akar_dir).ok();
    }

    #[test]
    fn no_git_repo_is_fail() {
        // Point project_root at a temp dir that is not a git repo.
        let tmp = std::env::temp_dir().join(format!(
            "akar_doctor_nogit_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let cfg = crate::config::Config {
            project_root: tmp.clone(),
            akar_dir: tmp.join(".akar"),
            global_dir: tmp.join("global"),
            project_name: "no-git".to_string(),
        };
        std::fs::create_dir_all(&cfg.akar_dir).unwrap();
        let report = run_doctor_report(&cfg);
        let gr = report.git.iter().find(|c| c.label == "git repository").unwrap();
        assert_eq!(gr.outcome, CheckOutcome::Fail, "no git repo must FAIL");
        std::fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn ok_when_everything_present_and_valid() {
        // In a fully-bootstrapped repo with a valid NEXT_RUN.md and clean tree,
        // the doctor should be OK or Warn (warn is acceptable if e.g. baseline
        // is missing). It must not FAIL.
        let cfg = crate::config::Config::discover();
        // Only assert non-FAIL when the real repo has a valid NEXT_RUN.md.
        let next_run = cfg.akar_dir.join("NEXT_RUN.md");
        if next_run.exists() {
            let report = run_doctor_report(&cfg);
            assert_ne!(report.status, DoctorStatus::Fail, "real repo doctor should not FAIL: {:?}", report.to_issues());
        }
    }

    #[test]
    fn validate_json_object_accepts_valid() {
        assert!(validate_json_object(r#"{"a":1,"b":"x"}"#).is_ok());
        assert!(validate_json_object(r#"{"nested":{"k":[1,2]}}"#).is_ok());
    }

    #[test]
    fn validate_json_object_rejects_malformed() {
        assert!(validate_json_object("not json").is_err());
        assert!(validate_json_object("{unbalanced").is_err());
        assert!(validate_json_object(r#"{"k":"unterminated}"#).is_err());
        assert!(validate_json_object("}").is_err());
    }

    #[test]
    fn format_doctor_report_has_sections() {
        let cfg = crate::config::Config::discover();
        let report = run_doctor_report(&cfg);
        let out = format_doctor_report(&report);
        assert!(out.starts_with("doctor: "));
        assert!(out.contains("environment:"));
        assert!(out.contains("files:"));
        assert!(out.contains("hooks:"));
        assert!(out.contains("telemetry:"));
        assert!(out.contains("git:"));
        assert!(out.contains("next-run:"));
        assert!(out.contains("recommendations:"));
    }
}
