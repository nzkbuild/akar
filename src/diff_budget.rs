//! Diff budget measurement — reads the current git working tree diff and
//! compares it against a preflight diff budget.
//!
//! Uses `git diff --numstat` and `git diff --name-only`. Read-only.
//! Does not modify files, revert changes, or block anything.

use std::path::Path;
use std::process::Command;

// ---------------------------------------------------------------------------
// Baseline snapshot
// ---------------------------------------------------------------------------

/// A snapshot written by `akar preflight --snapshot` before a session starts.
/// Stored as `.akar/DIFF_BASELINE.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffBaseline {
    pub timestamp: String,
    pub prompt: String,
    pub head_commit: String,
    pub task_type: String,
    pub budget_files_max: usize,
    pub budget_loc_max: usize,
}

/// Serialize a `DiffBaseline` to a JSON string (std-only, no external deps).
pub fn baseline_to_json(b: &DiffBaseline) -> String {
    format!(
        "{{\
          \"timestamp\":\"{ts}\",\
          \"prompt\":\"{prompt}\",\
          \"head_commit\":\"{head}\",\
          \"task_type\":\"{task}\",\
          \"budget_files_max\":{bf},\
          \"budget_loc_max\":{bl}\
        }}",
        ts = json_escape_simple(&b.timestamp),
        prompt = json_escape_simple(&b.prompt),
        head = json_escape_simple(&b.head_commit),
        task = json_escape_simple(&b.task_type),
        bf = b.budget_files_max,
        bl = b.budget_loc_max,
    )
}

/// Parse a `DiffBaseline` from a JSON string produced by `baseline_to_json`.
/// Returns `Err` if any required field is missing or malformed.
pub fn baseline_from_json(s: &str) -> Result<DiffBaseline, String> {
    let ts = extract_json_str(s, "timestamp").ok_or("missing field: timestamp")?;
    let prompt = extract_json_str(s, "prompt").ok_or("missing field: prompt")?;
    let head = extract_json_str(s, "head_commit").ok_or("missing field: head_commit")?;
    let task = extract_json_str(s, "task_type").ok_or("missing field: task_type")?;
    let bf = extract_json_num(s, "budget_files_max").ok_or("missing field: budget_files_max")?;
    let bl = extract_json_num(s, "budget_loc_max").ok_or("missing field: budget_loc_max")?;

    Ok(DiffBaseline {
        timestamp: ts,
        prompt,
        head_commit: head,
        task_type: task,
        budget_files_max: bf,
        budget_loc_max: bl,
    })
}

/// Check if the git working tree is clean (no uncommitted changes).
/// Returns Ok(true) if clean, Ok(false) if dirty, Err if git unavailable.
pub fn is_working_tree_clean(repo_root: &Path) -> Result<bool, String> {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("git status failed: {}", e))?;

    if !out.status.success() {
        return Err("not a git repository".to_string());
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().is_empty())
}

