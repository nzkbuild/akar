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

/// Verify hook templates exist, are readable, and contain `akar safety`.
pub fn check_hooks(cfg: &config::Config) -> HooksCheckResult {
    let templates = find_hook_templates(&cfg.project_root);
    let mut templates_found = Vec::new();
    let mut templates_missing = Vec::new();

    let expected = ["pre-tool-call.sh", "pre-tool-call.ps1"];

    for name in &expected {
        if let Some(t) = templates.iter().find(|t| t.name == *name) {
            if t.content.contains("akar safety") {
                templates_found.push(name.to_string());
            } else {
                templates_missing.push(format!("{} (missing 'akar safety' call)", name));
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
        out.push_str("        \"preToolCall\": [\n");
        out.push_str("          {\n");
        out.push_str("            \"matcher\": \"\",\n");
        out.push_str("            \"command\": \"bash .akar/hooks/pre-tool-call.sh $COMMAND\"\n");
        out.push_str("          }\n");
        out.push_str("        ]\n");
        out.push_str("      }\n");
        out.push_str("    }\n");
    }

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
            "#!/bin/bash\nakar safety \"$1\"\n",
        );
        let _ = fs::write(
            hooks_dir.join("pre-tool-call.ps1"),
            "# PowerShell\nakar safety $Command\n",
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
        let _ = fs::write(hooks_dir.join("pre-tool-call.sh"), "#!/bin/bash\necho hello\n");
        let _ = fs::write(hooks_dir.join("pre-tool-call.ps1"), "# PowerShell\necho hello\n");
        let result = check_hooks(&cfg);
        assert!(!result.all_valid);
        assert!(result.templates_missing.iter().any(|m| m.contains("akar safety")));
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
}
