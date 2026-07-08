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
///   --claude     Record that the user wants Claude Code integration.

use crate::bootstrap;
use crate::config;
use crate::doctor;

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

pub fn run_init(skip: bool, claude_integration: bool) -> InitResult {
    let mut result = InitResult::default();
    result.shell = detect_shell();
    result.claude_integration = claude_integration;

    let cfg = config::Config::discover();

    // If .akar/ already exists and doctor is clean, nothing to do.
    if cfg.akar_dir.exists() {
        let issues = doctor::run_checks(&cfg);
        if issues.is_empty() && !skip {
            result.bootstrapped = false;
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

    result
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
        out.push_str("  copy .claude/commands/akar-*.md to your project's .claude/commands/\n");
        out.push_str("  then use /akar-preflight and /akar-doctor as slash commands\n");
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
        };
        let out = format_init_report(&result);
        assert!(out.contains("claude integration"), "got: {}", out);
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
        };
        let out = format_init_report(&result);
        assert!(out.contains("issues remain"), "got: {}", out);
        assert!(out.contains("akar doctor --fix"), "got: {}", out);
    }
}
