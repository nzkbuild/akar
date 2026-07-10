//! Hook template management — check, install, and verify hook scripts.
//!
//! AKAR embeds the PreToolUse hook templates directly in the binary (via
//! `include_str!`) so `akar hooks --install`, `akar hooks --check`, and
//! `akar doctor` work in a fresh external repo **without the AKAR source
//! tree**. The source-tree `templates/hooks/` files remain the editable
//! source; the embedded copies are regenerated at compile time.
//!
//! Template discovery order for `--check`:
//! 1. **source-tree** — `<project_root>/templates/hooks/` (dev mode)
//! 2. **exe-dir** — `<exe_dir>/templates/hooks/` (rare)
//! 3. **project-installed** — `<project_root>/.akar/hooks/` (after `--install`)
//! 4. **embedded fallback** — the `include_str!` copies baked into the binary
//!
//! `--install` always writes the **embedded** templates so it works without a
//! source tree. AKAR never modifies `~/.claude/settings.json`; the user wires
//! the PreToolUse hook manually.

use crate::config;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Embedded templates (compiled into the binary)
// ---------------------------------------------------------------------------

/// The bash PreToolUse hook template, embedded at compile time.
pub const EMBEDDED_HOOK_SH: &str = include_str!("../templates/hooks/pre-tool-call.sh");

/// The PowerShell PreToolUse hook template, embedded at compile time.
pub const EMBEDDED_HOOK_PS1: &str = include_str!("../templates/hooks/pre-tool-call.ps1");

/// Return the embedded template content for a given template name, or `None`.
pub fn embedded_template_content(name: &str) -> Option<&'static str> {
    match name {
        "pre-tool-call.sh" => Some(EMBEDDED_HOOK_SH),
        "pre-tool-call.ps1" => Some(EMBEDDED_HOOK_PS1),
        _ => None,
    }
}

/// The set of expected hook template names, in stable order.
pub const EXPECTED_HOOK_TEMPLATES: &[&str] = &["pre-tool-call.sh", "pre-tool-call.ps1"];

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Where a hook template was discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookTemplateSource {
    /// `<project_root>/templates/hooks/` (source tree / dev mode).
    SourceTree,
    /// `<exe_dir>/templates/hooks/`.
    ExeDir,
    /// `<project_root>/.akar/hooks/` (installed by `akar hooks --install`).
    ProjectInstalled,
    /// Compiled into the binary via `include_str!`.
    Embedded,
}

impl HookTemplateSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookTemplateSource::SourceTree => "source-tree",
            HookTemplateSource::ExeDir => "exe-dir",
            HookTemplateSource::ProjectInstalled => "project .akar/hooks",
            HookTemplateSource::Embedded => "embedded",
        }
    }
}

#[derive(Debug)]
pub struct HookTemplate {
    pub name: String,
    /// Filesystem path the template was read from (synthetic for embedded).
    #[allow(dead_code)]
    pub source: PathBuf,
    pub content: String,
    /// How this template was discovered (per-template; the aggregated source
    /// used by `check_hooks` is returned separately by `discover_hook_templates`).
    #[allow(dead_code)]
    pub discovered_via: HookTemplateSource,
}

#[derive(Debug)]
pub struct HooksCheckResult {
    pub templates_found: Vec<String>,
    pub templates_missing: Vec<String>,
    pub all_valid: bool,
    /// Where the checked templates came from (None if none found).
    pub source: Option<HookTemplateSource>,
}

