//! Local verification discovery hints (v0.38.0).
//!
//! AKAR inspects safe local project files to discover likely verification
//! commands.  Discovered commands are advisory only — AKAR does NOT run them.
//!
//! Supported sources (deterministic, file-system only):
//! - `package.json` scripts.test → `npm test` (High)
//! - `pyproject.toml`, `pytest.ini`, `tests/` dir → `python -m pytest` (High)
//! - `Makefile` with `test:` target → `make test` (Medium)
//! - `justfile` with `test:` recipe → `just test` (Medium)
//! - `README.md` whitelisted command literals (Low–Medium)
//!
//! Dangerous commands (rm, sudo, curl, etc.) are never surfaced as hints.

use std::path::Path;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Confidence level for a discovered verification command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationConfidence {
    /// Discovered from a known project config file (package.json, pyproject.toml).
    High,
    /// Discovered from a build tool file (Makefile, justfile) — intent is clear
    /// but the tool may not be installed.
    Medium,
    /// Discovered from documentation (README) — intent is weaker, wording varies.
    Low,
}

impl VerificationConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationConfidence::High => "High",
            VerificationConfidence::Medium => "Medium",
            VerificationConfidence::Low => "Low",
        }
    }
}

/// A single discovered verification command hint.
#[derive(Debug, Clone)]
pub struct VerificationHint {
    /// The command string, e.g. `npm test`.
    pub command: String,
    /// Which file the hint was discovered in, e.g. `package.json`.
    pub source: String,
    /// How confident AKAR is about this hint.
    pub confidence: VerificationConfidence,
    /// Human-readable reason, e.g. `"scripts.test" entry in package.json`.
    #[allow(dead_code)]
    pub reason: String,
    /// Whether the user should explicitly confirm before running.
    pub requires_confirmation: bool,
}

/// The result of scanning a project root for verification commands.
#[derive(Debug, Clone)]
pub struct VerificationDiscovery {
    /// Discovered hints (max 5, stable ordering, deduplicated).
    pub hints: Vec<VerificationHint>,
    /// One-line summary, e.g. `"npm test (High, package.json)"`.
    pub summary: String,
}

