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
///   --hooks      Set up AKAR Claude Code hooks in .claude/settings.local.json.
///   --yes        Skip confirmation prompts (non-interactive mode).
use std::path::{Path, PathBuf};

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
    /// Result of Claude Code hook setup (only when --hooks).
    pub hook_setup: Option<HookSetupResult>,
}

/// Result of setting up Claude Code hooks via `akar init --hooks`.
#[derive(Debug)]
pub struct HookSetupResult {
    pub action: String,
    pub path: PathBuf,
    pub detail: String,
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

pub fn run_init(skip: bool, claude_integration: bool, hooks: bool, yes: bool) -> InitResult {
    let mut result = InitResult::default();
    result.shell = detect_shell();
    result.claude_integration = claude_integration;

    let cfg = config::Config::discover();

    // If .akar/ already exists and doctor is clean, nothing to do.
    if cfg.akar_dir.exists() {
        let issues = doctor::run_checks(&cfg);
        if issues.is_empty() && !skip {
            result.bootstrapped = false;
            // Still handle --claude, --hooks, and PATH health even when .akar/ exists.
            run_claude_snippet(&cfg, &mut result, claude_integration, yes);
            run_hook_setup(&cfg, &mut result, hooks, yes);
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

    // Claude Code hook setup.
    run_hook_setup(&cfg, &mut result, hooks, yes);

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

/// Set up Claude Code UserPromptSubmit hook in .claude/settings.local.json.
fn run_hook_setup(cfg: &config::Config, result: &mut InitResult, hooks: bool, yes: bool) {
    if !hooks {
        return;
    }

    println!();
    println!("claude code hooks:");
    let confirmed = yes
        || confirm_action(
            "Set up AKAR Claude Code auto-context hook in .claude/settings.local.json?",
        );
    result.hook_setup = Some(setup_claude_hooks(&cfg.project_root, confirmed));
}

/// Create or update .claude/settings.local.json with the AKAR UserPromptSubmit hook.
///
/// - Creates `.claude/` directory if it doesn't exist.
/// - Reads existing settings if present.
/// - Preserves unrelated hooks, merges AKAR hook idempotently.
/// - Backs up before overwriting.
/// - Uses project-local `pwsh` command to invoke the hook handler.
fn setup_claude_hooks(project_root: &Path, confirmed: bool) -> HookSetupResult {
    let claude_dir = project_root.join(".claude");
    let settings_path = claude_dir.join("settings.local.json");

    if !confirmed {
        return HookSetupResult {
            action: "cancelled".to_string(),
            path: settings_path,
            detail: "hook setup skipped — confirmation not given".to_string(),
        };
    }

    // Create .claude/ directory if needed.
    if !claude_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&claude_dir) {
            return HookSetupResult {
                action: "failed".to_string(),
                path: settings_path,
                detail: format!("could not create .claude/ directory: {}", e),
            };
        }
    }

    let akar_marker = "akar hook user-prompt-submit";

    // Read existing file content.
    let existing_text = if settings_path.exists() {
        match std::fs::read_to_string(&settings_path) {
            Ok(s) => s,
            Err(e) => {
                return HookSetupResult {
                    action: "failed".to_string(),
                    path: settings_path,
                    detail: format!("could not read settings.local.json: {}", e),
                };
            }
        }
    } else {
        String::new()
    };

    // Check for existing AKAR hook — idempotent.
    if existing_text.contains(akar_marker) {
        return HookSetupResult {
            action: "unchanged".to_string(),
            path: settings_path,
            detail: "AKAR UserPromptSubmit hook already present".to_string(),
        };
    }

    // Backup existing file before overwriting.
    let pre_existing = !existing_text.trim().is_empty();
    if pre_existing {
        let _ = crate::backup::backup_file(&settings_path);
    }

    // Build the hook command.
    let hook_command = if cfg!(windows) {
        "pwsh -NoProfile -Command \\\"akar hook user-prompt-submit\\\""
    } else {
        "akar hook user-prompt-submit"
    };

    let akar_entry = format!(
        r#"{{
    "matcher": "",
    "hooks": [
      {{
        "type": "command",
        "command": "{hook_command}"
      }}
    ]
  }}"#,
    );

    // Build the merged file: preserve unrelated keys, add/merge AKAR hook.
    let merged = if pre_existing {
        let trimmed = existing_text.trim();
        // Trim trailing "}" and any trailing whitespace.
        let inner = trimmed
            .strip_prefix('{')
            .and_then(|s| s.strip_suffix('}'))
            .map(|s| s.trim())
            .unwrap_or("");

        if inner.is_empty() {
            // Empty object — just write our hook.
            format!(
                "{{\n  \"hooks\": {{\n    \"UserPromptSubmit\": [\n      {akar_entry}\n    ]\n  }}\n}}\n"
            )
        } else if inner.contains(r#""UserPromptSubmit""#) {
            // Existing "UserPromptSubmit" array — append our entry before its closing "]".
            // Use bracket counting from the "[" that opens the array value.
            let ups_key_pos = existing_text.rfind(r#""UserPromptSubmit""#).unwrap();
            let after_key = &existing_text[ups_key_pos..];
            // Find the opening "[" of the array value (after ": ").
            let array_start = after_key
                .find('[')
                .map(|p| ups_key_pos + p)
                .unwrap_or(existing_text.len());
            // Count brackets from array_start to find the matching "]".
            let mut depth = 0;
            let mut array_end = array_start;
            for (i, ch) in existing_text[array_start..].char_indices() {
                match ch {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            array_end = array_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let before = &existing_text[..array_end];
            let after = &existing_text[array_end..];
            format!("{before},\n      {akar_entry}\n    {after}")
        } else {
            // No existing UserPromptSubmit — add it before the final "}".
            format!(
                "{},\n  \"hooks\": {{\n    \"UserPromptSubmit\": [\n      {akar_entry}\n    ]\n  }}\n}}",
                inner.trim_end().trim_end_matches(',')
            )
        }
    } else {
        format!(
            "{{\n  \"hooks\": {{\n    \"UserPromptSubmit\": [\n      {akar_entry}\n    ]\n  }}\n}}\n"
        )
    };

    if let Err(e) = std::fs::write(&settings_path, &merged) {
        return HookSetupResult {
            action: "failed".to_string(),
            path: settings_path,
            detail: format!("could not write settings.local.json: {}", e),
        };
    }

    let action = if pre_existing { "merged" } else { "created" };

    HookSetupResult {
        action: action.to_string(),
        path: settings_path,
        detail: format!(
            "{} AKAR UserPromptSubmit hook in .claude/settings.local.json",
            action
        ),
    }
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

    if let Some(ref hook) = result.hook_setup {
        out.push('\n');
        out.push_str("claude code hooks:\n");
        out.push_str(&format!("  action: {}\n", hook.action));
        out.push_str(&format!("  file: {}\n", hook.path.display()));
        out.push_str(&format!("  detail: {}\n", hook.detail));
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
            hook_setup: None,
        };
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
            hook_setup: None,
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
            hook_setup: None,
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
            hook_setup: None,
        };
        let out = format_init_report(&result);
        assert!(out.contains("issues remain"), "got: {}", out);
        assert!(out.contains("akar doctor --fix"), "got: {}", out);
    }
}
