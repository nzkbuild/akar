/// `akar init` — first-run onboarding command.
///
/// Simpler than `akar bootstrap` in presentation:
/// - Detects the current shell (PowerShell vs bash).
/// - Explains what AKAR will create.
/// - Runs bootstrap + doctor in sequence.
/// - Prints a short "what to do next" guide.
///
/// Flags:
///   --skip       Skip the interactive prompt, just run bootstrap + doctor.
///   --claude     Apply the AKAR session guidance snippet to CLAUDE.md.
///   --yes        Skip confirmation prompts (non-interactive mode).
use crate::bootstrap;
use crate::claude_snippet;
use crate::config;
use crate::doctor;
use crate::path_health;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct InitResult {
    /// Shell detected at runtime.
    pub shell: ShellKind,
    /// Whether bootstrap was run.
    pub bootstrapped: bool,
    /// Created files from bootstrap.
    pub created: Vec<String>,
    /// Skipped files from bootstrap.
    pub skipped: Vec<String>,
    /// Doctor issues remaining after bootstrap.
    pub doctor_issues: Vec<String>,
    /// Whether user opted into Claude integration hint.
    pub claude_integration: bool,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
    /// Result of CLAUDE.md snippet management (only when --claude).
    pub claude_snippet: Option<crate::claude_snippet::ClaudeSnippetResult>,
    /// PATH health assessment.
    pub path_health: Option<crate::path_health::PathHealth>,
    /// PATH repair result (if repair was performed).
    pub path_repair: Option<crate::path_health::PathRepairResult>,
}

#[derive(Debug, Default, PartialEq)]
pub enum ShellKind {
    PowerShell,
    Bash,
    #[default]
    Unknown,
}

// ---------------------------------------------------------------------------
// Core logic
// ---------------------------------------------------------------------------

pub fn run_init(skip: bool, claude_integration: bool, yes: bool) -> InitResult {
    let mut result = InitResult::default();
    result.shell = detect_shell();
    result.claude_integration = claude_integration;

    let cfg = config::Config::discover();

    // If .akar/ already exists and doctor is clean, nothing to do.
    if cfg.akar_dir.exists() {
        let issues = doctor::run_checks(&cfg);
        if issues.is_empty() && !skip {
            result.bootstrapped = false;
            // Still handle --claude and PATH health even when .akar/ exists.
            run_claude_snippet(&cfg, &mut result, claude_integration, yes);
            run_path_health_check(&mut result, yes);
            return result;
        }
    }

    // Run bootstrap.
    let br = bootstrap::run_bootstrap(&cfg);
    result.bootstrapped = true;
    result.created = br.created;
    result.skipped = br.skipped;
    result.warnings.extend(br.warnings);

    // Run doctor and collect remaining issues.
    let issues = doctor::run_checks(&cfg);
    result.doctor_issues = issues.iter().map(|i| i.message.clone()).collect();

    // CLAUDE.md snippet integration.
    run_claude_snippet(&cfg, &mut result, claude_integration, yes);

    // PATH health.
    run_path_health_check(&mut result, yes);

    result
}

/// Handle CLAUDE.md snippet management when --claude is set.
fn run_claude_snippet(
    cfg: &config::Config,
    result: &mut InitResult,
    claude_integration: bool,
    yes: bool,
) {
    if !claude_integration {
        return;
    }

    println!();
    println!("claude.md snippet:");
    let confirmed = yes || confirm_action("Apply AKAR session guidance snippet to CLAUDE.md?");
    let snippet_result = claude_snippet::apply_snippet(&cfg.project_root, confirmed);
    result.claude_snippet = Some(snippet_result);
}

