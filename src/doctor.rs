/// Phase 5 Doctor — stub for parallel implementation.
/// The full read-only check engine will be filled in by the Phase 5 agent.
/// Phase 6 (safe_fix) depends on this module for the `Issue` and `Check` types.

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Issue types — Phase 6 safe_fix acts on these
// ---------------------------------------------------------------------------

/// Severity of a doctor finding.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// A single issue found during a doctor run.
#[derive(Debug, Clone)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
    /// The kind of fix that can automatically resolve this issue, if any.
    pub fix_hint: Option<FixHint>,
}

/// Hints that tell safe_fix what action to take for an issue.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum FixHint {
    /// The directory at this path is missing and should be created.
    CreateDir(PathBuf),
    /// A template file should be copied to `dest`.
    CreateFromTemplate { dest: PathBuf, template_name: String },
}

// ---------------------------------------------------------------------------
// Doctor run
// ---------------------------------------------------------------------------

/// Run the read-only doctor checks against `cfg` and return all findings.
/// Stub: currently delegates to config validation only.
pub fn run_checks(cfg: &crate::config::Config) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Reuse Config::validate() until the full Phase 5 engine lands.
    for msg in cfg.validate() {
        // Infer a fix hint where possible.
        let fix_hint = if msg.contains(".akar") {
            Some(FixHint::CreateDir(cfg.akar_dir.clone()))
        } else if msg.contains("global akar") {
            Some(FixHint::CreateDir(cfg.global_dir.clone()))
        } else {
            None
        };

        issues.push(Issue {
            severity: Severity::Error,
            message: msg,
            fix_hint,
        });
    }

    issues
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

    #[test]
    fn run_checks_returns_issues_when_dirs_missing() {
        let cfg = make_cfg_all_missing();
        let issues = run_checks(&cfg);
        assert!(!issues.is_empty(), "expected issues for missing dirs");
    }

    #[test]
    fn run_checks_returns_no_issues_when_dirs_exist() {
        let root = std::env::current_dir().unwrap();
        let cfg = crate::config::Config {
            akar_dir: root.clone(),
            global_dir: root.clone(),
            project_name: "stub-test".to_string(),
            project_root: root,
        };
        let issues = run_checks(&cfg);
        assert!(issues.is_empty(), "expected no issues when dirs exist");
    }
}