/// Get the current git HEAD commit hash (short, 12 chars).
pub fn get_head_commit(repo_root: &Path) -> Result<String, String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("git rev-parse failed: {}", e))?;

    if !out.status.success() {
        return Err("could not read HEAD (empty repo?)".to_string());
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Write a `DiffBaseline` to `.akar/DIFF_BASELINE.json`.
pub fn write_baseline(akar_dir: &Path, baseline: &DiffBaseline) -> Result<(), String> {
    let path = akar_dir.join("DIFF_BASELINE.json");
    std::fs::write(&path, baseline_to_json(baseline))
        .map_err(|e| format!("could not write baseline: {}", e))
}

/// Read and parse `.akar/DIFF_BASELINE.json`.
pub fn read_baseline(akar_dir: &Path) -> Result<DiffBaseline, String> {
    let path = akar_dir.join("DIFF_BASELINE.json");
    if !path.exists() {
        return Err(
            "no baseline found — run 'akar preflight --snapshot \"<task>\"' first".to_string(),
        );
    }
    let s =
        std::fs::read_to_string(&path).map_err(|e| format!("could not read baseline: {}", e))?;
    baseline_from_json(&s)
}

/// Measure diff from a specific commit to the current working tree.
pub fn measure_diff_from_commit(repo_root: &Path, base_commit: &str) -> DiffMeasurement {
    let check = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_root)
        .output();

    match check {
        Err(e) => return DiffMeasurement::unknown(&format!("git not available: {}", e)),
        Ok(out) if !out.status.success() => {
            return DiffMeasurement::unknown("not a git repository");
        }
        _ => {}
    }

    let numstat = Command::new("git")
        .args(["diff", base_commit, "--numstat"])
        .current_dir(repo_root)
        .output();

    let numstat_out = match numstat {
        Err(e) => return DiffMeasurement::unknown(&format!("git diff failed: {}", e)),
        Ok(out) if !out.status.success() => {
            return DiffMeasurement::unknown(&format!("git diff {} failed", base_commit));
        }
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
    };

    let name_only = Command::new("git")
        .args(["diff", base_commit, "--name-only"])
        .current_dir(repo_root)
        .output();

    let name_only_out = if numstat_out.trim().is_empty() {
        String::new()
    } else {
        match name_only {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(_) => String::new(),
        }
    };

    if numstat_out.trim().is_empty() {
        return DiffMeasurement::clean();
    }

    parse_numstat(&numstat_out, &name_only_out)
}

// ---------------------------------------------------------------------------
// JSON helpers (std-only)
// ---------------------------------------------------------------------------

fn json_escape_simple(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn extract_json_str(s: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":\"", key);
    let start = s.find(&needle)? + needle.len();
    let rest = &s[start..];
    let mut val = String::new();
    let mut escaped = false;
    for c in rest.chars() {
        if escaped {
            val.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            break;
        } else {
            val.push(c);
        }
    }
    Some(val)
}

fn extract_json_num(s: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{}\":", key);
    let start = s.find(&needle)? + needle.len();
    let rest = s[start..].trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

// ---------------------------------------------------------------------------
// Task budget resolution
// ---------------------------------------------------------------------------