/// Assess PATH health and optionally repair.
fn run_path_health_check(result: &mut InitResult, yes: bool) {
    let ph = path_health::check_path_health();
    let needs_repair = matches!(
        ph.status,
        path_health::PathHealthStatus::Missing | path_health::PathHealthStatus::Mismatch
    );

    if needs_repair {
        println!();
        match ph.status {
            path_health::PathHealthStatus::Missing => {
                println!("akar not found on PATH — hooks may fail open.");
            }
            path_health::PathHealthStatus::Mismatch => {
                println!(
                    "PATH akar version mismatch: v{} (running) vs v{} (PATH at {})",
                    ph.running_version,
                    ph.path_version.as_deref().unwrap_or("unknown"),
                    ph.path_akar
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                );
            }
            _ => {}
        }

        let confirmed = yes || confirm_action("Copy running akar to PATH to fix this?");
        let repair = path_health::repair_path(&ph, confirmed);
        result.path_repair = Some(repair);
    }

    result.path_health = Some(ph);
}

/// Prompt the user for confirmation. Returns true if they type "INSTALL".
fn confirm_action(prompt: &str) -> bool {
    println!("{}", prompt);
    println!("  Type INSTALL to confirm, or anything else to cancel:");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
    input.trim() == "INSTALL"
}

/// Detect the shell from environment variables.
pub fn detect_shell() -> ShellKind {
    // PSModulePath is only set by PowerShell.
    if std::env::var("PSModulePath").is_ok() {
        return ShellKind::PowerShell;
    }
    // BASH_VERSION or SHELL pointing to bash.
    if std::env::var("BASH_VERSION").is_ok() {
        return ShellKind::Bash;
    }
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("bash") {
            return ShellKind::Bash;
        }
        if shell.contains("zsh") || shell.contains("sh") {
            return ShellKind::Bash; // treat POSIX shells as bash-compatible
        }
    }
    ShellKind::Unknown
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

