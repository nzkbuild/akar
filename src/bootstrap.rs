/// Bootstrap engine — copies templates into .akar/ for a fresh project.
///
/// Idempotent: never overwrites files that already exist.

use std::path::{Path, PathBuf};

use crate::config;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Outcome of a bootstrap run.
#[derive(Debug, Default)]
pub struct BootstrapResult {
    /// Relative paths that were newly created (template files copied).
    pub created: Vec<String>,
    /// Relative paths that were skipped because the destination already existed.
    pub skipped: Vec<String>,
    /// Non-fatal errors and warnings accumulated during the run.
    pub warnings: Vec<String>,
    /// True if `.akar/` was newly created by this run (v0.25 honesty fix).
    pub akar_dir_created: bool,
    /// True if `~/.claude/akar/` was newly created by this run.
    pub global_dir_created: bool,
}

// ---------------------------------------------------------------------------
// Core logic
// ---------------------------------------------------------------------------

/// Run bootstrap against the given config.
///
/// Steps:
/// 1. Ensure `cfg.akar_dir` exists (create_dir_all).
/// 2. Ensure `cfg.global_dir` exists (create_dir_all).
/// 3. Locate the templates directory.
/// 4. Copy each `.md` file from templates/ to `cfg.akar_dir/` — skip if dest exists.
pub fn run_bootstrap(cfg: &config::Config) -> BootstrapResult {
    let mut result = BootstrapResult::default();

    // 1. Ensure .akar/ exists.
    if !cfg.akar_dir.exists() {
        match std::fs::create_dir_all(&cfg.akar_dir) {
            Ok(()) => result.akar_dir_created = true,
            Err(e) => {
                result.warnings.push(format!(
                    "could not create akar_dir {}: {}",
                    cfg.akar_dir.display(),
                    e
                ));
                // Without the target dir we cannot copy anything useful, but keep going
                // so global_dir still gets created.
            }
        }
    }

    // 2. Ensure ~/.claude/akar/ exists.
    if !cfg.global_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&cfg.global_dir) {
            result.warnings.push(format!(
                "could not create global_dir {}: {}",
                cfg.global_dir.display(),
                e
            ));
        } else {
            result.global_dir_created = true;
        }
    }

    // 3. Find templates directory.
    let templates_dir = match find_templates_dir(&cfg.project_root) {
        Some(p) => p,
        None => {
            result
                .warnings
                .push("templates directory not found".to_string());
            return result;
        }
    };

    // 4. Walk templates/ and copy .md files.
    let entries = match std::fs::read_dir(&templates_dir) {
        Ok(e) => e,
        Err(e) => {
            result.warnings.push(format!(
                "could not read templates dir {}: {}",
                templates_dir.display(),
                e
            ));
            return result;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                result
                    .warnings
                    .push(format!("error reading template entry: {}", e));
                continue;
            }
        };

        let src = entry.path();

        // Only process .md files.
        if src.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let file_name = match src.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => {
                result
                    .warnings
                    .push(format!("skipping entry with invalid filename: {}", src.display()));
                continue;
            }
        };

        let dest = cfg.akar_dir.join(&file_name);

        if dest.exists() {
            result.skipped.push(file_name);
        } else {
            match std::fs::copy(&src, &dest) {
                Ok(_) => result.created.push(file_name),
                Err(e) => result.warnings.push(format!(
                    "failed to copy {} -> {}: {}",
                    src.display(),
                    dest.display(),
                    e
                )),
            }
        }
    }

    result
}