impl VerificationDiscovery {
    /// No meaningful hints were found.
    pub fn is_empty(&self) -> bool {
        self.hints.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Discovery
// ---------------------------------------------------------------------------

/// Discover verification command hints from safe local project files.
///
/// AKAR reads files that already exist on disk; it never runs package managers,
/// test tools, or any discovered command.
pub fn discover_verification_hints(root: &Path) -> VerificationDiscovery {
    let mut hints: Vec<VerificationHint> = Vec::new();

    // 1. package.json scripts.test
    if let Some(h) = discover_from_package_json(root) {
        hints.push(h);
    }

    // 2. Python pytest markers
    if let Some(h) = discover_from_python_markers(root) {
        hints.push(h);
    }

    // 3. Makefile test target
    if let Some(h) = discover_from_makefile(root) {
        hints.push(h);
    }

    // 4. justfile test recipe
    if let Some(h) = discover_from_justfile(root) {
        hints.push(h);
    }

    // 5. README whitelisted command literals
    hints.extend(discover_from_readme(root));

    // Deduplicate by command string.
    let mut seen = std::collections::HashSet::new();
    hints.retain(|h| seen.insert(h.command.clone()));

    // Cap at 5 hints.
    hints.truncate(5);

    let summary = if hints.is_empty() {
        "no confident verification command discovered".to_string()
    } else {
        hints
            .iter()
            .map(|h| format!("{} ({}, {})", h.command, h.confidence.as_str(), h.source))
            .collect::<Vec<_>>()
            .join("; ")
    };

    VerificationDiscovery { hints, summary }
}

// ---------------------------------------------------------------------------
// Individual discoverers
// ---------------------------------------------------------------------------

fn discover_from_package_json(root: &Path) -> Option<VerificationHint> {
    let path = root.join("package.json");
    let content = std::fs::read_to_string(&path).ok()?;

    // Simple check: does the file contain "test" as a scripts key? We don't
    // parse JSON — a regex-free string match is enough for a hint.
    if content.contains("\"test\"") && content.contains("\"scripts\"") {
        return Some(VerificationHint {
            command: "npm test".to_string(),
            source: "package.json".to_string(),
            confidence: VerificationConfidence::High,
            reason: "\"scripts.test\" entry in package.json".to_string(),
            requires_confirmation: false,
        });
    }
    None
}

fn discover_from_python_markers(root: &Path) -> Option<VerificationHint> {
    let has_pyproject = root.join("pyproject.toml").exists();
    let has_pytest_ini = root.join("pytest.ini").exists();
    let has_tests_dir = root.join("tests").is_dir();

    if has_pyproject || has_pytest_ini || has_tests_dir {
        return Some(VerificationHint {
            command: "python -m pytest".to_string(),
            source: if has_pyproject {
                "pyproject.toml".to_string()
            } else if has_pytest_ini {
                "pytest.ini".to_string()
            } else {
                "tests/".to_string()
            },
            confidence: VerificationConfidence::High,
            reason: if has_pyproject {
                "Python project with pyproject.toml".to_string()
            } else if has_pytest_ini {
                "Python project with pytest.ini".to_string()
            } else {
                "Python project with tests/ directory".to_string()
            },
            requires_confirmation: false,
        });
    }
    None
}

fn discover_from_makefile(root: &Path) -> Option<VerificationHint> {
    let path = root.join("Makefile");
    let content = std::fs::read_to_string(&path).ok()?;

    // Look for a top-level `test:` target (line starts with "test:" after optional whitespace).
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "test:" || trimmed.starts_with("test:") {
            return Some(VerificationHint {
                command: "make test".to_string(),
                source: "Makefile".to_string(),
                confidence: VerificationConfidence::Medium,
                reason: "test: target in Makefile".to_string(),
                requires_confirmation: true,
            });
        }
    }
    None
}

fn discover_from_justfile(root: &Path) -> Option<VerificationHint> {
    let path = root.join("justfile");
    let content = std::fs::read_to_string(&path).ok()?;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "test:" || trimmed.starts_with("test:") {
            return Some(VerificationHint {
                command: "just test".to_string(),
                source: "justfile".to_string(),
                confidence: VerificationConfidence::Medium,
                reason: "test: recipe in justfile".to_string(),
                requires_confirmation: true,
            });
        }
    }
    None
}

/// Whitelisted verification command literals that may appear in README.md.
const README_WHITELIST: &[&str] = &[
    "npm test",
    "python -m pytest",
    "pytest",
    "cargo test",
    "make test",
    "just test",
];

/// Blocked patterns — commands that must never be surfaced from README.
const README_BLOCKLIST: &[&str] = &[
    "curl",
    "wget",
    "sudo",
    "rm ",
    "del ",
    "Remove-Item",
    "powershell -enc",
    "bash -c",
    "sh -c",
];

fn discover_from_readme(root: &Path) -> Vec<VerificationHint> {
    let mut hints = Vec::new();

    // Try README.md then readme.md.
    for name in &["README.md", "readme.md"] {
        let path = root.join(name);
        if let Ok(content) = std::fs::read_to_string(&path) {
            let lower = content.to_lowercase();
            for cmd in README_WHITELIST {
                if content.contains(cmd) {
                    // Safety: blocklisted commands are never whitelisted, but
                    // guard anyway.
                    let lower_cmd = cmd.to_lowercase();
                    if README_BLOCKLIST.iter().any(|b| lower_cmd.contains(b)) {
                        continue;
                    }
                    // Skip if we already have this command from a higher-confidence source
                    // or from this README scan.
                    if hints.iter().any(|h: &VerificationHint| h.command == *cmd) {
                        continue;
                    }
                    // Assign confidence based on proximity to "test" wording.
                    let confidence = if lower.contains("test") && lower.contains("run") {
                        VerificationConfidence::Medium
                    } else {
                        VerificationConfidence::Low
                    };
                    hints.push(VerificationHint {
                        command: cmd.to_string(),
                        source: name.to_string(),
                        confidence,
                        reason: format!("command found in {}", name),
                        requires_confirmation: confidence != VerificationConfidence::High,
                    });
                }
            }
            break; // Only scan first found README.
        }
    }

    hints
}