pub fn format_init_report(result: &InitResult) -> String {
    let mut out = String::new();

    // Already initialised, nothing to do.
    if !result.bootstrapped && result.doctor_issues.is_empty() {
        out.push_str("init: already set up\n");
        out.push_str("  .akar/ exists and doctor is clean\n");
        out.push_str("  run 'akar status' to check runtime health\n");
        return out;
    }

    let shell_label = match result.shell {
        ShellKind::PowerShell => "PowerShell",
        ShellKind::Bash => "bash",
        ShellKind::Unknown => "unknown shell",
    };

    out.push_str(&format!("init: shell={}\n", shell_label));
    out.push('\n');

    if result.bootstrapped {
        out.push_str(&format!(
            "bootstrap: {} created, {} skipped\n",
            result.created.len(),
            result.skipped.len()
        ));
        for name in &result.created {
            out.push_str(&format!("  + {}\n", name));
        }
    }

    if !result.warnings.is_empty() {
        out.push('\n');
        out.push_str("warnings:\n");
        for w in &result.warnings {
            out.push_str(&format!("  - {}\n", w));
        }
    }

    if result.doctor_issues.is_empty() {
        out.push('\n');
        out.push_str("doctor: OK\n");
    } else {
        out.push('\n');
        out.push_str("doctor: issues remain\n");
        for issue in &result.doctor_issues {
            out.push_str(&format!("  - {}\n", issue));
        }
        out.push_str("  hint: run 'akar doctor --fix' to resolve\n");
    }

    out.push('\n');
    out.push_str(".akar/ notice:\n");
    out.push_str("  .akar/ contains local AKAR runtime state. Inspect 'git status'.\n");
    out.push_str("  Intentionally add .akar/ to .gitignore or commit only files you\n");
    out.push_str("  want tracked. AKAR will not decide for you.\n");
    out.push_str("  Do not use destructive cleanup blindly.\n");
    out.push('\n');
    out.push_str("next steps:\n");
    out.push_str("  akar status              — confirm runtime health\n");
    out.push_str("  akar preflight \"<task>\"  — review strategy before acting\n");
    out.push_str("  akar run \"<task>\"        — full workflow in one command\n");

    if result.claude_integration {
        out.push('\n');
        out.push_str("claude integration:\n");
        if let Some(ref snippet) = result.claude_snippet {
            out.push_str(&format!("  action: {}\n", snippet.action));
            out.push_str(&format!("  file: {}\n", snippet.path.display()));
            out.push_str(&format!("  detail: {}\n", snippet.detail));
        } else {
            out.push_str("  copy .claude/commands/akar-*.md to your project's .claude/commands/\n");
            out.push_str("  then use /akar-preflight and /akar-doctor as slash commands\n");
        }
    }

    if let Some(ref ph) = result.path_health {
        out.push('\n');
        out.push_str("path health:\n");
        out.push_str(&format!(
            "  running: {} (v{})\n",
            ph.running_path.display(),
            ph.running_version
        ));
        match ph.status {
            path_health::PathHealthStatus::Healthy => {
                out.push_str("  path akar: OK\n");
            }
            path_health::PathHealthStatus::Missing => {
                out.push_str("  path akar: MISSING — hooks may fail open\n");
            }
            path_health::PathHealthStatus::Mismatch => {
                out.push_str(&format!(
                    "  path akar: MISMATCH — PATH has v{} at {}\n",
                    ph.path_version.as_deref().unwrap_or("unknown"),
                    ph.path_akar
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                ));
            }
            path_health::PathHealthStatus::UnknownVersion => {
                out.push_str("  path akar: found but version unknown\n");
            }
        }
        if let Some(ref repair) = result.path_repair {
            out.push_str(&format!(
                "  repair: {} — {}\n",
                repair.action, repair.detail
            ));
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

    #[test]
    fn detect_shell_returns_a_value() {
        // Just verify it doesn't panic and returns a valid enum.
        let shell = detect_shell();
        let _ = shell; // ShellKind is Debug
    }

    #[test]
    fn format_already_setup() {
        let result = InitResult {
            shell: ShellKind::PowerShell,
            bootstrapped: false,
            created: vec![],
            skipped: vec![],
            doctor_issues: vec![],
            claude_integration: false,
            warnings: vec![],
            claude_snippet: None,
            path_health: None,
            path_repair: None,
        };
        let out = format_init_report(&result);
        assert!(out.contains("already set up"), "got: {}", out);
    }

    #[test]
    fn format_fresh_init() {
        let result = InitResult {
            shell: ShellKind::PowerShell,
            bootstrapped: true,
            created: vec!["PROJECT_DNA.md".to_string(), "STATE.md".to_string()],
            skipped: vec![],
            doctor_issues: vec![],
            claude_integration: false,
            warnings: vec![],
            claude_snippet: None,
            path_health: None,
            path_repair: None,
        };
        let out = format_init_report(&result);
        assert!(out.contains("bootstrap: 2 created"), "got: {}", out);
        assert!(out.contains("doctor: OK"), "got: {}", out);
        assert!(out.contains(".akar/ notice"), "got: {}", out);
        assert!(out.contains("next steps"), "got: {}", out);
    }

    #[test]
    fn format_with_claude_integration() {
        let result = InitResult {
            shell: ShellKind::Bash,
            bootstrapped: true,
            created: vec!["STATE.md".to_string()],
            skipped: vec![],
            doctor_issues: vec![],
            claude_integration: true,
            warnings: vec![],
            claude_snippet: Some(crate::claude_snippet::ClaudeSnippetResult {
                path: std::path::PathBuf::from("/tmp/test/CLAUDE.md"),
                action: "created".to_string(),
                prior_state: "absent".to_string(),
                detail: "created CLAUDE.md with AKAR session guidance snippet".to_string(),
            }),
            path_health: None,
            path_repair: None,
        };
        let out = format_init_report(&result);
        assert!(out.contains("claude integration"), "got: {}", out);
        assert!(out.contains("action: created"), "got: {}", out);
    }

    #[test]
    fn format_with_doctor_issues() {
        let result = InitResult {
            shell: ShellKind::Unknown,
            bootstrapped: true,
            created: vec![],
            skipped: vec![],
            doctor_issues: vec!["missing .akar/STATE.md".to_string()],
            claude_integration: false,
            warnings: vec![],
            claude_snippet: None,
            path_health: None,
            path_repair: None,
        };
        let out = format_init_report(&result);
        assert!(out.contains("issues remain"), "got: {}", out);
        assert!(out.contains("akar doctor --fix"), "got: {}", out);
    }
}