#[derive(Debug)]
pub struct HooksInstallResult {
    pub copied: Vec<String>,
    pub backed_up: Vec<String>,
    /// Files whose content already matched the embedded template (no write).
    pub unchanged: Vec<String>,
    pub cancelled: bool,
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Template discovery
// ---------------------------------------------------------------------------

/// Locate hook template files from the source tree or exe dir.
///
/// Searches:
/// 1. `<project_root>/templates/hooks/` (source tree / dev mode)
/// 2. `<exe_dir>/templates/hooks/`
///
/// Does NOT include the embedded fallback or project-installed templates —
/// those are handled by [`discover_hook_templates`], which `check_hooks` and
/// `install_hooks` use. Retained as a public API for tests that validate the
/// source-tree template files directly.
#[allow(dead_code)]
pub fn find_hook_templates(project_root: &Path) -> Vec<HookTemplate> {
    let (templates, _source) = discover_from_filesystem(project_root);
    templates
}

/// Filesystem-only discovery (source-tree then exe-dir). Returns the templates
/// found and which source they came from. Returns `(vec![], None)` if neither
/// directory has templates.
fn discover_from_filesystem(
    project_root: &Path,
) -> (Vec<HookTemplate>, Option<HookTemplateSource>) {
    let candidates: Vec<(PathBuf, HookTemplateSource)> = vec![
        (
            project_root.join("templates").join("hooks"),
            HookTemplateSource::SourceTree,
        ),
        {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("templates").join("hooks")));
            match exe_dir {
                Some(d) => (d, HookTemplateSource::ExeDir),
                None => (
                    PathBuf::from("/__nonexistent_exe_dir__"),
                    HookTemplateSource::ExeDir,
                ),
            }
        },
    ];

    for (dir, src) in &candidates {
        if !dir.is_dir() {
            continue;
        }
        let mut templates = Vec::new();
        for name in EXPECTED_HOOK_TEMPLATES {
            let path = dir.join(name);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    templates.push(HookTemplate {
                        name: name.to_string(),
                        source: path,
                        content,
                        discovered_via: *src,
                    });
                }
            }
        }
        if !templates.is_empty() {
            return (templates, Some(*src));
        }
    }

    (Vec::new(), None)
}

/// Read installed templates from `<project_root>/.akar/hooks/`. Returns the
/// templates found (with `discovered_via = ProjectInstalled`) or an empty vec.
fn discover_installed(akar_dir: &Path) -> Vec<HookTemplate> {
    let dir = akar_dir.join("hooks");
    if !dir.is_dir() {
        return Vec::new();
    }
    let mut templates = Vec::new();
    for name in EXPECTED_HOOK_TEMPLATES {
        let path = dir.join(name);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                templates.push(HookTemplate {
                    name: name.to_string(),
                    source: path,
                    content,
                    discovered_via: HookTemplateSource::ProjectInstalled,
                });
            }
        }
    }
    templates
}

/// Return the embedded templates as `HookTemplate`s (synthetic source path).
fn embedded_templates() -> Vec<HookTemplate> {
    EXPECTED_HOOK_TEMPLATES
        .iter()
        .filter_map(|name| {
            embedded_template_content(name).map(|content| HookTemplate {
                name: name.to_string(),
                source: PathBuf::from(format!("<embedded>/{}", name)),
                content: content.to_string(),
                discovered_via: HookTemplateSource::Embedded,
            })
        })
        .collect()
}

/// Discover hook templates in priority order, returning the templates and the
/// source they came from:
///
/// 1. source-tree / exe-dir filesystem templates (if present)
/// 2. project-installed `.akar/hooks/` templates (if present)
/// 3. embedded fallback (always available from the binary)
pub fn discover_hook_templates(cfg: &config::Config) -> (Vec<HookTemplate>, HookTemplateSource) {
    let (fs_templates, fs_source) = discover_from_filesystem(&cfg.project_root);
    if !fs_templates.is_empty() {
        return (
            fs_templates,
            fs_source.unwrap_or(HookTemplateSource::SourceTree),
        );
    }

    let installed = discover_installed(&cfg.akar_dir);
    if !installed.is_empty() {
        return (installed, HookTemplateSource::ProjectInstalled);
    }

    (embedded_templates(), HookTemplateSource::Embedded)
}

// ---------------------------------------------------------------------------
// Hooks check
// ---------------------------------------------------------------------------

