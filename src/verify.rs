//! Phase 9 — Verification Recipe and Test Intelligence.
//!
//! Detects the project's build system, runs verification commands, classifies
//! failures, and formats results in the AKAR done-format.

use std::path::Path;
use std::process::Command;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single verification command to run.
#[derive(Debug, Clone)]
pub struct VerifyCommand {
    /// Human-readable label, e.g. `"cargo test"`.
    pub name: String,
    /// Executable, e.g. `"cargo"`.
    pub command: String,
    /// Arguments, e.g. `["test"]`.
    pub args: Vec<String>,
    /// If `true`, a failure here is blocking.
    pub required: bool,
}

/// A full verification recipe for a project.
#[derive(Debug, Clone)]
pub struct VerifyRecipe {
    /// Commands to execute.
    pub commands: Vec<VerifyCommand>,
    /// Things a human must verify manually.
    pub manual_checks: Vec<String>,
    /// Things that are explicitly out of scope for automated verification.
    pub not_verified: Vec<String>,
}

/// Classification of why a test/build step failed.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TestFailureClass {
    /// The production code is wrong.
    CodeWrong,
    /// The test itself is out of date.
    TestStale,
    /// Test scaffolding / fixtures are broken.
    TestSetupWrong,
    /// The local environment is misconfigured.
    EnvironmentIssue,
    /// The test is non-deterministic.
    FlakyTest,
    /// Behaviour exists but no test covers it.
    CoverageGap,
}

impl std::fmt::Display for TestFailureClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestFailureClass::CodeWrong => write!(f, "CodeWrong"),
            TestFailureClass::TestStale => write!(f, "TestStale"),
            TestFailureClass::TestSetupWrong => write!(f, "TestSetupWrong"),
            TestFailureClass::EnvironmentIssue => write!(f, "EnvironmentIssue"),
            TestFailureClass::FlakyTest => write!(f, "FlakyTest"),
            TestFailureClass::CoverageGap => write!(f, "CoverageGap"),
        }
    }
}

/// The outcome of running a single `VerifyCommand`.
#[derive(Debug, Clone)]
pub struct VerifyResult {
    /// The human-readable command name (mirrors `VerifyCommand::name`).
    pub command: String,
    /// Whether the command exited successfully.
    pub passed: bool,
    /// Combined stdout + stderr output.
    pub output: String,
    /// Populated only when `passed == false`.
    pub failure_class: Option<TestFailureClass>,
}

// ---------------------------------------------------------------------------
// Recipe detection
// ---------------------------------------------------------------------------

/// Inspect `project_root` and build an appropriate `VerifyRecipe`.
///
/// Detection order:
/// 1. `Cargo.toml` → Rust / cargo project
/// 2. `package.json` → Node / npm project
/// 3. Neither → manual-only recipe
pub fn detect_recipe(project_root: &Path) -> VerifyRecipe {
    let mut commands = Vec::new();

    if project_root.join("Cargo.toml").exists() {
        commands.push(VerifyCommand {
            name: "cargo build".to_string(),
            command: "cargo".to_string(),
            args: vec!["build".to_string()],
            required: true,
        });
        commands.push(VerifyCommand {
            name: "cargo test".to_string(),
            command: "cargo".to_string(),
            args: vec!["test".to_string()],
            required: true,
        });
    } else if project_root.join("package.json").exists() {
        commands.push(VerifyCommand {
            name: "npm run build".to_string(),
            command: "npm".to_string(),
            args: vec!["run".to_string(), "build".to_string()],
            required: false,
        });
        commands.push(VerifyCommand {
            name: "npm test".to_string(),
            command: "npm".to_string(),
            args: vec!["test".to_string()],
            required: false,
        });
    }

    let manual_checks = vec![
        "check changed files match task scope".to_string(),
        "no secrets in output".to_string(),
    ];

    let not_verified = vec![
        "browser click-through".to_string(),
        "production deployment".to_string(),
    ];

    let mut recipe = VerifyRecipe {
        commands,
        manual_checks,
        not_verified,
    };

    if recipe.commands.is_empty() {
        recipe
            .manual_checks
            .insert(0, "no build system detected".to_string());
    }

    recipe
}

// ---------------------------------------------------------------------------
// Failure classification
// ---------------------------------------------------------------------------

/// Classify a failure based on the combined command output.
#[allow(dead_code)]
pub fn classify_failure(output: &str) -> TestFailureClass {
    let lower = output.to_lowercase();

    // Environment / missing file indicators take priority over assertion messages
    // because "No such file" can appear in compiler errors alongside other text.
    if lower.contains("environment")
        || lower.contains("enoent")
        || lower.contains("no such file")
    {
        return TestFailureClass::EnvironmentIssue;
    }

    if lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("flaky")
        || lower.contains("intermittent")
    {
        return TestFailureClass::FlakyTest;
    }

    if lower.contains("cannot find")
        || lower.contains("undefined")
        || lower.contains("not found")
        || lower.contains("expected")
        || lower.contains("assertion")
        || lower.contains("assert_eq")
    {
        return TestFailureClass::CodeWrong;
    }

    TestFailureClass::CodeWrong
}

// ---------------------------------------------------------------------------
// Recipe runner
// ---------------------------------------------------------------------------

