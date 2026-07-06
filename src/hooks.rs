//! Hook template management — check, install, and verify hook scripts.
//!
//! AKAR ships hook templates in `templates/hooks/`. The `akar hooks` command
//! prints instructions. `akar hooks --check` verifies templates exist and are
//! readable. `akar hooks --install` copies templates into `.akar/hooks/` after
//! explicit user confirmation.

use std::path::{Path, PathBuf};
use crate::config;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct HookTemplate {
    pub name: String,
    #[allow(dead_code)]
    pub source: PathBuf,
    pub content: String,
}

#[derive(Debug)]
pub struct HooksCheckResult {
    pub templates_found: Vec<String>,
    pub templates_missing: Vec<String>,
    pub all_valid: bool,
}

#[derive(Debug)]
pub struct HooksInstallResult {
    pub copied: Vec<String>,
    pub backed_up: Vec<String>,
    pub cancelled: bool,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Template discovery
// ---------------------------------------------------------------------------

/// Locate hook template files. Searches:
/// 1. `<project_root>/templates/hooks/`
/// 2. `<exe_dir>/templates/hooks/`
pub fn find_hook_templates(project_root: &Path) -> Vec<HookTemplate> {
    let mut templates = Vec::new();
    let template_names = ["pre-tool-call.sh", "pre-tool-call.ps1"];

    let candidates: Vec<PathBuf> = vec![
        Some(project_root.join("templates").join("hooks")),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("templates").join("hooks"))),
    ].into_iter().flatten().collect();

    for dir in candidates {
        if !dir.is_dir() {
            continue;
        }
        for name in &template_names {
            let path = dir.join(name);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    templates.push(HookTemplate {
                        name: name.to_string(),
                        source: path,
                        content,
                    });
                }
            }
        }
        // Stop after first directory that has templates
        if !templates.is_empty() {
            break;
        }
    }

    templates
}

// ---------------------------------------------------------------------------
// Hooks check
// ---------------------------------------------------------------------------

/// Verify hook templates exist, are readable, and contain required elements:
/// - calls `akar safety`
/// - reads from stdin (not argv)
/// - writes to `.akar/HOOK_EVENTS.jsonl`
/// - uses exit 2 for BLOCK
pub fn check_hooks(cfg: &config::Config) -> HooksCheckResult {
    let templates = find_hook_templates(&cfg.project_root);
    let mut templates_found = Vec::new();
    let mut templates_missing = Vec::new();

    let expected = ["pre-tool-call.sh", "pre-tool-call.ps1"];

    for name in &expected {
        if let Some(t) = templates.iter().find(|t| t.name == *name) {
            let mut issues = Vec::new();

            if !t.content.contains("akar safety") {
                issues.push("missing 'akar safety' call");
            }

            // stdin check: bash uses 'cat', ps1 uses '$input'
            let reads_stdin = if name.ends_with(".sh") {
                t.content.contains("cat")
            } else {
                t.content.contains("$input")
            };
            if !reads_stdin {
                issues.push("does not read stdin JSON");
            }

            if !t.content.contains("HOOK_EVENTS.jsonl") {
                issues.push("does not write to .akar/HOOK_EVENTS.jsonl");
            }

            if !t.content.contains("exit 2") {
                issues.push("does not use exit 2 for BLOCK");
            }

            if issues.is_empty() {
                templates_found.push(name.to_string());
            } else {
                templates_missing.push(format!("{} ({})", name, issues.join(", ")));
            }
        } else {
            templates_missing.push(name.to_string());
        }
    }

    HooksCheckResult {
        all_valid: templates_missing.is_empty(),
        templates_found,
        templates_missing,
    }
}

// ---------------------------------------------------------------------------
// Hooks install
// ---------------------------------------------------------------------------