/// Verify hook templates exist, are readable, and contain required elements:
/// - calls `akar safety`
/// - reads from stdin (not argv)
/// - writes to `.akar/HOOK_EVENTS.jsonl`
/// - uses exit 2 for BLOCK
///
/// Templates are discovered in priority order (source-tree → exe-dir →
/// project-installed `.akar/hooks/` → embedded fallback). The result records
/// which source was used. A fresh external repo with no source-tree templates
/// still PASSes via the embedded fallback.
pub fn check_hooks(cfg: &config::Config) -> HooksCheckResult {
    let (templates, source) = discover_hook_templates(cfg);
    let mut templates_found = Vec::new();
    let mut templates_missing = Vec::new();

    for name in EXPECTED_HOOK_TEMPLATES {
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

    let all_valid = templates_missing.is_empty() && !templates_found.is_empty();
    HooksCheckResult {
        all_valid,
        templates_found,
        templates_missing,
        source: Some(source),
    }
}

// ---------------------------------------------------------------------------
// Hooks install
// ---------------------------------------------------------------------------

/// Install hook templates into `.akar/hooks/` from the **embedded** templates
/// baked into the binary, so installation works in a fresh external repo
/// without the AKAR source tree.
///
/// Behavior:
/// - Ensures `.akar/hooks/` exists.
/// - For each expected template:
///   - if the dest does not exist → write it (`copied`)
///   - if the dest exists and content is identical to the embedded template →
///     skip (`unchanged`)
///   - if the dest exists and content differs → back up the existing file,
///     then overwrite (`copied` + `backed_up`)
/// - Does not modify `~/.claude/settings.json`.
///
/// Returns without writing anything if `confirmed` is false.
pub fn install_hooks(cfg: &config::Config, confirmed: bool) -> HooksInstallResult {
    let embedded = embedded_templates();

    if embedded.is_empty() {
        return HooksInstallResult {
            copied: Vec::new(),
            backed_up: Vec::new(),
            unchanged: Vec::new(),
            cancelled: true,
            reason: "no embedded hook templates available".to_string(),
        };
    }

    if !confirmed {
        let mut files_to_copy = Vec::new();
        for t in &embedded {
            files_to_copy.push(t.name.clone());
        }
        return HooksInstallResult {
            copied: Vec::new(),
            backed_up: Vec::new(),
            unchanged: Vec::new(),
            cancelled: true,
            reason: format!("would copy: {}", files_to_copy.join(", ")),
        };
    }

    let hooks_dir = cfg.akar_dir.join("hooks");
    if let Err(e) = std::fs::create_dir_all(&hooks_dir) {
        return HooksInstallResult {
            copied: Vec::new(),
            backed_up: Vec::new(),
            unchanged: Vec::new(),
            cancelled: true,
            reason: format!("failed to create {}: {}", hooks_dir.display(), e),
        };
    }

    let mut copied = Vec::new();
    let mut backed_up = Vec::new();
    let mut unchanged = Vec::new();

    for t in &embedded {
        let dest = hooks_dir.join(&t.name);

        if dest.exists() {
            let existing = std::fs::read_to_string(&dest).unwrap_or_default();
            if existing == t.content {
                unchanged.push(t.name.clone());
                continue;
            }
            // Content differs — back up before overwrite.
            if let Err(e) = crate::backup::backup_file(&dest) {
                return HooksInstallResult {
                    copied,
                    backed_up,
                    unchanged,
                    cancelled: true,
                    reason: format!("backup failed for {}: {}", dest.display(), e),
                };
            }
            backed_up.push(t.name.clone());
        }

        match std::fs::write(&dest, &t.content) {
            Ok(_) => copied.push(t.name.clone()),
            Err(e) => {
                return HooksInstallResult {
                    copied,
                    backed_up,
                    unchanged,
                    cancelled: true,
                    reason: format!("write failed for {}: {}", dest.display(), e),
                };
            }
        }
    }

    HooksInstallResult {
        copied,
        backed_up,
        unchanged,
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
    out.push_str("  AKAR embeds PreToolUse hook templates in the binary and can install\n");
    out.push_str("  them into a project's .akar/hooks/. The templates call `akar safety`\n");
    out.push_str("  before tool execution. AKAR never edits ~/.claude/settings.json.\n");
    out.push_str("\n");
    out.push_str("Templates (embedded in the binary):\n");
    out.push_str("  pre-tool-call.sh   (bash)\n");
    out.push_str("  pre-tool-call.ps1  (PowerShell)\n");
    out.push_str("\n");
    out.push_str("Commands:\n");
    out.push_str("  akar hooks              Show this help\n");
    out.push_str("  akar hooks --check      Verify templates (source-tree, project .akar/hooks/, or embedded)\n");
    out.push_str("  akar hooks --install    Write embedded templates into .akar/hooks/ (requires confirmation)\n");
    out.push_str("\n");
    out.push_str("Manual wiring (required — AKAR does not do this):\n");
    out.push_str("  1. Run `akar hooks --install` to write templates to .akar/hooks/\n");
    out.push_str("  2. Register the hook in ~/.claude/settings.json under hooks.preToolUse\n");
    out.push_str("     pointing at .akar/hooks/pre-tool-call.ps1 (Windows) or .sh (POSIX)\n");
    out.push_str("  3. Test with: echo 'rm -rf /' | bash .akar/hooks/pre-tool-call.sh\n");
    out.push_str("\n");
    out.push_str("Note: AKAR does not install hooks into Claude Code automatically.\n");
    out.push_str("      The user must copy templates and configure Claude Code manually.\n");
    out
}

pub fn format_hooks_check(result: &HooksCheckResult) -> String {
    let mut out = String::new();
    out.push_str("hooks check:\n");

    if let Some(src) = &result.source {
        out.push_str(&format!("  source: {}\n", src.as_str()));
    }

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
        out.push_str(&format!(
            "  guidance: {}\n",
            crate::foundation::hook_broken_playbook()
        ));
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
        if !result.copied.is_empty() {
            out.push_str("  copied (written to .akar/hooks/):\n");
            for name in &result.copied {
                out.push_str(&format!("    - {}\n", name));
            }
        }
        if !result.unchanged.is_empty() {
            out.push_str("  unchanged (content already matches embedded template):\n");
            for name in &result.unchanged {
                out.push_str(&format!("    - {}\n", name));
            }
        }
        if !result.backed_up.is_empty() {
            out.push_str("  backed up (existing file differed — backed up before overwrite):\n");
            for name in &result.backed_up {
                out.push_str(&format!("    - {}\n", name));
            }
        }
        out.push_str("\n");
        out.push_str("  next: register the hook in ~/.claude/settings.json manually\n");
        out.push_str("  (AKAR will NOT edit ~/.claude/settings.json).\n");
        out.push_str("  example:\n");
        out.push_str("    {\n");
        out.push_str("      \"hooks\": {\n");
        out.push_str("        \"PreToolUse\": [\n");
        out.push_str("          {\n");
        out.push_str("            \"matcher\": \"Bash\",\n");
        out.push_str("            \"hooks\": [\n");
        out.push_str("              {\n");
        out.push_str("                \"type\": \"command\",\n");
        out.push_str("                \"command\": \"pwsh \\\"<project>/.akar/hooks/pre-tool-call.ps1\\\"\"\n");
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
    /// The project root cwd from the hook JSON, or empty if absent.
    /// v0.29.0: Claude Code provides this as a top-level "cwd" field.
    pub cwd: String,
}

/// Parse the relevant fields from a Claude Code PreToolUse JSON stdin payload.
/// Uses only std — no external JSON parser.
///
/// v0.29.0: also extracts the top-level "cwd" field (the working directory
/// when the hook fired). This is used as the hook event log root.
#[allow(dead_code)]
pub fn parse_hook_event(json: &str) -> HookEvent {
    let tool_name = extract_json_str_value(json, "tool_name").unwrap_or_default();
    let command = extract_json_str_value(json, "command").unwrap_or_default();
    let cwd = extract_json_str_value(json, "cwd").unwrap_or_default();
    HookEvent {
        tool_name,
        command,
        cwd,
    }
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
                _ => {
                    val.push('\\');
                    val.push(c);
                }
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

    // -- embedded templates ---------------------------------------------------

    #[test]
    fn embedded_bash_template_is_nonempty() {
        assert!(
            !EMBEDDED_HOOK_SH.trim().is_empty(),
            "embedded bash template must be non-empty"
        );
        assert!(EMBEDDED_HOOK_SH.contains("akar safety"));
        assert!(EMBEDDED_HOOK_SH.contains("HOOK_EVENTS.jsonl"));
        assert!(EMBEDDED_HOOK_SH.contains("exit 2"));
        // v0.29.0: template must extract cwd from JSON for log root targeting
        assert!(
            EMBEDDED_HOOK_SH.contains("\"cwd\""),
            "bash template must read cwd from JSON"
        );
        assert!(
            EMBEDDED_HOOK_SH.contains("log_root"),
            "bash template must include log_root in events"
        );
    }

    #[test]
    fn embedded_powershell_template_is_nonempty() {
        assert!(
            !EMBEDDED_HOOK_PS1.trim().is_empty(),
            "embedded ps1 template must be non-empty"
        );
        assert!(EMBEDDED_HOOK_PS1.contains("akar safety"));
        assert!(EMBEDDED_HOOK_PS1.contains("HOOK_EVENTS.jsonl"));
        assert!(EMBEDDED_HOOK_PS1.contains("exit 2"));
        // v0.29.0: template must extract cwd from JSON for log root targeting
        assert!(
            EMBEDDED_HOOK_PS1.contains("\"cwd\""),
            "ps1 template must read cwd from JSON"
        );
        assert!(
            EMBEDDED_HOOK_PS1.contains("log_root"),
            "ps1 template must include log_root in events"
        );
    }

    #[test]
    fn embedded_template_accessor_returns_correct_content() {
        assert_eq!(
            embedded_template_content("pre-tool-call.sh"),
            Some(EMBEDDED_HOOK_SH)
        );
        assert_eq!(
            embedded_template_content("pre-tool-call.ps1"),
            Some(EMBEDDED_HOOK_PS1)
        );
        assert_eq!(embedded_template_content("nonexistent"), None);
    }

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
        assert!(
            result.all_valid,
            "expected all_valid=true, got: {:?}",
            result.templates_missing
        );
        assert_eq!(result.templates_found.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_passes_via_embedded_when_no_source_templates() {
        // v0.25: a fresh repo with no source-tree templates must still PASS
        // because the embedded templates are the fallback. This is the core
        // fix for the v0.24 dogfood blocker.
        let (cfg, dir) = temp_cfg("check_embedded");
        let result = check_hooks(&cfg);
        assert!(
            result.all_valid,
            "expected PASS via embedded/templates, got missing: {:?}",
            result.templates_missing
        );
        assert_eq!(result.templates_found.len(), 2);
        // Source must be reported (embedded, exe-dir, or source-tree).
        assert!(result.source.is_some(), "source must be reported");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_fails_when_template_missing_akar_safety() {
        let (cfg, dir) = temp_cfg("check_no_safety");
        let hooks_dir = dir.join("templates").join("hooks");
        let _ = fs::create_dir_all(&hooks_dir);
        let _ = fs::write(
            hooks_dir.join("pre-tool-call.sh"),
            "#!/bin/bash\nJSON=$(cat)\nHOOK_EVENTS.jsonl\nexit 2\n",
        );
        let _ = fs::write(
            hooks_dir.join("pre-tool-call.ps1"),
            "# PowerShell\n$input | Out-String\nHOOK_EVENTS.jsonl\nexit 2\n",
        );
        let result = check_hooks(&cfg);
        assert!(!result.all_valid);
        assert!(
            result
                .templates_missing
                .iter()
                .any(|m| m.contains("akar safety"))
        );
        let _ = fs::remove_dir_all(&dir);
    }

    // -- hook logging tests ---------------------------------------------------

    #[test]
    fn hook_templates_write_to_hook_events_jsonl() {
        let templates = find_hook_templates(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        assert!(!templates.is_empty(), "no templates found");
        for t in &templates {
            assert!(
                t.content.contains("HOOK_EVENTS.jsonl"),
                "template {} must write to .akar/HOOK_EVENTS.jsonl",
                t.name
            );
        }
    }

    #[test]
    fn hook_templates_do_not_log_full_stdin_json() {
        let templates = find_hook_templates(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        for t in &templates {
            // The full JSON blob variable must not be written to the log line
            // (bash: $JSON must not appear in the write_event/log line;
            //  ps1: $json must not appear in the Add-Content logLine)
            let log_line_writes_full_json = if t.name.ends_with(".sh") {
                // Check that no line writing to HOOK_EVENTS.jsonl contains $JSON verbatim
                t.content
                    .lines()
                    .any(|line| line.contains("HOOK_EVENTS.jsonl") && line.contains("$JSON"))
            } else {
                // ps1: no logLine construction should contain $json (the full blob variable)
                t.content.lines().any(|line| {
                    (line.contains("HOOK_EVENTS.jsonl") || line.contains("logLine"))
                        && line.contains("$json")
                })
            };
            assert!(
                !log_line_writes_full_json,
                "template {} must not write full stdin JSON blob to hook log",
                t.name
            );
        }
    }

    #[test]
    fn hook_check_fails_when_logging_line_missing() {
        let (cfg, dir) = temp_cfg("check_no_logging");
        let hooks_dir = dir.join("templates").join("hooks");
        let _ = fs::create_dir_all(&hooks_dir);
        // Templates have akar safety and stdin but no HOOK_EVENTS.jsonl
        let _ = fs::write(
            hooks_dir.join("pre-tool-call.sh"),
            "#!/bin/bash\nJSON=$(cat)\nakar safety \"$CMD\"\nexit 2\n",
        );
        let _ = fs::write(
            hooks_dir.join("pre-tool-call.ps1"),
            "# PowerShell\n$input | Out-String\nakar safety $cmd\nexit 2\n",
        );
        let result = check_hooks(&cfg);
        assert!(!result.all_valid);
        assert!(
            result
                .templates_missing
                .iter()
                .any(|m| m.contains("HOOK_EVENTS.jsonl"))
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn hook_check_fails_when_stdin_read_missing() {
        let (cfg, dir) = temp_cfg("check_no_stdin");
        let hooks_dir = dir.join("templates").join("hooks");
        let _ = fs::create_dir_all(&hooks_dir);
        // Templates have akar safety and logging but no stdin read
        let _ = fs::write(
            hooks_dir.join("pre-tool-call.sh"),
            "#!/bin/bash\nakar safety \"$1\"\nHOOK_EVENTS.jsonl\nexit 2\n",
        );
        let _ = fs::write(
            hooks_dir.join("pre-tool-call.ps1"),
            "# PowerShell\nakar safety $Command\nHOOK_EVENTS.jsonl\nexit 2\n",
        );
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
        assert!(
            cfg.akar_dir
                .join("hooks")
                .join("pre-tool-call.ps1")
                .exists()
        );
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
    fn install_writes_embedded_templates_without_source_tree() {
        // v0.25: install_hooks writes the EMBEDDED templates, so it works in a
        // fresh external repo with no source tree. (Previously it failed with
        // "no hook templates found".)
        let (cfg, dir) = temp_cfg("install_embedded");
        let _ = std::fs::create_dir_all(&cfg.akar_dir);
        let result = install_hooks(&cfg, true);
        assert!(
            !result.cancelled,
            "install should not cancel: {}",
            result.reason
        );
        assert_eq!(
            result.copied.len(),
            2,
            "both embedded templates should be written"
        );
        assert!(cfg.akar_dir.join("hooks").join("pre-tool-call.sh").exists());
        assert!(
            cfg.akar_dir
                .join("hooks")
                .join("pre-tool-call.ps1")
                .exists()
        );
        // The written content must match the embedded templates.
        let sh =
            std::fs::read_to_string(cfg.akar_dir.join("hooks").join("pre-tool-call.sh")).unwrap();
        assert_eq!(sh, EMBEDDED_HOOK_SH);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn install_skips_when_content_identical() {
        // If the installed file already matches the embedded template, install
        // should report it unchanged (no backup, no rewrite).
        let (cfg, dir) = temp_cfg("install_identical");
        let _ = std::fs::create_dir_all(&cfg.akar_dir);
        // First install writes the templates.
        let first = install_hooks(&cfg, true);
        assert_eq!(first.copied.len(), 2);
        // Second install should find identical content.
        let second = install_hooks(&cfg, true);
        assert!(!second.cancelled);
        assert!(
            second.copied.is_empty(),
            "no rewrites when identical: {:?}",
            second.copied
        );
        assert_eq!(second.unchanged.len(), 2, "both should be unchanged");
        assert!(second.backed_up.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn install_backs_up_when_content_differs() {
        // If the installed file differs from the embedded template, install
        // backs it up before overwriting.
        let (cfg, dir) = temp_cfg("install_differs");
        let _ = std::fs::create_dir_all(&cfg.akar_dir);
        let hooks_dir = cfg.akar_dir.join("hooks");
        let _ = std::fs::create_dir_all(&hooks_dir);
        std::fs::write(
            hooks_dir.join("pre-tool-call.sh"),
            "user-modified content\n",
        )
        .unwrap();
        let result = install_hooks(&cfg, true);
        assert!(!result.cancelled);
        assert!(result.backed_up.contains(&"pre-tool-call.sh".to_string()));
        // The file is now the embedded template (overwritten after backup).
        let after = std::fs::read_to_string(hooks_dir.join("pre-tool-call.sh")).unwrap();
        assert_eq!(after, EMBEDDED_HOOK_SH);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn format_hooks_help_contains_key_info() {
        let out = format_hooks_help();
        assert!(out.contains("akar hooks"));
        assert!(out.contains("pre-tool-call.sh"));
        assert!(out.contains("pre-tool-call.ps1"));
        assert!(out.contains("does not install hooks into Claude Code automatically"));
        assert!(
            out.contains("akar hooks --install"),
            "help must mention the install command"
        );
        assert!(
            out.contains("~/.claude/settings.json"),
            "help must mention manual settings wiring"
        );
    }

    // -- hook JSON parsing ----------------------------------------------------

    const BASH_CARGO_TEST: &str = r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test"}}"#;
    const BASH_RM_RF: &str = r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}"#;
    const NON_BASH_READ: &str = r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"/foo"}}"#;
    const BASH_NO_COMMAND: &str = r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{}}"#;
    const BASH_CARGO_TEST_WITH_CWD: &str = r#"{"session_id":"test","cwd":"/home/user/my-project","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test"}}"#;
    const BASH_RM_RF_WITH_CWD: &str = r#"{"session_id":"test","cwd":"/tmp/target-repo","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"rm -rf /"}}"#;
    const BASH_NO_CWD: &str = r#"{"session_id":"test","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"npm test"}}"#;

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

    // -- v0.29.0: cwd extraction from hook JSON --------------------------------

    #[test]
    fn parse_hook_event_extracts_cwd_when_present() {
        let e = parse_hook_event(BASH_CARGO_TEST_WITH_CWD);
        assert_eq!(e.tool_name, "Bash");
        assert_eq!(e.command, "cargo test");
        assert_eq!(e.cwd, "/home/user/my-project");
    }

    #[test]
    fn parse_hook_event_cwd_empty_when_absent() {
        let e = parse_hook_event(BASH_NO_CWD);
        assert_eq!(e.tool_name, "Bash");
        assert_eq!(e.command, "npm test");
        assert_eq!(e.cwd, "");
    }

    #[test]
    fn parse_hook_event_cwd_does_not_affect_safety_rm_rf() {
        let e = parse_hook_event(BASH_RM_RF_WITH_CWD);
        assert_eq!(e.tool_name, "Bash");
        assert_eq!(e.command, "rm -rf /");
        assert_eq!(e.cwd, "/tmp/target-repo");
        // Safety classification must still work — rm -rf / is still BLOCKED
        if let HookDecision::Check(cmd) = hook_decision(&e) {
            let a = crate::safety::classify_command(&cmd);
            assert!(a.blocked, "rm -rf / must still be BLOCKED with cwd present");
        } else {
            panic!("expected Check decision");
        }
    }

    #[test]
    fn parse_hook_event_cwd_does_not_affect_safety_cargo_test() {
        let e = parse_hook_event(BASH_CARGO_TEST_WITH_CWD);
        assert_eq!(e.tool_name, "Bash");
        assert_eq!(e.command, "cargo test");
        assert_eq!(e.cwd, "/home/user/my-project");
        if let HookDecision::Check(cmd) = hook_decision(&e) {
            let a = crate::safety::classify_command(&cmd);
            assert!(
                !a.blocked,
                "cargo test must still be ALLOWed with cwd present"
            );
        } else {
            panic!("expected Check decision");
        }
    }

    #[test]
    fn parse_hook_event_non_bash_skip_unaffected_by_cwd() {
        // Non-Bash tools still produce Skip regardless of cwd.
        let json = r#"{"session_id":"test","cwd":"/some/path","hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"/foo"}}"#;
        let e = parse_hook_event(json);
        assert_eq!(e.tool_name, "Read");
        assert_eq!(e.cwd, "/some/path");
        assert_eq!(hook_decision(&e), HookDecision::Skip);
    }

    #[test]
    fn hook_decision_non_bash_is_skip() {
        let e = parse_hook_event(NON_BASH_READ);
        assert_eq!(hook_decision(&e), HookDecision::Skip);
    }

    #[test]
    fn hook_decision_bash_cargo_test_is_check() {
        let e = parse_hook_event(BASH_CARGO_TEST);
        assert_eq!(
            hook_decision(&e),
            HookDecision::Check("cargo test".to_string())
        );
    }

    #[test]
    fn hook_decision_bash_rm_rf_is_check() {
        let e = parse_hook_event(BASH_RM_RF);
        assert_eq!(
            hook_decision(&e),
            HookDecision::Check("rm -rf /".to_string())
        );
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
        let templates = find_hook_templates(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        for t in &templates {
            assert!(
                t.content.contains("exit 2"),
                "template {} must use exit 2 for BLOCKED, not exit 1",
                t.name
            );
            // Check that no non-comment line uses exit 1
            for line in t.content.lines() {
                let trimmed = line.trim();
                let is_comment = trimmed.starts_with('#') || trimmed.starts_with("//");
                if !is_comment {
                    assert!(
                        !trimmed.contains("exit 1"),
                        "template {} has executable exit 1 on line: {}",
                        t.name,
                        line
                    );
                }
            }
        }
    }

    #[test]
    fn hook_templates_read_stdin_not_argv() {
        // Templates must read JSON from stdin, not $1 / param
        let templates = find_hook_templates(&std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
        for t in &templates {
            if t.name.ends_with(".sh") {
                assert!(
                    t.content.contains("cat")
                        || t.content.contains("stdin")
                        || t.content.contains("read"),
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