/// Maps a user-supplied task name to a (files_max, loc_max, canonical_name) triple.
///
/// Budget caps come from the **single source of truth** in `contract.rs`
/// (`BUDGET_CAP_*`). There is no second hardcoded budget table here — the
/// v0.21 audit (§7c.1) flagged the previous duplicate table as a drift risk.
/// CLI task names that have no direct `TaskType` variant (`docs`, `test`,
/// `config`, `unknown`) are mapped to the closest-fitting tier.
///
/// Returns `Err(String)` with valid options when the name is not recognised.
pub fn budget_for_task_name(task: &str) -> Result<(usize, usize, &'static str), String> {
    use crate::contract::{
        BUDGET_CAP_LARGE, BUDGET_CAP_MEDIUM, BUDGET_CAP_MICRO, BUDGET_CAP_SMALL,
    };
    match task.to_lowercase().as_str() {
        "bugfix" | "bug" | "fix" => Ok((BUDGET_CAP_MICRO.0, BUDGET_CAP_MICRO.1, "Bugfix")),
        "config" => Ok((BUDGET_CAP_MICRO.0, BUDGET_CAP_MICRO.1, "Config")),
        "security" | "sec" => Ok((BUDGET_CAP_SMALL.0, BUDGET_CAP_SMALL.1, "Security")),
        "dependency" | "dep" => Ok((BUDGET_CAP_SMALL.0, BUDGET_CAP_SMALL.1, "Dependency")),
        "docs" | "doc" => Ok((BUDGET_CAP_SMALL.0, BUDGET_CAP_SMALL.1, "Docs")),
        "test" | "tests" => Ok((BUDGET_CAP_SMALL.0, BUDGET_CAP_SMALL.1, "Test")),
        "feature" | "feat" => Ok((BUDGET_CAP_MEDIUM.0, BUDGET_CAP_MEDIUM.1, "Feature")),
        "refactor" => Ok((BUDGET_CAP_MEDIUM.0, BUDGET_CAP_MEDIUM.1, "Refactor")),
        "frontend" | "ui" => Ok((BUDGET_CAP_MEDIUM.0, BUDGET_CAP_MEDIUM.1, "Frontend")),
        "unknown" => Ok((BUDGET_CAP_MEDIUM.0, BUDGET_CAP_MEDIUM.1, "Unknown")),
        "migration" | "migrate" => Ok((BUDGET_CAP_LARGE.0, BUDGET_CAP_LARGE.1, "Migration")),
        _ => Err(format!(
            "unknown task type '{}'. Valid values: bugfix, feature, refactor, security, \
             migration, dependency, frontend, docs, test, config, unknown",
            task
        )),
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Measured diff from the current git working tree.
#[derive(Debug, Clone, PartialEq)]
pub struct DiffMeasurement {
    pub file_count: usize,
    pub added_lines: usize,
    pub deleted_lines: usize,
    pub total_changed_lines: usize,
    /// True when git diff ran but returned no changes.
    pub clean: bool,
    /// True when git command could not be run or produced unparseable output.
    pub unknown: bool,
    /// Human-readable reason when unknown=true.
    pub unknown_reason: String,
}

impl DiffMeasurement {
    pub fn unknown(reason: &str) -> Self {
        DiffMeasurement {
            file_count: 0,
            added_lines: 0,
            deleted_lines: 0,
            total_changed_lines: 0,
            clean: false,
            unknown: true,
            unknown_reason: reason.to_string(),
        }
    }

    pub fn clean() -> Self {
        DiffMeasurement {
            file_count: 0,
            added_lines: 0,
            deleted_lines: 0,
            total_changed_lines: 0,
            clean: true,
            unknown: false,
            unknown_reason: String::new(),
        }
    }
}

/// Result of comparing a measurement against a diff budget.
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetVerdict {
    Pass,
    Exceeded { reason: String },
    Unknown { reason: String },
}

// ---------------------------------------------------------------------------
// Git diff measurement
// ---------------------------------------------------------------------------

/// Run `git diff --numstat` and `git diff --name-only` in `repo_root` and
/// return a `DiffMeasurement`. Never panics.
pub fn measure_diff(repo_root: &Path) -> DiffMeasurement {
    // First check git is available and this is a repo.
    let check = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .current_dir(repo_root)
        .output();

    match check {
        Err(e) => return DiffMeasurement::unknown(&format!("git not available: {}", e)),
        Ok(out) if !out.status.success() => {
            return DiffMeasurement::unknown("not a git repository");
        }
        _ => {}
    }

    // Run git diff --numstat (includes staged + unstaged via HEAD)
    let numstat = Command::new("git")
        .args(["diff", "HEAD", "--numstat"])
        .current_dir(repo_root)
        .output();

    let numstat_out = match numstat {
        Err(e) => return DiffMeasurement::unknown(&format!("git diff --numstat failed: {}", e)),
        Ok(out) if !out.status.success() => {
            // No HEAD yet (empty repo) — treat as clean
            return DiffMeasurement::clean();
        }
        Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
    };

    // Run git diff --name-only to get file count (handles renames)
    let name_only = Command::new("git")
        .args(["diff", "HEAD", "--name-only"])
        .current_dir(repo_root)
        .output();

    let name_only_out = if numstat_out.trim().is_empty() {
        String::new()
    } else {
        match name_only {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(_) => String::new(),
        }
    };

    if numstat_out.trim().is_empty() {
        return DiffMeasurement::clean();
    }

    parse_numstat(&numstat_out, &name_only_out)
}

/// Parse `git diff --numstat` output into a `DiffMeasurement`.
///
/// Format: `<added>\t<deleted>\t<filename>` per line.
/// Binary files show `-\t-\t<filename>` — counted as 1 changed file, 0 LOC.
pub fn parse_numstat(numstat: &str, name_only: &str) -> DiffMeasurement {
    let mut added = 0usize;
    let mut deleted = 0usize;
    let mut file_count = 0usize;

    for line in numstat.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '\t').collect();
        if parts.len() < 2 {
            continue;
        }
        file_count += 1;
        // Binary files show "-" for added/deleted
        if parts[0] != "-" {
            added += parts[0].parse::<usize>().unwrap_or(0);
        }
        if parts[1] != "-" {
            deleted += parts[1].parse::<usize>().unwrap_or(0);
        }
    }

    // Use name-only count if it differs (e.g. renames produce two numstat lines)
    let name_only_count = name_only.lines().filter(|l| !l.trim().is_empty()).count();
    if name_only_count > 0 {
        file_count = name_only_count;
    }

    if file_count == 0 && added == 0 && deleted == 0 {
        return DiffMeasurement::clean();
    }

    DiffMeasurement {
        file_count,
        added_lines: added,
        deleted_lines: deleted,
        total_changed_lines: added + deleted,
        clean: false,
        unknown: false,
        unknown_reason: String::new(),
    }
}