/// Install hook templates into `.akar/hooks/`. Creates backups before overwrite.
/// Returns the result without writing anything if `confirmed` is false.
pub fn install_hooks(cfg: &config::Config, confirmed: bool) -> HooksInstallResult {
    let templates = find_hook_templates(&cfg.project_root);

    if templates.is_empty() {
        return HooksInstallResult {
            copied: Vec::new(),
            backed_up: Vec::new(),
            cancelled: true,
            reason: "no hook templates found".to_string(),
        };
    }

    if !confirmed {
        let mut files_to_copy = Vec::new();
        for t in &templates {
            files_to_copy.push(t.name.clone());
        }
        return HooksInstallResult {
            copied: Vec::new(),
            backed_up: Vec::new(),
            cancelled: true,
            reason: format!("would copy: {}", files_to_copy.join(", ")),
        };
    }

    let hooks_dir = cfg.akar_dir.join("hooks");
    if let Err(e) = std::fs::create_dir_all(&hooks_dir) {
        return HooksInstallResult {
            copied: Vec::new(),
            backed_up: Vec::new(),
            cancelled: true,
            reason: format!("failed to create {}: {}", hooks_dir.display(), e),
        };
    }

    let mut copied = Vec::new();
    let mut backed_up = Vec::new();

    for t in &templates {
        let dest = hooks_dir.join(&t.name);

        // Backup existing file before overwrite
        if dest.exists() {
            if let Err(e) = crate::backup::backup_file(&dest) {
                return HooksInstallResult {
                    copied,
                    backed_up,
                    cancelled: true,
                    reason: format!("backup failed for {}: {}", dest.display(), e),
                };
            }
            backed_up.push(t.name.clone());
        }

        // Copy template
        match std::fs::write(&dest, &t.content) {
            Ok(_) => copied.push(t.name.clone()),
            Err(e) => {
                return HooksInstallResult {
                    copied,
                    backed_up,
                    cancelled: true,
                    reason: format!("write failed for {}: {}", dest.display(), e),
                };
            }
        }
    }

    HooksInstallResult {
        copied,
        backed_up,
        cancelled: false,
        reason: "ok".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

pub fn format_hooks_help() -> String {
    let mut out = String::new();
    out.push_str("akar hooks:\n");
    out.push_str("  AKAR ships hook templates for Claude Code integration.\n");
    out.push_str("  These templates call `akar safety` before tool execution.\n");
    out.push_str("\n");
    out.push_str("Templates:\n");
    out.push_str("  templates/hooks/pre-tool-call.sh   (bash)\n");
    out.push_str("  templates/hooks/pre-tool-call.ps1  (PowerShell)\n");
    out.push_str("\n");
    out.push_str("Commands:\n");
    out.push_str("  akar hooks              Show this help\n");
    out.push_str("  akar hooks --check      Verify templates exist and are valid\n");
    out.push_str("  akar hooks --install    Copy templates into .akar/hooks/ (requires confirmation)\n");
    out.push_str("\n");
    out.push_str("Manual install:\n");
    out.push_str("  1. Copy templates/hooks/pre-tool-call.sh to your project\n");
    out.push_str("  2. Register in ~/.claude/settings.json under hooks.preToolCall\n");
    out.push_str("  3. Test with: echo 'rm -rf /' | bash pre-tool-call.sh\n");
    out.push_str("\n");
    out.push_str("Note: AKAR does not install hooks automatically in v0.5.0.\n");
    out.push_str("      The user must copy templates and configure Claude Code manually.\n");
    out
}

pub fn format_hooks_check(result: &HooksCheckResult) -> String {
    let mut out = String::new();
    out.push_str("hooks check:\n");

    if result.all_valid {
        out.push_str("  status: PASS\n");
        out.push_str("  templates found:\n");
        for name in &result.templates_found {
            out.push_str(&format!("    - {}\n", name));
        }
    } else {
        out.push_str("  status: FAIL\n");
        if !result.templates_found.is_empty() {
            out.push_str("  valid:\n");
            for name in &result.templates_found {
                out.push_str(&format!("    - {}\n", name));
            }
        }
        out.push_str("  missing or invalid:\n");
        for name in &result.templates_missing {
            out.push_str(&format!("    - {}\n", name));
        }
        out.push_str(&format!("  guidance: {}\n", crate::foundation::hook_broken_playbook()));
    }

    out
}

pub fn format_hooks_install(result: &HooksInstallResult) -> String {
    let mut out = String::new();
    out.push_str("hooks install:\n");

    if result.cancelled {
        out.push_str(&format!("  cancelled: {}\n", result.reason));
    } else {
        out.push_str("  status: ok\n");
        if !result.backed_up.is_empty() {
            out.push_str("  backed up:\n");
            for name in &result.backed_up {
                out.push_str(&format!("    - {}\n", name));
            }
        }
        out.push_str("  copied:\n");
        for name in &result.copied {
            out.push_str(&format!("    - {}\n", name));
        }
        out.push_str("\n");
        out.push_str("  next: register hooks in ~/.claude/settings.json\n");
        out.push_str("  example:\n");
        out.push_str("    {\n");
        out.push_str("      \"hooks\": {\n");
        out.push_str("        \"PreToolUse\": [\n");
        out.push_str("          {\n");
        out.push_str("            \"matcher\": \"Bash\",\n");
        out.push_str("            \"hooks\": [\n");
        out.push_str("              {\n");
        out.push_str("                \"type\": \"command\",\n");
        out.push_str("                \"command\": \"pwsh \\\"C:\\\\path\\\\to\\\\akar\\\\templates\\\\hooks\\\\pre-tool-call.ps1\\\"\"\n");
        out.push_str("              }\n");
        out.push_str("            ]\n");
        out.push_str("          }\n");
        out.push_str("        ]\n");
        out.push_str("      }\n");
        out.push_str("    }\n");
    }

    out
}

// ---------------------------------------------------------------------------
// Hook JSON parsing (mirrors template logic — used in tests and future CLI)
// ---------------------------------------------------------------------------

/// Parsed fields from a Claude Code PreToolUse JSON event.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct HookEvent {
    pub tool_name: String,
    /// Empty string when not present or not a Bash call.
    pub command: String,
}

/// Parse the relevant fields from a Claude Code PreToolUse JSON stdin payload.
/// Uses only std — no external JSON parser.
#[allow(dead_code)]
pub fn parse_hook_event(json: &str) -> HookEvent {
    let tool_name = extract_json_str_value(json, "tool_name").unwrap_or_default();
    let command = extract_json_str_value(json, "command").unwrap_or_default();
    HookEvent { tool_name, command }
}

/// Decide what to do with a parsed hook event.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum HookDecision {
    /// Not a Bash tool — skip safety check.
    Skip,
    /// Bash tool — check this command string with `akar safety`.
    Check(String),
    /// Bash tool but no command field present.
    Allow,
}