/// Run every command in `recipe` with `project_root` as the working directory.
///
/// Stdout and stderr are captured and merged.  The exit code determines
/// `passed`.  Failures are classified with `classify_failure`.
pub fn run_recipe(recipe: &VerifyRecipe, project_root: &Path) -> Vec<VerifyResult> {
    let mut results = Vec::new();

    for cmd in &recipe.commands {
        let output = Command::new(&cmd.command)
            .args(&cmd.args)
            .current_dir(project_root)
            // Merge stderr into stdout so callers see a single stream.
            .output();

        let result = match output {
            Ok(out) => {
                let combined = {
                    let mut s = String::new();
                    s.push_str(&String::from_utf8_lossy(&out.stdout));
                    s.push_str(&String::from_utf8_lossy(&out.stderr));
                    s
                };
                let passed = out.status.success();
                let failure_class = if passed {
                    None
                } else {
                    Some(classify_failure(&combined))
                };
                VerifyResult {
                    command: cmd.name.clone(),
                    passed,
                    output: combined,
                    failure_class,
                }
            }
            Err(e) => {
                // Command could not be spawned at all (e.g. binary not found).
                let output = format!("failed to spawn '{}': {}", cmd.command, e);
                VerifyResult {
                    command: cmd.name.clone(),
                    passed: false,
                    output: output.clone(),
                    failure_class: Some(classify_failure(&output)),
                }
            }
        };

        results.push(result);
    }

    results
}

// ---------------------------------------------------------------------------
// Output formatting
// ---------------------------------------------------------------------------

/// Format `results` plus the recipe's `not_verified` list in the AKAR done-format.
pub fn format_results(results: &[VerifyResult], recipe: &VerifyRecipe) -> String {
    let mut out = String::new();

    out.push_str("Verified:\n");
    if results.is_empty() {
        out.push_str("  (no automated checks)\n");
    } else {
        for r in results {
            if r.passed {
                out.push_str(&format!("  - {}: PASS\n", r.command));
            } else {
                let class = r
                    .failure_class
                    .as_ref()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                out.push_str(&format!("  - {}: FAIL ({})\n", r.command, class));
            }
        }
    }

    if !recipe.not_verified.is_empty() {
        out.push_str("Not verified:\n");
        for item in &recipe.not_verified {
            out.push_str(&format!("  - {}\n", item));
        }
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

    /// detect_recipe finds Cargo.toml in the project root and returns cargo commands.
    #[test]
    fn test_detect_recipe_finds_cargo_toml() {
        // The akar project itself has a Cargo.toml at the workspace root.
        // Cargo sets CARGO_MANIFEST_DIR to that directory during test runs.
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let recipe = detect_recipe(&root);

        let names: Vec<&str> = recipe.commands.iter().map(|c| c.name.as_str()).collect();
        assert!(
            names.contains(&"cargo build"),
            "expected 'cargo build' in recipe, got: {:?}",
            names
        );
        assert!(
            names.contains(&"cargo test"),
            "expected 'cargo test' in recipe, got: {:?}",
            names
        );

        // Both cargo commands must be marked required.
        for cmd in &recipe.commands {
            assert!(cmd.required, "'{}' should be required", cmd.name);
        }
    }

    /// detect_recipe always includes the standard manual checks.
    #[test]
    fn test_detect_recipe_manual_checks_always_present() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let recipe = detect_recipe(&root);

        assert!(
            recipe
                .manual_checks
                .iter()
                .any(|c| c.contains("check changed files")),
            "missing 'check changed files' manual check"
        );
        assert!(
            recipe
                .manual_checks
                .iter()
                .any(|c| c.contains("no secrets")),
            "missing 'no secrets' manual check"
        );
    }

    /// classify_failure on an assertion message → CodeWrong.
    #[test]
    fn test_classify_failure_assertion_is_code_wrong() {
        assert_eq!(
            classify_failure("expected 1 got 2"),
            TestFailureClass::CodeWrong
        );
    }

    /// classify_failure on a missing file message → EnvironmentIssue.
    #[test]
    fn test_classify_failure_no_such_file_is_environment_issue() {
        assert_eq!(
            classify_failure("No such file or directory"),
            TestFailureClass::EnvironmentIssue
        );
    }

    /// classify_failure on a timeout message → FlakyTest.
    #[test]
    fn test_classify_failure_timeout_is_flaky() {
        assert_eq!(
            classify_failure("test timed out after 30s"),
            TestFailureClass::FlakyTest
        );
    }

    /// classify_failure on an unknown message defaults to CodeWrong.
    #[test]
    fn test_classify_failure_default_is_code_wrong() {
        assert_eq!(
            classify_failure("something went wrong"),
            TestFailureClass::CodeWrong
        );
    }

    /// format_results produces the correct output shape.
    #[test]
    fn test_format_results_shape() {
        let results = vec![
            VerifyResult {
                command: "cargo build".to_string(),
                passed: true,
                output: String::new(),
                failure_class: None,
            },
            VerifyResult {
                command: "cargo test".to_string(),
                passed: false,
                output: "expected 1 got 2".to_string(),
                failure_class: Some(TestFailureClass::CodeWrong),
            },
        ];

        let recipe = VerifyRecipe {
            commands: vec![],
            manual_checks: vec![],
            not_verified: vec![
                "browser click-through".to_string(),
                "production deployment".to_string(),
            ],
        };

        let output = format_results(&results, &recipe);

        assert!(output.contains("Verified:"), "missing 'Verified:' header");
        assert!(
            output.contains("cargo build: PASS"),
            "missing pass line for cargo build"
        );
        assert!(
            output.contains("cargo test: FAIL (CodeWrong)"),
            "missing fail line for cargo test"
        );
        assert!(
            output.contains("Not verified:"),
            "missing 'Not verified:' header"
        );
        assert!(
            output.contains("browser click-through"),
            "missing browser click-through item"
        );
    }
}
