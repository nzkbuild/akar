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
/// Checks performed (in order):
/// 1. `.akar/DESIGN_DNA.md` exists → warning if absent.
/// 2. Any `.css` / `.scss` / `.tailwind` file present without design DNA →
///    warning (frontend files detected without design guidance).
/// 3. `index.html` present without design DNA → info.
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

    // Scan for frontend files only when design DNA is absent — if it's present
    // there is nothing to warn about.
    if !has_design_dna {
        let has_frontend = has_files_with_extensions(project_root, &["css", "scss", "tailwind"]);
        if has_frontend {
            issues.push(DesignIssue {
                severity: "warning".to_string(),
                check: "frontend_without_design_dna".to_string(),
                message: "frontend files detected without design DNA".to_string(),
            });
        }

        let index_html = project_root.join("index.html");
        if index_html.exists() {
            issues.push(DesignIssue {
                severity: "info".to_string(),
                check: "html_without_design_dna".to_string(),
                message: "HTML file found, design module recommended".to_string(),
            });
        }
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
// Helpers
// ---------------------------------------------------------------------------

/// Return true if any file under `root` (recursively) has one of the given
/// extensions. Silently skips directories / entries that cannot be read.
fn has_files_with_extensions(root: &Path, extensions: &[&str]) -> bool {
    walk_has_extension(root, extensions)
}

fn walk_has_extension(dir: &Path, extensions: &[&str]) -> bool {
    let rd = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return false,
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden dirs and common non-source dirs to keep it fast.
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            if walk_has_extension(&path, extensions) {
                return true;
            }
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.contains(&ext) {
                return true;
            }
        }
    }
    false
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
        // The AKAR project has no .akar/ at runtime (it's a Rust CLI project,
        // not a deployed AKAR workspace), so we mainly verify the function
        // runs without panicking and returns a coherent report.
        let root = project_root();
        let report = check_project(&root);
        // has_design_dna reflects whether .akar/DESIGN_DNA.md exists on disk.
        // Either outcome is valid; we just assert the struct is well-formed.
        let _ = report.has_design_dna;
        // Issues must be a Vec (possibly empty).
        let _ = &report.issues;
    }

    #[test]
    fn check_project_no_frontend_files_means_no_frontend_warning() {
        // AKAR source tree has no .css/.scss/.tailwind files, so even if
        // DESIGN_DNA.md is absent the frontend warning should NOT fire.
        let root = project_root();
        let report = check_project(&root);
        let has_frontend_warn = report.issues.iter().any(|i| i.check == "frontend_without_design_dna");
        assert!(
            !has_frontend_warn,
            "no frontend files in this project, but frontend warning fired"
        );
    }

    // -- missing DESIGN_DNA warning ------------------------------------------

    #[test]
    fn missing_design_dna_produces_warning() {
        // Point check_project at a temp dir that has no .akar/ subdirectory.
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

        // Cleanup (best-effort).
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
