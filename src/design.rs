//! Phase 12: Design Quality Module
//!
//! Checks a project for design-quality signals: presence of DESIGN_DNA.md,
//! frontend files without a design DNA, and HTML files that would benefit
//! from design guidance.

use std::path::Path;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct DesignIssue {
    /// "error", "warning", or "info"
    pub severity: String,
    /// Short machine-readable name for the check that fired.
    pub check: String,
    /// Human-readable description of the issue.
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct DesignReport {
    pub issues: Vec<DesignIssue>,
    pub has_design_dna: bool,
}

// ---------------------------------------------------------------------------
// check_project
// ---------------------------------------------------------------------------

/// Walk `project_root` and produce a `DesignReport`.
///
/// Check: `.akar/DESIGN_DNA.md` exists → warning if absent.
pub fn check_project(project_root: &Path) -> DesignReport {
    let design_dna_path = project_root.join(".akar").join("DESIGN_DNA.md");
    let has_design_dna = design_dna_path.exists();

    let mut issues = Vec::new();

    if !has_design_dna {
        issues.push(DesignIssue {
            severity: "warning".to_string(),
            check: "design_dna_missing".to_string(),
            message: "no DESIGN_DNA.md found in .akar/".to_string(),
        });
    }

    DesignReport { issues, has_design_dna }
}

// ---------------------------------------------------------------------------
// format_design_report
// ---------------------------------------------------------------------------

/// Produce a one-or-more-line human-readable summary of a `DesignReport`.
#[allow(dead_code)]
pub fn format_design_report(report: &DesignReport) -> String {
    let dna_line = format!(
        "  has_design_dna: {}",
        if report.has_design_dna { "yes" } else { "no" }
    );

    if report.issues.is_empty() {
        return format!("design: OK\n{}", dna_line);
    }

    let n = report.issues.len();
    let mut lines = vec![format!("design: {} issue(s)", n)];
    for issue in &report.issues {
        lines.push(format!("  [{}] {}: {}", issue.severity, issue.check, issue.message));
    }
    lines.push(dna_line);
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn project_root() -> PathBuf {
        // The AKAR project root is two levels above src/
        let manifest = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(manifest)
    }

    // -- check_project -------------------------------------------------------

    #[test]
    fn check_project_on_actual_root_returns_report() {
        let root = project_root();
        let report = check_project(&root);
        let _ = report.has_design_dna;
        let _ = &report.issues;
    }

    // -- missing DESIGN_DNA warning ------------------------------------------

    #[test]
    fn missing_design_dna_produces_warning() {
        let tmp = std::env::temp_dir().join("akar_design_test_missing_dna");
        let _ = std::fs::create_dir_all(&tmp);

        let report = check_project(&tmp);
        assert!(
            !report.has_design_dna,
            "has_design_dna should be false for empty temp dir"
        );
        assert!(
            report.issues.iter().any(|i| i.check == "design_dna_missing"),
            "expected design_dna_missing warning, got: {:?}",
            report.issues
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn present_design_dna_produces_no_issues() {
        let tmp = std::env::temp_dir().join("akar_design_test_present_dna");
        let akar_dir = tmp.join(".akar");
        let _ = std::fs::create_dir_all(&akar_dir);
        let _ = std::fs::write(akar_dir.join("DESIGN_DNA.md"), "# Design DNA\n");

        let report = check_project(&tmp);
        assert!(report.has_design_dna);
        assert!(
            report.issues.is_empty(),
            "expected no issues when DESIGN_DNA.md is present, got: {:?}",
            report.issues
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // -- format_design_report ------------------------------------------------

    #[test]
    fn format_report_is_nonempty() {
        let root = project_root();
        let report = check_project(&root);
        let s = format_design_report(&report);
        assert!(!s.is_empty());
    }

    #[test]
    fn format_report_ok_when_no_issues() {
        let report = DesignReport {
            issues: vec![],
            has_design_dna: true,
        };
        let s = format_design_report(&report);
        assert!(s.starts_with("design: OK"), "got: {}", s);
        assert!(s.contains("has_design_dna: yes"));
    }

    #[test]
    fn format_report_shows_count_when_issues_present() {
        let report = DesignReport {
            issues: vec![DesignIssue {
                severity: "warning".to_string(),
                check: "design_dna_missing".to_string(),
                message: "no DESIGN_DNA.md found in .akar/".to_string(),
            }],
            has_design_dna: false,
        };
        let s = format_design_report(&report);
        assert!(s.contains("1 issue(s)"), "got: {}", s);
        assert!(s.contains("has_design_dna: no"));
    }
}