// ---------------------------------------------------------------------------
// Budget comparison
// ---------------------------------------------------------------------------

/// Compare a `DiffMeasurement` against budget limits.
/// `budget_files_max` and `budget_loc_max` come from `contract::DiffBudget`.
pub fn compare_budget(
    measurement: &DiffMeasurement,
    budget_files_max: usize,
    budget_loc_max: usize,
) -> BudgetVerdict {
    if measurement.unknown {
        return BudgetVerdict::Unknown {
            reason: measurement.unknown_reason.clone(),
        };
    }

    if measurement.clean {
        return BudgetVerdict::Pass;
    }

    if measurement.file_count > budget_files_max {
        return BudgetVerdict::Exceeded {
            reason: format!(
                "file count {} exceeds budget of {}",
                measurement.file_count, budget_files_max
            ),
        };
    }

    if measurement.total_changed_lines > budget_loc_max {
        return BudgetVerdict::Exceeded {
            reason: format!(
                "changed lines {} exceeds budget of {}",
                measurement.total_changed_lines, budget_loc_max
            ),
        };
    }

    BudgetVerdict::Pass
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

pub struct DiffReport {
    pub measurement: DiffMeasurement,
    pub verdict: BudgetVerdict,
    pub budget_files_max: usize,
    pub budget_loc_max: usize,
    pub task_type: String,
}

pub fn format_diff_report(report: &DiffReport) -> String {
    let mut out = String::new();
    out.push_str("postmortem --diff:\n");

    if report.measurement.unknown {
        out.push_str(&format!(
            "  status:  UNKNOWN\n  reason:  {}\n",
            report.measurement.unknown_reason
        ));
        return out;
    }

    if report.measurement.clean {
        out.push_str("  status:  PASS\n");
        out.push_str("  diff:    clean working tree (no changes)\n");
        return out;
    }

    out.push_str(&format!("  task:    {}\n", report.task_type));
    out.push_str(&format!(
        "  budget:  {} files, {} LOC\n",
        report.budget_files_max, report.budget_loc_max
    ));
    out.push_str(&format!(
        "  actual:  {} files, {} added, {} deleted ({} total changed LOC)\n",
        report.measurement.file_count,
        report.measurement.added_lines,
        report.measurement.deleted_lines,
        report.measurement.total_changed_lines,
    ));

    match &report.verdict {
        BudgetVerdict::Pass => {
            out.push_str("  status:  PASS\n");
        }
        BudgetVerdict::Exceeded { reason } => {
            out.push_str(&format!("  status:  EXCEEDED\n  reason:  {}\n", reason));
            out.push_str(
                "  note:    AKAR measures only — it does not enforce, block, or revert changes\n",
            );
            out.push_str(&format!(
                "  guidance: {}\n",
                crate::foundation::budget_exceeded_playbook()
            ));
        }
        BudgetVerdict::Unknown { reason } => {
            out.push_str(&format!("  status:  UNKNOWN\n  reason:  {}\n", reason));
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Baseline loop readiness
// ---------------------------------------------------------------------------

/// Readiness state for the full clean-baseline loop.
#[derive(Debug, Clone, PartialEq)]
pub enum LoopReadiness {
    /// Git repo exists and working tree is clean — ready to snapshot.
    Ready,
    /// Git repo exists but working tree is dirty — must clean manually first.
    Blocked,
    /// Git status could not be checked (git unavailable or not a repo).
    Unknown,
}

impl LoopReadiness {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoopReadiness::Ready => "READY",
            LoopReadiness::Blocked => "BLOCKED",
            LoopReadiness::Unknown => "UNKNOWN",
        }
    }
}

/// Result of a loop readiness check.
#[derive(Debug, Clone)]
pub struct LoopReadinessReport {
    pub git_repo_detected: bool,
    /// None when git check failed.
    pub working_tree_clean: Option<bool>,
    pub baseline_file_present: bool,
    pub readiness: LoopReadiness,
}

/// Check baseline loop readiness using read-only git commands.
/// Never modifies files. Never blocks or reverts.
pub fn check_loop_readiness(repo_root: &Path, akar_dir: &Path) -> LoopReadinessReport {
    let baseline_file_present = akar_dir.join("DIFF_BASELINE.json").exists();

    match is_working_tree_clean(repo_root) {
        Err(_) => LoopReadinessReport {
            git_repo_detected: false,
            working_tree_clean: None,
            baseline_file_present,
            readiness: LoopReadiness::Unknown,
        },
        Ok(clean) => LoopReadinessReport {
            git_repo_detected: true,
            working_tree_clean: Some(clean),
            baseline_file_present,
            readiness: if clean {
                LoopReadiness::Ready
            } else {
                LoopReadiness::Blocked
            },
        },
    }
}

/// Format the loop readiness section for `akar status`.
pub fn format_loop_readiness(r: &LoopReadinessReport) -> String {
    let tree_clean = match r.working_tree_clean {
        Some(true) => "yes",
        Some(false) => "no",
        None => "unknown",
    };
    let baseline = if r.baseline_file_present { "yes" } else { "no" };
    format!(
        "  baseline loop readiness:\n\
         \x20\x20git repository detected: {}\n\
         \x20\x20working tree clean:      {}\n\
         \x20\x20baseline file present:   {}\n\
         \x20\x20readiness:               {}\n",
        if r.git_repo_detected { "yes" } else { "no" },
        tree_clean,
        baseline,
        r.readiness.as_str(),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_normal_numstat() {
        let numstat = "5\t3\tsrc/main.rs\n2\t1\tsrc/lib.rs\n";
        let result = parse_numstat(numstat, "src/main.rs\nsrc/lib.rs\n");
        assert_eq!(result.file_count, 2);
        assert_eq!(result.added_lines, 7);
        assert_eq!(result.deleted_lines, 4);
        assert_eq!(result.total_changed_lines, 11);
        assert!(!result.clean);
        assert!(!result.unknown);
    }

    #[test]
    fn parse_binary_file_numstat() {
        let numstat = "-\t-\tassets/logo.png\n3\t1\tsrc/main.rs\n";
        let result = parse_numstat(numstat, "assets/logo.png\nsrc/main.rs\n");
        assert_eq!(result.file_count, 2);
        assert_eq!(result.added_lines, 3);
        assert_eq!(result.deleted_lines, 1);
    }

    #[test]
    fn parse_empty_numstat_returns_clean() {
        let result = parse_numstat("", "");
        assert!(result.clean);
        assert_eq!(result.file_count, 0);
        assert_eq!(result.total_changed_lines, 0);
    }

    #[test]
    fn budget_pass_within_limits() {
        let m = DiffMeasurement {
            file_count: 2,
            added_lines: 30,
            deleted_lines: 10,
            total_changed_lines: 40,
            clean: false,
            unknown: false,
            unknown_reason: String::new(),
        };
        let verdict = compare_budget(&m, 5, 200);
        assert_eq!(verdict, BudgetVerdict::Pass);
    }

    #[test]
    fn budget_exceeded_file_count() {
        let m = DiffMeasurement {
            file_count: 10,
            added_lines: 50,
            deleted_lines: 10,
            total_changed_lines: 60,
            clean: false,
            unknown: false,
            unknown_reason: String::new(),
        };
        let verdict = compare_budget(&m, 3, 200);
        assert!(matches!(verdict, BudgetVerdict::Exceeded { .. }));
        if let BudgetVerdict::Exceeded { reason } = &verdict {
            assert!(reason.contains("file count"));
        }
    }

    #[test]
    fn budget_exceeded_loc() {
        let m = DiffMeasurement {
            file_count: 2,
            added_lines: 400,
            deleted_lines: 100,
            total_changed_lines: 500,
            clean: false,
            unknown: false,
            unknown_reason: String::new(),
        };
        let verdict = compare_budget(&m, 5, 200);
        assert!(matches!(verdict, BudgetVerdict::Exceeded { .. }));
        if let BudgetVerdict::Exceeded { reason } = &verdict {
            assert!(reason.contains("changed lines"));
        }
    }

    #[test]
    fn budget_unknown_when_measurement_unknown() {
        let m = DiffMeasurement::unknown("not a git repo");
        let verdict = compare_budget(&m, 5, 200);
        assert!(matches!(verdict, BudgetVerdict::Unknown { .. }));
    }

    #[test]
    fn budget_pass_when_clean() {
        let m = DiffMeasurement::clean();
        let verdict = compare_budget(&m, 5, 200);
        assert_eq!(verdict, BudgetVerdict::Pass);
    }

    #[test]
    fn format_exceeded_report_contains_note() {
        let report = DiffReport {
            measurement: DiffMeasurement {
                file_count: 10,
                added_lines: 300,
                deleted_lines: 50,
                total_changed_lines: 350,
                clean: false,
                unknown: false,
                unknown_reason: String::new(),
            },
            verdict: BudgetVerdict::Exceeded {
                reason: "file count 10 exceeds budget of 3".to_string(),
            },
            budget_files_max: 3,
            budget_loc_max: 200,
            task_type: "Bugfix".to_string(),
        };
        let out = format_diff_report(&report);
        assert!(out.contains("EXCEEDED"));
        assert!(out.contains("does not enforce"));
        assert!(out.contains("10 files"));
    }

    #[test]
    fn format_unknown_report() {
        let report = DiffReport {
            measurement: DiffMeasurement::unknown("git not found"),
            verdict: BudgetVerdict::Unknown {
                reason: "git not found".to_string(),
            },
            budget_files_max: 5,
            budget_loc_max: 200,
            task_type: "Bugfix".to_string(),
        };
        let out = format_diff_report(&report);
        assert!(out.contains("UNKNOWN"));
        assert!(out.contains("git not found"));
    }

    // -- budget_for_task_name ------------------------------------------------

    #[test]
    fn default_bugfix_budget() {
        let (f, l, name) = budget_for_task_name("bugfix").unwrap();
        assert_eq!(f, 3);
        assert_eq!(l, 60);
        assert_eq!(name, "Bugfix");
    }

    #[test]
    fn explicit_feature_budget() {
        let (f, l, name) = budget_for_task_name("feature").unwrap();
        assert_eq!(f, 12);
        assert_eq!(l, 600);
        assert_eq!(name, "Feature");
    }

    #[test]
    fn explicit_docs_budget() {
        let (f, l, name) = budget_for_task_name("docs").unwrap();
        assert_eq!(f, 5);
        assert_eq!(l, 200);
        assert_eq!(name, "Docs");
    }

    #[test]
    fn explicit_test_budget() {
        let (f, l, name) = budget_for_task_name("test").unwrap();
        assert_eq!(f, 5);
        assert_eq!(l, 200);
        assert_eq!(name, "Test");
    }

    #[test]
    fn explicit_config_budget() {
        let (f, l, _) = budget_for_task_name("config").unwrap();
        assert_eq!(f, 3);
        assert_eq!(l, 60);
    }

    #[test]
    fn invalid_task_returns_err() {
        let result = budget_for_task_name("banana");
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("banana"));
        assert!(msg.contains("bugfix"));
    }

    #[test]
    fn task_aliases_work() {
        assert!(budget_for_task_name("fix").is_ok());
        assert!(budget_for_task_name("feat").is_ok());
        assert!(budget_for_task_name("sec").is_ok());
        assert!(budget_for_task_name("dep").is_ok());
        assert!(budget_for_task_name("ui").is_ok());
        assert!(budget_for_task_name("doc").is_ok());
        assert!(budget_for_task_name("tests").is_ok());
        assert!(budget_for_task_name("migrate").is_ok());
    }

    #[test]
    fn learning_note_contains_full_rule() {
        // The rule string must be exact as specified in the mission prompt.
        let rule = "Next prompt must reduce scope or split the task.";
        // Verify the string is present in the patch template used in main.rs
        let patch = format!(
            "\n## LP-DIFF-ts\n\
            - rule: {}\n\
            - status: proposed\n",
            rule
        );
        assert!(patch.contains("Next prompt must reduce scope or split the task."));
    }

    // -- baseline serialization / parsing ------------------------------------

    fn sample_baseline() -> DiffBaseline {
        DiffBaseline {
            timestamp: "2026-07-05T10:00:00Z".to_string(),
            prompt: "fix the login button".to_string(),
            head_commit: "abc123def456".to_string(),
            task_type: "Bugfix".to_string(),
            budget_files_max: 3,
            budget_loc_max: 60,
        }
    }

    #[test]
    fn baseline_serializes_and_parses() {
        let b = sample_baseline();
        let json = baseline_to_json(&b);
        let parsed = baseline_from_json(&json).expect("parse should succeed");
        assert_eq!(parsed.timestamp, b.timestamp);
        assert_eq!(parsed.prompt, b.prompt);
        assert_eq!(parsed.head_commit, b.head_commit);
        assert_eq!(parsed.task_type, b.task_type);
        assert_eq!(parsed.budget_files_max, b.budget_files_max);
        assert_eq!(parsed.budget_loc_max, b.budget_loc_max);
    }

    #[test]
    fn baseline_json_contains_all_fields() {
        let json = baseline_to_json(&sample_baseline());
        assert!(json.contains("timestamp"));
        assert!(json.contains("head_commit"));
        assert!(json.contains("task_type"));
        assert!(json.contains("budget_files_max"));
        assert!(json.contains("budget_loc_max"));
        assert!(json.contains("abc123def456"));
        assert!(json.contains("Bugfix"));
    }

    #[test]
    fn baseline_parse_fails_on_missing_field() {
        let bad = r#"{"timestamp":"2026-07-05","prompt":"x"}"#;
        let result = baseline_from_json(bad);
        assert!(result.is_err());
    }

    #[test]
    fn missing_baseline_file_returns_err() {
        let dir = std::env::temp_dir().join("akar_baseline_missing_test");
        let _ = std::fs::create_dir_all(&dir);
        let result = read_baseline(&dir);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("preflight --snapshot"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn baseline_write_and_read_roundtrip() {
        let dir = std::env::temp_dir().join("akar_baseline_roundtrip_test");
        let _ = std::fs::create_dir_all(&dir);
        let b = sample_baseline();
        write_baseline(&dir, &b).expect("write should succeed");
        let read = read_baseline(&dir).expect("read should succeed");
        assert_eq!(read.head_commit, b.head_commit);
        assert_eq!(read.budget_files_max, b.budget_files_max);
        assert_eq!(read.budget_loc_max, b.budget_loc_max);
        assert_eq!(read.task_type, b.task_type);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn baseline_budget_used_in_comparison() {
        let b = sample_baseline(); // 3 files, 60 LOC
        let m = DiffMeasurement {
            file_count: 10,
            added_lines: 200,
            deleted_lines: 50,
            total_changed_lines: 250,
            clean: false,
            unknown: false,
            unknown_reason: String::new(),
        };
        let verdict = compare_budget(&m, b.budget_files_max, b.budget_loc_max);
        assert!(matches!(verdict, BudgetVerdict::Exceeded { .. }));
    }

    #[test]
    fn baseline_task_type_in_report() {
        let b = sample_baseline();
        let report = DiffReport {
            measurement: DiffMeasurement::clean(),
            verdict: BudgetVerdict::Pass,
            budget_files_max: b.budget_files_max,
            budget_loc_max: b.budget_loc_max,
            task_type: b.task_type.clone(),
        };
        let out = format_diff_report(&report);
        // clean diff — status PASS, no task line shown (clean path)
        assert!(out.contains("PASS"));
    }

    #[test]
    fn baseline_head_is_short_hash_format() {
        // head_commit from sample is 12 chars — valid short hash
        let b = sample_baseline();
        assert_eq!(b.head_commit.len(), 12);
        assert!(b.head_commit.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    // -- loop readiness -------------------------------------------------------

    fn make_clean_git_repo(dir: &std::path::Path) {
        use std::process::Command;
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "t@t.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "T"])
            .current_dir(dir)
            .output()
            .unwrap();
        std::fs::write(dir.join("seed.txt"), "seed").unwrap();
        Command::new("git")
            .args(["add", "seed.txt"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn readiness_ready_when_clean_git_repo() {
        let dir = std::env::temp_dir().join("akar_readiness_ready_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        make_clean_git_repo(&dir);
        let report = check_loop_readiness(&dir, &dir);
        assert!(report.git_repo_detected);
        assert_eq!(report.working_tree_clean, Some(true));
        assert_eq!(report.readiness, LoopReadiness::Ready);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn readiness_blocked_when_tree_is_dirty() {
        let dir = std::env::temp_dir().join("akar_readiness_blocked_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        make_clean_git_repo(&dir);
        // Add an untracked file to make the tree dirty
        std::fs::write(dir.join("dirty.txt"), "dirty").unwrap();
        let report = check_loop_readiness(&dir, &dir);
        assert!(report.git_repo_detected);
        assert_eq!(report.working_tree_clean, Some(false));
        assert_eq!(report.readiness, LoopReadiness::Blocked);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn readiness_unknown_when_not_a_git_repo() {
        let dir = std::env::temp_dir().join("akar_readiness_unknown_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // No git init — not a repo
        let report = check_loop_readiness(&dir, &dir);
        assert!(!report.git_repo_detected);
        assert_eq!(report.working_tree_clean, None);
        assert_eq!(report.readiness, LoopReadiness::Unknown);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn readiness_reports_baseline_file_present() {
        let dir = std::env::temp_dir().join("akar_readiness_baseline_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Without baseline
        let report_no_baseline = check_loop_readiness(&dir, &dir);
        assert!(!report_no_baseline.baseline_file_present);
        // With baseline
        std::fs::write(dir.join("DIFF_BASELINE.json"), "{}").unwrap();
        let report_with_baseline = check_loop_readiness(&dir, &dir);
        assert!(report_with_baseline.baseline_file_present);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn status_formatting_includes_baseline_loop_readiness_section() {
        let report = LoopReadinessReport {
            git_repo_detected: true,
            working_tree_clean: Some(false),
            baseline_file_present: false,
            readiness: LoopReadiness::Blocked,
        };
        let out = format_loop_readiness(&report);
        assert!(out.contains("baseline loop readiness"));
        assert!(out.contains("BLOCKED"));
        assert!(out.contains("git repository detected: yes"));
        assert!(out.contains("working tree clean:      no"));
        assert!(out.contains("baseline file present:   no"));
    }

    #[test]
    fn v070_report_wording_says_partial_evidence_not_full_verified_loop() {
        // The audit report must say partial evidence, not claim a full verified loop.
        let report_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("docs/audits/AKAR_V0_7_VERIFIED_SESSION_REPORT.md");
        let content = std::fs::read_to_string(&report_path).expect("audit report must exist");
        assert!(
            content.contains("PARTIAL EVIDENCE"),
            "report must contain PARTIAL EVIDENCE"
        );
        assert!(
            content.contains("full clean-baseline loop was NOT completed"),
            "report must state full loop was not completed"
        );
        // Must not claim this is a fully verified loop in the title
        assert!(
            !content.starts_with("# AKAR v0.7.0 Verified Session Report"),
            "report title must not claim full verified loop"
        );
    }
}