/// Locate the templates directory.
///
/// Resolution order:
///   1. `project_root/templates`
///   2. `<executable_dir>/templates`
fn find_templates_dir(project_root: &Path) -> Option<PathBuf> {
    let candidate = project_root.join("templates");
    if candidate.is_dir() {
        return Some(candidate);
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let candidate = exe_dir.join("templates");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Format a human-readable bootstrap report.
pub fn format_bootstrap_report(result: &BootstrapResult) -> String {
    let mut out = String::new();

    // Header: distinguish directory creation from template-file copy.
    let mut header_parts = Vec::new();
    if result.akar_dir_created {
        header_parts.push(".akar/ created".to_string());
    } else {
        header_parts.push(".akar/ already present".to_string());
    }
    if result.global_dir_created {
        header_parts.push("global dir created".to_string());
    }
    header_parts.push(format!("{} template file(s) created", result.created.len()));
    header_parts.push(format!("{} skipped", result.skipped.len()));
    out.push_str(&format!("bootstrap: {}\n", header_parts.join(", ")));

    if !result.created.is_empty() {
        out.push('\n');
        out.push_str("  created:\n");
        for name in &result.created {
            out.push_str(&format!("    - {}\n", name));
        }
    }

    if !result.skipped.is_empty() {
        out.push('\n');
        out.push_str("  skipped:\n");
        for name in &result.skipped {
            out.push_str(&format!("    - {} (already exists)\n", name));
        }
    }

    if !result.warnings.is_empty() {
        out.push('\n');
        out.push_str("  warnings:\n");
        for w in &result.warnings {
            out.push_str(&format!("    - {}\n", w));
        }
    }

    out.push('\n');
    out.push_str("  next: run 'akar doctor' to verify\n");

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// A unique temp directory for each test invocation.
    fn temp_dir(label: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("akar_bootstrap_{}_{}", label, ts));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    /// Build a minimal Config pointing at a temp directory.
    /// `project_root` is set to the *real* project root so we can find templates/.
    fn make_cfg(akar_dir: PathBuf, global_dir: PathBuf) -> config::Config {
        // Use the actual workspace root so templates/ is discoverable.
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        config::Config {
            project_root,
            akar_dir,
            global_dir,
            project_name: "test".to_string(),
        }
    }

    #[test]
    fn bootstrap_creates_akar_dir() {
        let base = temp_dir("creates_dir");
        let akar = base.join(".akar");
        let global = base.join("global");

        assert!(!akar.exists(), "precondition: .akar should not exist yet");

        let cfg = make_cfg(akar.clone(), global.clone());
        run_bootstrap(&cfg);

        assert!(akar.exists(), ".akar dir should have been created");
    }

    #[test]
    fn bootstrap_copies_templates() {
        let base = temp_dir("copies_templates");
        let akar = base.join(".akar");
        let global = base.join("global");

        let cfg = make_cfg(akar.clone(), global.clone());
        let result = run_bootstrap(&cfg);

        // We should have created at least one file (templates/ has 9 .md files).
        assert!(
            !result.created.is_empty(),
            "expected at least one created file, got warnings: {:?}",
            result.warnings
        );

        // Every created name should exist on disk.
        for name in &result.created {
            let dest = akar.join(name);
            assert!(dest.exists(), "created file missing on disk: {}", name);
        }

        // Nothing should have been skipped on the first run.
        assert!(
            result.skipped.is_empty(),
            "expected no skips on first run, got: {:?}",
            result.skipped
        );
    }

    #[test]
    fn bootstrap_is_idempotent() {
        let base = temp_dir("idempotent");
        let akar = base.join(".akar");
        let global = base.join("global");
        let cfg = make_cfg(akar.clone(), global.clone());

        let first = run_bootstrap(&cfg);
        assert!(!first.created.is_empty(), "first run should create files");

        let second = run_bootstrap(&cfg);
        assert!(
            second.created.is_empty(),
            "second run should create nothing, got: {:?}",
            second.created
        );
        assert_eq!(
            second.skipped.len(),
            first.created.len(),
            "second run should skip exactly what the first run created"
        );
    }

    #[test]
    fn bootstrap_does_not_overwrite_existing_files() {
        let base = temp_dir("no_overwrite");
        let akar = base.join(".akar");
        let global = base.join("global");
        fs::create_dir_all(&akar).expect("create .akar");
        fs::create_dir_all(&global).expect("create global");

        // Pre-populate one file with custom content.
        let sentinel_name = "PROJECT_DNA.md";
        let sentinel_content = b"CUSTOM USER CONTENT - MUST NOT BE OVERWRITTEN";
        let sentinel_path = akar.join(sentinel_name);
        fs::write(&sentinel_path, sentinel_content).expect("write sentinel");

        let cfg = make_cfg(akar.clone(), global.clone());
        let result = run_bootstrap(&cfg);

        // The sentinel file must appear in skipped, not created.
        assert!(
            result.skipped.contains(&sentinel_name.to_string()),
            "sentinel should be in skipped, got skipped={:?}",
            result.skipped
        );
        assert!(
            !result.created.contains(&sentinel_name.to_string()),
            "sentinel must not appear in created"
        );

        // Content must be unchanged.
        let after = fs::read(&sentinel_path).expect("read sentinel after bootstrap");
        assert_eq!(
            after, sentinel_content,
            "sentinel file content was overwritten — this is a bug"
        );
    }

    #[test]
    fn format_bootstrap_report_shows_correct_counts() {
        let result = BootstrapResult {
            created: vec!["PROJECT_DNA.md".to_string(), "STATE.md".to_string()],
            skipped: vec!["LESSONS.md".to_string()],
            warnings: vec![],
            akar_dir_created: true,
            global_dir_created: false,
        };

        let report = format_bootstrap_report(&result);

        assert!(
            report.contains("bootstrap: .akar/ created, 2 template file(s) created, 1 skipped"),
            "header line wrong, got: {}",
            report
        );
        assert!(report.contains("PROJECT_DNA.md"), "should list created file");
        assert!(report.contains("LESSONS.md (already exists)"), "should list skipped file");
        assert!(report.contains("next: run 'akar doctor' to verify"), "should have next hint");
        // No warnings section when warnings is empty.
        assert!(!report.contains("warnings:"), "should not show empty warnings section");
    }

    #[test]
    fn format_bootstrap_report_distinguishes_dir_creation_from_template_copy() {
        // v0.25 honesty: when bootstrap creates .akar/ but copies 0 templates,
        // the header must still say .akar/ was created (not "0 created, 0 skipped").
        let result = BootstrapResult {
            created: vec![],
            skipped: vec![],
            warnings: vec!["templates directory not found".to_string()],
            akar_dir_created: true,
            global_dir_created: false,
        };
        let report = format_bootstrap_report(&result);
        assert!(
            report.contains(".akar/ created"),
            "header must report .akar/ creation: {}",
            report
        );
        assert!(
            report.contains("0 template file(s) created"),
            "header must report 0 template files: {}",
            report
        );
        assert!(
            report.contains("templates directory not found"),
            "warnings must be shown"
        );
    }

    #[test]
    fn format_bootstrap_report_says_already_present_when_dir_existed() {
        let result = BootstrapResult {
            created: vec![],
            skipped: vec![],
            warnings: vec![],
            akar_dir_created: false,
            global_dir_created: false,
        };
        let report = format_bootstrap_report(&result);
        assert!(
            report.contains(".akar/ already present"),
            "header must say .akar/ already present: {}",
            report
        );
    }

    #[test]
    fn format_bootstrap_report_shows_warnings_when_present() {
        let result = BootstrapResult {
            created: vec![],
            skipped: vec![],
            warnings: vec!["templates directory not found".to_string()],
            akar_dir_created: true,
            global_dir_created: false,
        };

        let report = format_bootstrap_report(&result);
        assert!(report.contains("warnings:"), "should show warnings section");
        assert!(
            report.contains("templates directory not found"),
            "should show warning text"
        );
    }
}