#[allow(dead_code)]
pub fn hook_decision(event: &HookEvent) -> HookDecision {
    if event.tool_name != "Bash" {
        return HookDecision::Skip;
    }
    let cmd = event.command.trim().to_string();
    if cmd.is_empty() {
        return HookDecision::Allow;
    }
    HookDecision::Check(cmd)
}

#[allow(dead_code)]
fn extract_json_str_value(s: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let key_pos = s.find(&needle)?;
    let after_key = &s[key_pos + needle.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = after_key[colon_pos + 1..].trim_start();
    if !after_colon.starts_with('"') {
        return None;
    }
    let inner = &after_colon[1..];
    let mut val = String::new();
    let mut escaped = false;
    for c in inner.chars() {
        if escaped {
            match c {
                'n' => val.push('\n'),
                't' => val.push('\t'),
                '"' => val.push('"'),
                '\\' => val.push('\\'),
                _ => { val.push('\\'); val.push(c); }
            }
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_cfg(name: &str) -> (config::Config, PathBuf) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("akar_hooks_test_{}_{}", name, ts));
        let _ = fs::create_dir_all(&dir);
        let cfg = config::Config {
            project_root: dir.clone(),
            akar_dir: dir.join(".akar"),
            global_dir: dir.join("global"),
            project_name: "test".to_string(),
        };
        (cfg, dir)
    }

    fn write_templates(project_root: &Path) {
        let hooks_dir = project_root.join("templates").join("hooks");
        let _ = fs::create_dir_all(&hooks_dir);
        let _ = fs::write(
            hooks_dir.join("pre-tool-call.sh"),
            "#!/bin/bash\nJSON=$(cat)\nakar safety \"$CMD\"\nHOOK_EVENTS.jsonl\nexit 2\n",
        );
        let _ = fs::write(
            hooks_dir.join("pre-tool-call.ps1"),
            "# PowerShell\n$input | Out-String\nakar safety $Command\nHOOK_EVENTS.jsonl\nexit 2\n",
        );
    }

    #[test]
    fn check_passes_when_templates_exist() {
        let (cfg, dir) = temp_cfg("check_pass");
        write_templates(&dir);
        let result = check_hooks(&cfg);
        assert!(result.all_valid, "expected all_valid=true, got: {:?}", result.templates_missing);
        assert_eq!(result.templates_found.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_fails_when_templates_missing() {
        let (cfg, dir) = temp_cfg("check_fail");
        let result = check_hooks(&cfg);
        assert!(!result.all_valid);
        assert_eq!(result.templates_missing.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_fails_when_template_missing_akar_safety() {
        let (cfg, dir) = temp_cfg("check_no_safety");
        let hooks_dir = dir.join("templates").join("hooks");
        let _ = fs::create_dir_all(&hooks_dir);
        let _ = fs::write(hooks_dir.join("pre-tool-call.sh"), "#!/bin/bash\nJSON=$(cat)\nHOOK_EVENTS.jsonl\nexit 2\n");
        let _ = fs::write(hooks_dir.join("pre-tool-call.ps1"), "# PowerShell\n$input | Out-String\nHOOK_EVENTS.jsonl\nexit 2\n");
        let result = check_hooks(&cfg);
        assert!(!result.all_valid);
        assert!(result.templates_missing.iter().any(|m| m.contains("akar safety")));
        let _ = fs::remove_dir_all(&dir);
    }

    // -- hook logging tests ---------------------------------------------------

    #[test]
    fn hook_templates_write_to_hook_events_jsonl() {
        let templates = find_hook_templates(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        );
        assert!(!templates.is_empty(), "no templates found");
        for t in &templates {
            assert!(
                t.content.contains("HOOK_EVENTS.jsonl"),
                "template {} must write to .akar/HOOK_EVENTS.jsonl", t.name
            );
        }
    }

    #[test]
    fn hook_templates_do_not_log_full_stdin_json() {
        let templates = find_hook_templates(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        );
        for t in &templates {
            // The full JSON blob variable must not be written to the log line
            // (bash: $JSON must not appear in the write_event/log line;
            //  ps1: $json must not appear in the Add-Content logLine)
            let log_line_writes_full_json = if t.name.ends_with(".sh") {
                // Check that no line writing to HOOK_EVENTS.jsonl contains $JSON verbatim
                t.content.lines().any(|line| {
                    line.contains("HOOK_EVENTS.jsonl") && line.contains("$JSON")
                })
            } else {
                // ps1: no logLine construction should contain $json (the full blob variable)
                t.content.lines().any(|line| {
                    (line.contains("HOOK_EVENTS.jsonl") || line.contains("logLine")) && line.contains("$json")
                })
            };
            assert!(
                !log_line_writes_full_json,
                "template {} must not write full stdin JSON blob to hook log", t.name
            );
        }
    }

    #[test]
    fn hook_check_fails_when_logging_line_missing() {
        let (cfg, dir) = temp_cfg("check_no_logging");
        let hooks_dir = dir.join("templates").join("hooks");
        let _ = fs::create_dir_all(&hooks_dir);
        // Templates have akar safety and stdin but no HOOK_EVENTS.jsonl
        let _ = fs::write(hooks_dir.join("pre-tool-call.sh"),
            "#!/bin/bash\nJSON=$(cat)\nakar safety \"$CMD\"\nexit 2\n");
        let _ = fs::write(hooks_dir.join("pre-tool-call.ps1"),
            "# PowerShell\n$input | Out-String\nakar safety $cmd\nexit 2\n");
        let result = check_hooks(&cfg);
        assert!(!result.all_valid);
        assert!(result.templates_missing.iter().any(|m| m.contains("HOOK_EVENTS.jsonl")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hook_check_fails_when_stdin_read_missing() {
        let (cfg, dir) = temp_cfg("check_no_stdin");
        let hooks_dir = dir.join("templates").join("hooks");
        let _ = fs::create_dir_all(&hooks_dir);
        // Templates have akar safety and logging but no stdin read
        let _ = fs::write(hooks_dir.join("pre-tool-call.sh"),
            "#!/bin/bash\nakar safety \"$1\"\nHOOK_EVENTS.jsonl\nexit 2\n");
        let _ = fs::write(hooks_dir.join("pre-tool-call.ps1"),
            "# PowerShell\nakar safety $Command\nHOOK_EVENTS.jsonl\nexit 2\n");
        let result = check_hooks(&cfg);
        assert!(!result.all_valid);
        assert!(result.templates_missing.iter().any(|m| m.contains("stdin")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn install_cancelled_when_not_confirmed() {
        let (cfg, dir) = temp_cfg("install_cancel");
        write_templates(&dir);
        let result = install_hooks(&cfg, false);
        assert!(result.cancelled);
        assert!(result.copied.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn install_copies_when_confirmed() {
        let (cfg, dir) = temp_cfg("install_copy");
        write_templates(&dir);
        let _ = fs::create_dir_all(&cfg.akar_dir);
        let result = install_hooks(&cfg, true);
        assert!(!result.cancelled);
        assert_eq!(result.copied.len(), 2);
        assert!(cfg.akar_dir.join("hooks").join("pre-tool-call.sh").exists());
        assert!(cfg.akar_dir.join("hooks").join("pre-tool-call.ps1").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn install_creates_backup_before_overwrite() {
        let (cfg, dir) = temp_cfg("install_backup");
        write_templates(&dir);
        let _ = fs::create_dir_all(&cfg.akar_dir);
        let hooks_dir = cfg.akar_dir.join("hooks");
        let _ = fs::create_dir_all(&hooks_dir);
        let _ = fs::write(hooks_dir.join("pre-tool-call.sh"), "old content\n");
        let result = install_hooks(&cfg, true);
        assert!(!result.cancelled);
        assert!(!result.backed_up.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn install_fails_when_no_templates_found() {
        let (cfg, dir) = temp_cfg("install_no_templates");
        let result = install_hooks(&cfg, true);
        assert!(result.cancelled);
        assert!(result.reason.contains("no hook templates"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn format_hooks_help_contains_key_info() {
        let out = format_hooks_help();
        assert!(out.contains("akar hooks"));
        assert!(out.contains("pre-tool-call.sh"));
        assert!(out.contains("pre-tool-call.ps1"));
        assert!(out.contains("does not install hooks automatically"));
    }

    // -- hook JSON parsing ----------------------------------------------------

    const BASH_CARGO_TEST: &str = r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test"}}"#;
    const BASH_RM_RF: &str = r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}"#;
    const NON_BASH_READ: &str = r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"/foo"}}"#;
    const BASH_NO_COMMAND: &str = r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{}}"#;

    #[test]
    fn parse_hook_event_bash_cargo_test() {
        let e = parse_hook_event(BASH_CARGO_TEST);
        assert_eq!(e.tool_name, "Bash");
        assert_eq!(e.command, "cargo test");
    }

    #[test]
    fn parse_hook_event_bash_rm_rf() {
        let e = parse_hook_event(BASH_RM_RF);
        assert_eq!(e.tool_name, "Bash");
        assert_eq!(e.command, "rm -rf /");
    }

    #[test]
    fn parse_hook_event_non_bash_read() {
        let e = parse_hook_event(NON_BASH_READ);
        assert_eq!(e.tool_name, "Read");
        assert_eq!(e.command, "");
    }

    #[test]
    fn parse_hook_event_bash_no_command() {
        let e = parse_hook_event(BASH_NO_COMMAND);
        assert_eq!(e.tool_name, "Bash");
        assert_eq!(e.command, "");
    }

    #[test]
    fn hook_decision_non_bash_is_skip() {
        let e = parse_hook_event(NON_BASH_READ);
        assert_eq!(hook_decision(&e), HookDecision::Skip);
    }

    #[test]
    fn hook_decision_bash_cargo_test_is_check() {
        let e = parse_hook_event(BASH_CARGO_TEST);
        assert_eq!(hook_decision(&e), HookDecision::Check("cargo test".to_string()));
    }

    #[test]
    fn hook_decision_bash_rm_rf_is_check() {
        let e = parse_hook_event(BASH_RM_RF);
        assert_eq!(hook_decision(&e), HookDecision::Check("rm -rf /".to_string()));
    }

    #[test]
    fn hook_decision_bash_no_command_is_allow() {
        let e = parse_hook_event(BASH_NO_COMMAND);
        assert_eq!(hook_decision(&e), HookDecision::Allow);
    }

    #[test]
    fn hook_decision_check_integrates_with_safety_cargo_test() {
        // cargo test should be Safe (not blocked)
        let e = parse_hook_event(BASH_CARGO_TEST);
        if let HookDecision::Check(cmd) = hook_decision(&e) {
            let assessment = crate::safety::classify_command(&cmd);
            assert!(!assessment.blocked, "cargo test must not be blocked");
        } else {
            panic!("expected Check decision");
        }
    }

    #[test]
    fn hook_decision_check_integrates_with_safety_rm_rf() {
        // rm -rf / should be BLOCKED
        let e = parse_hook_event(BASH_RM_RF);
        if let HookDecision::Check(cmd) = hook_decision(&e) {
            let assessment = crate::safety::classify_command(&cmd);
            assert!(assessment.blocked, "rm -rf / must be blocked");
        } else {
            panic!("expected Check decision");
        }
    }

    #[test]
    fn hook_templates_use_exit_2_for_block() {
        // Templates must use exit 2 (not exit 1) — Claude Code only blocks on exit 2
        let templates = find_hook_templates(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        );
        for t in &templates {
            assert!(
                t.content.contains("exit 2"),
                "template {} must use exit 2 for BLOCKED, not exit 1", t.name
            );
            // Check that no non-comment line uses exit 1
            for line in t.content.lines() {
                let trimmed = line.trim();
                let is_comment = trimmed.starts_with('#') || trimmed.starts_with("//");
                if !is_comment {
                    assert!(
                        !trimmed.contains("exit 1"),
                        "template {} has executable exit 1 on line: {}", t.name, line
                    );
                }
            }
        }
    }

    #[test]
    fn hook_templates_read_stdin_not_argv() {
        // Templates must read JSON from stdin, not $1 / param
        let templates = find_hook_templates(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        );
        for t in &templates {
            if t.name.ends_with(".sh") {
                assert!(
                    t.content.contains("cat") || t.content.contains("stdin") || t.content.contains("read"),
                    "bash template must read from stdin"
                );
            }
            if t.name.ends_with(".ps1") {
                assert!(
                    t.content.contains("$input"),
                    "PowerShell template must read from $input (stdin)"
                );
            }
        }
    }
}