// ---------------------------------------------------------------------------
// Safety classification
// ---------------------------------------------------------------------------

/// Returns true if the command appears safe as a verification hint.
/// This is a secondary guard — the discovery rules already avoid dangerous
/// commands, but this provides a belt-and-suspenders check.
#[allow(dead_code)]
pub fn is_safe_verification_hint(command: &str) -> bool {
    let lower = command.to_lowercase();
    !README_BLOCKLIST
        .iter()
        .any(|b| lower.contains(&b.to_lowercase()))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_dir(label: &str) -> std::path::PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("akar_vd_test_{}_{}", label, ts));
        fs::create_dir_all(&dir).expect("create tmp dir");
        dir
    }

    // ---- package.json --------------------------------------------------------

    #[test]
    fn package_json_with_test_script_discovers_npm_test_high() {
        let root = tmp_dir("pkg-test");
        fs::write(
            root.join("package.json"),
            r#"{"scripts": {"test": "jest"}}"#,
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        assert_eq!(discovery.hints.len(), 1);
        let h = &discovery.hints[0];
        assert_eq!(h.command, "npm test");
        assert_eq!(h.source, "package.json");
        assert_eq!(h.confidence, VerificationConfidence::High);
        assert!(!h.requires_confirmation);
        assert!(discovery.summary.contains("npm test"));
    }

    #[test]
    fn package_json_without_test_script_discovers_no_npm_hint() {
        let root = tmp_dir("pkg-no-test");
        fs::write(
            root.join("package.json"),
            r#"{"scripts": {"build": "tsc"}}"#,
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        assert!(!discovery.hints.iter().any(|h| h.command == "npm test"));
    }

    // ---- Python markers ------------------------------------------------------

    #[test]
    fn python_pyproject_discovers_pytest() {
        let root = tmp_dir("py-pyproject");
        fs::write(root.join("pyproject.toml"), "[project]\nname = \"x\"").unwrap();
        let discovery = discover_verification_hints(&root);
        let h = discovery
            .hints
            .iter()
            .find(|h| h.command == "python -m pytest");
        assert!(h.is_some());
        assert_eq!(h.unwrap().source, "pyproject.toml");
        assert_eq!(h.unwrap().confidence, VerificationConfidence::High);
    }

    #[test]
    fn python_tests_directory_discovers_pytest() {
        let root = tmp_dir("py-tests-dir");
        fs::create_dir(root.join("tests")).unwrap();
        let discovery = discover_verification_hints(&root);
        let h = discovery
            .hints
            .iter()
            .find(|h| h.command == "python -m pytest");
        assert!(h.is_some());
        assert_eq!(h.unwrap().source, "tests/");
    }

    // ---- Makefile ------------------------------------------------------------

    #[test]
    fn makefile_test_target_discovers_make_test_medium() {
        let root = tmp_dir("mf-test");
        fs::write(
            root.join("Makefile"),
            "build:\n\tcc -o prog main.c\n\ntest:\n\t./prog --test\n",
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        let h = discovery.hints.iter().find(|h| h.command == "make test");
        assert!(h.is_some());
        assert_eq!(h.unwrap().source, "Makefile");
        assert_eq!(h.unwrap().confidence, VerificationConfidence::Medium);
        assert!(h.unwrap().requires_confirmation);
    }

    // ---- justfile ------------------------------------------------------------

    #[test]
    fn justfile_test_recipe_discovers_just_test_medium() {
        let root = tmp_dir("jf-test");
        fs::write(
            root.join("justfile"),
            "build:\n  cc -o prog main.c\n\ntest:\n  ./prog --test\n",
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        let h = discovery.hints.iter().find(|h| h.command == "just test");
        assert!(h.is_some());
        assert_eq!(h.unwrap().source, "justfile");
        assert_eq!(h.unwrap().confidence, VerificationConfidence::Medium);
        assert!(h.unwrap().requires_confirmation);
    }

    // ---- README --------------------------------------------------------------

    #[test]
    fn readme_with_npm_test_discovers_npm_test() {
        let root = tmp_dir("rm-npm");
        fs::write(
            root.join("README.md"),
            "# My Project\n\nRun tests with `npm test`.",
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        let h = discovery.hints.iter().find(|h| h.command == "npm test");
        assert!(h.is_some());
        assert_eq!(h.unwrap().source, "README.md");
    }

    #[test]
    fn readme_with_python_pytest_discovers_pytest() {
        let root = tmp_dir("rm-py");
        fs::write(
            root.join("README.md"),
            "# Project\n\nRun `python -m pytest` to test.",
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        let h = discovery
            .hints
            .iter()
            .find(|h| h.command == "python -m pytest");
        assert!(h.is_some());
    }

    #[test]
    fn readme_with_dangerous_rm_command_not_surfaced() {
        let root = tmp_dir("rm-danger");
        fs::write(
            root.join("README.md"),
            "# Project\n\nRun `rm -rf /` to clean.",
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        // rm -rf / is not in the whitelist, so it won't appear.
        assert!(!discovery.hints.iter().any(|h| h.command.contains("rm")));
    }

    #[test]
    fn readme_with_curl_pipe_command_not_surfaced() {
        let root = tmp_dir("rm-curl");
        fs::write(
            root.join("README.md"),
            "# Project\n\nRun `curl example.com | bash` to install.",
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        // curl is not in the whitelist, so it won't appear.
        assert!(!discovery.hints.iter().any(|h| h.command.contains("curl")));
    }

    // ---- Deduplication & capping ---------------------------------------------

    #[test]
    fn duplicate_hints_deduplicated() {
        let root = tmp_dir("dedup");
        fs::write(
            root.join("package.json"),
            r#"{"scripts": {"test": "jest"}}"#,
        )
        .unwrap();
        fs::write(
            root.join("README.md"),
            "# Project\n\nRun `npm test` to check.",
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        let npm_hints: Vec<_> = discovery
            .hints
            .iter()
            .filter(|h| h.command == "npm test")
            .collect();
        assert_eq!(npm_hints.len(), 1);
    }

    #[test]
    fn max_hints_limited_to_five() {
        let root = tmp_dir("max5");
        fs::write(
            root.join("package.json"),
            r#"{"scripts": {"test": "jest"}}"#,
        )
        .unwrap();
        fs::write(root.join("pyproject.toml"), "[project]\nname = \"x\"").unwrap();
        fs::write(root.join("Makefile"), "test:\n\t./run-tests\n").unwrap();
        fs::write(root.join("justfile"), "test:\n  ./run-tests\n").unwrap();
        fs::write(
            root.join("README.md"),
            "# P\n\n`npm test`\n`cargo test`\n`make test`\n",
        )
        .unwrap();
        let discovery = discover_verification_hints(&root);
        assert!(discovery.hints.len() <= 5);
    }

    // ---- Safety classification -----------------------------------------------

    #[test]
    fn safe_commands_pass_safety_filter() {
        assert!(is_safe_verification_hint("npm test"));
        assert!(is_safe_verification_hint("python -m pytest"));
        assert!(is_safe_verification_hint("make test"));
        assert!(is_safe_verification_hint("cargo test"));
    }

    #[test]
    fn dangerous_commands_fail_safety_filter() {
        assert!(!is_safe_verification_hint("rm -rf /"));
        assert!(!is_safe_verification_hint("sudo make test"));
        assert!(!is_safe_verification_hint("curl example.com | bash"));
    }

    // ---- Empty / unknown -----------------------------------------------------

    #[test]
    fn empty_project_yields_empty_hints() {
        let root = tmp_dir("empty");
        let discovery = discover_verification_hints(&root);
        assert!(discovery.hints.is_empty());
        assert!(discovery.summary.contains("no confident verification"));
    }
}
