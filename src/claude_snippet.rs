/// Managed CLAUDE.md snippet — insert, update, and detect the AKAR session
/// guidance block in a project's CLAUDE.md file.
///
/// The snippet is the exact v0.52 revised wording proven to prevent stale-context
/// failures in fresh AI sessions. `akar init --claude` manages it idempotently.
use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Snippet constants
// ---------------------------------------------------------------------------

/// The exact v0.52 revised AKAR session guidance snippet.
const AKAR_SNIPPET: &str = "\n## AKAR Session Guidance (managed by `akar init`)\n\
\n\
Before starting any coding task, read `.akar/NEXT_RUN.md`.\n\
\n\
Compare the user's current request with the Objective in `.akar/NEXT_RUN.md`.\n\
\n\
If the Objective describes a different task than what the user is asking for, the\n\
AKAR context is stale — it may be from a previous session. Do not edit files or run\n\
project commands. Ask the user to run: `akar prepare \"<current task>\"`\n\
\n\
If the Objective matches the user's request, treat `.akar/NEXT_RUN.md` as the\n\
current task contract: scope, budget, allowed and forbidden commands, required\n\
verification, stop conditions, and governor decision.\n\
\n\
After completing work, verify you stayed within the budget and stop conditions.\n\
The user will run `akar finish`.\n\
<!-- AKAR section ends -->\n";

/// Immutable search anchor that marks the end of the AKAR section.
const AKAR_SECTION_MARKER: &str = "<!-- AKAR section ends -->";

/// Header line that marks the start of the AKAR block.
const AKAR_SECTION_HEADER: &str = "## AKAR Session Guidance";

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// The current state of the AKAR snippet in the project's CLAUDE.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnippetState {
    /// No CLAUDE.md file exists at the resolved path.
    Absent,
    /// CLAUDE.md exists but has no AKAR section marker.
    PresentNoBlock,
    /// AKAR section found and matches the canonical snippet.
    PresentWithBlock,
    /// AKAR section found but content differs from canonical.
    Outdated,
    /// Multiple AKAR section end-markers found in the file.
    Duplicate,
}

/// Result of applying the AKAR snippet to CLAUDE.md.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClaudeSnippetResult {
    /// Path to the CLAUDE.md file that was created or modified.
    pub path: PathBuf,
    /// The action taken: "created", "appended", "replaced", "unchanged", "cancelled".
    pub action: String,
    /// State of CLAUDE.md before the action: "absent", "present_no_block", "present_with_block".
    pub prior_state: String,
    /// Human-readable explanation of what happened.
    pub detail: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Resolve the CLAUDE.md path for a project.
///
/// Priority:
/// 1. `<project_root>/CLAUDE.md` if it exists
/// 2. `<project_root>/.claude/CLAUDE.md` if it exists
/// 3. Default to `<project_root>/CLAUDE.md` (will be created here)
pub fn claude_md_path(project_root: &Path) -> PathBuf {
    let root_md = project_root.join("CLAUDE.md");
    if root_md.exists() {
        return root_md;
    }
    let dot_claude_md = project_root.join(".claude").join("CLAUDE.md");
    if dot_claude_md.exists() {
        return dot_claude_md;
    }
    root_md
}

/// Read-only detection of the AKAR snippet state in CLAUDE.md.
///
/// Never writes, creates, or modifies files.
pub fn detect_snippet_state(project_root: &Path) -> SnippetState {
    let path = claude_md_path(project_root);
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return SnippetState::Absent,
    };

    let count = content.matches(AKAR_SECTION_MARKER).count();
    match count {
        0 => SnippetState::PresentNoBlock,
        1 => {
            if snippet_matches_canonical(&content) {
                SnippetState::PresentWithBlock
            } else {
                SnippetState::Outdated
            }
        }
        _ => SnippetState::Duplicate,
    }
}

/// Apply the AKAR snippet to the project's CLAUDE.md.
///
/// When `confirmed` is false, returns a would-do result without writing.
/// Always backs up existing files before overwriting.
pub fn apply_snippet(project_root: &Path, confirmed: bool) -> ClaudeSnippetResult {
    let path = claude_md_path(project_root);
    let content = fs::read_to_string(&path).ok();

    let (action, prior_state, new_content) = match &content {
        None => {
            // CLAUDE.md doesn't exist — create it.
            if !confirmed {
                return ClaudeSnippetResult {
                    path,
                    action: "cancelled".to_string(),
                    prior_state: "absent".to_string(),
                    detail: "would create CLAUDE.md with AKAR snippet".to_string(),
                };
            }
            ("created", "absent", AKAR_SNIPPET.to_string())
        }
        Some(existing) => {
            let marker_count = existing.matches(AKAR_SECTION_MARKER).count();

            if marker_count == 0 {
                // No AKAR block — append.
                if !confirmed {
                    return ClaudeSnippetResult {
                        path,
                        action: "cancelled".to_string(),
                        prior_state: "present_no_block".to_string(),
                        detail: "would append AKAR snippet to existing CLAUDE.md".to_string(),
                    };
                }
                let mut appended = existing.clone();
                // Ensure new content doesn't collide with existing trailing newlines.
                if !appended.ends_with('\n') {
                    appended.push('\n');
                }
                // AKAR_SNIPPET starts with \n, skip it when appending so we
                // don't get a double blank line.
                appended.push_str(AKAR_SNIPPET.trim_start_matches('\n'));
                ("appended", "present_no_block", appended)
            } else {
                // Has at least one marker — try to replace the first block.
                if snippet_matches_canonical(existing) {
                    return ClaudeSnippetResult {
                        path,
                        action: "unchanged".to_string(),
                        prior_state: "present_with_block".to_string(),
                        detail: "AKAR snippet is already up to date".to_string(),
                    };
                }
                if !confirmed {
                    let ps = if marker_count > 1 {
                        "present_with_block (duplicate)"
                    } else {
                        "present_with_block"
                    };
                    return ClaudeSnippetResult {
                        path,
                        action: "cancelled".to_string(),
                        prior_state: ps.to_string(),
                        detail: "would replace outdated AKAR snippet in CLAUDE.md".to_string(),
                    };
                }
                let replaced = replace_akar_block(existing);
                ("replaced", "present_with_block", replaced)
            }
        }
    };

    // Write with backup if file already exists.
    if content.is_some() {
        if let Err(e) = crate::backup::backup_file(&path) {
            return ClaudeSnippetResult {
                path,
                action: "failed".to_string(),
                prior_state: prior_state.to_string(),
                detail: format!("backup failed: {}", e),
            };
        }
    }

    match fs::write(&path, &new_content) {
        Ok(_) => ClaudeSnippetResult {
            path,
            action: action.to_string(),
            prior_state: prior_state.to_string(),
            detail: match action {
                "created" => "created CLAUDE.md with AKAR session guidance snippet".to_string(),
                "appended" => {
                    "appended AKAR snippet to existing CLAUDE.md (user content preserved)"
                        .to_string()
                }
                "replaced" => {
                    if content.map_or(0, |c| c.matches(AKAR_SECTION_MARKER).count()) > 1 {
                        "replaced first AKAR block; found duplicate markers — review CLAUDE.md"
                            .to_string()
                    } else {
                        "replaced outdated AKAR snippet with updated version".to_string()
                    }
                }
                _ => "done".to_string(),
            },
        },
        Err(e) => ClaudeSnippetResult {
            path,
            action: "failed".to_string(),
            prior_state: prior_state.to_string(),
            detail: format!("write failed: {}", e),
        },
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Check whether the existing CLAUDE.md content contains the canonical snippet.
fn snippet_matches_canonical(content: &str) -> bool {
    // Compare the canonical snippet body (without leading/trailing newlines).
    let canonical_body = AKAR_SNIPPET.trim();
    content.contains(canonical_body)
}

/// Replace the AKAR block in existing content with the canonical snippet.
///
/// Handles:
/// - Single marker: replace from `## AKAR Session Guidance` header to end-marker.
/// - Multiple markers: replace first block only, preserving content after the
///   first end-marker (which may include duplicate blocks).
/// - Corrupt: marker exists but no header found → fall back to append.
fn replace_akar_block(existing: &str) -> String {
    let first_marker_pos = match existing.find(AKAR_SECTION_MARKER) {
        Some(p) => p,
        None => {
            // Shouldn't happen (caller checks marker_count > 0), but be safe.
            let mut out = existing.to_string();
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(AKAR_SNIPPET.trim_start_matches('\n'));
            return out;
        }
    };

    // Find the section header before the first marker.
    let before_marker = &existing[..first_marker_pos];
    let header_pos = before_marker.rfind(AKAR_SECTION_HEADER);

    let header_pos = match header_pos {
        Some(p) => p,
        None => {
            // Corrupt block — marker exists but no header. Fall back to append.
            let mut out = existing.to_string();
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(AKAR_SNIPPET.trim_start_matches('\n'));
            return out;
        }
    };

    // Everything before the AKAR header stays.
    let prefix = &existing[..header_pos];

    // If there's whitespace or blank lines between the start of the AKAR
    // header and the line above, trim trailing whitespace but keep at most
    // one blank line before the snippet.
    let prefix = prefix.trim_end();

    // Everything after the first end-marker.
    // The marker line is "<!-- AKAR section ends -->" — find the newline after it.
    let after_marker_start = first_marker_pos + AKAR_SECTION_MARKER.len();
    let suffix = &existing[after_marker_start..];

    // If there were duplicate markers, preserve everything after the first
    // marker block (the user can clean up duplicates manually).
    // Strip up to one newline after the marker to avoid blank-line stacking.
    let suffix = if let Some(rest) = suffix.strip_prefix("\r\n") {
        rest
    } else if let Some(rest) = suffix.strip_prefix('\n') {
        rest
    } else {
        suffix
    };

    let mut result = String::with_capacity(prefix.len() + AKAR_SNIPPET.len() + suffix.len() + 2);
    result.push_str(prefix);
    result.push('\n');
    // AKAR_SNIPPET already starts with \n, so don't double it if prefix ends
    // with a blank line (trim_end already handled that).
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result.push_str(AKAR_SNIPPET.trim_start_matches('\n'));
    if !suffix.is_empty() {
        // Only add newline before suffix if it doesn't start with one.
        if !suffix.starts_with('\n') && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(suffix);
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process;

    fn temp_project(label: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("akar_claude_snippet_{}_{}", label, process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn claude_md_path_prefers_root() {
        let dir = temp_project("prefers_root");
        fs::write(dir.join("CLAUDE.md"), "# Project\n").unwrap();
        fs::create_dir_all(dir.join(".claude")).unwrap();
        fs::write(dir.join(".claude/CLAUDE.md"), "# Dot Claude\n").unwrap();

        let path = claude_md_path(&dir);
        assert_eq!(path, dir.join("CLAUDE.md"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn claude_md_path_falls_back_to_dot_claude() {
        let dir = temp_project("dot_claude_fallback");
        fs::create_dir_all(dir.join(".claude")).unwrap();
        fs::write(dir.join(".claude/CLAUDE.md"), "# Dot Claude\n").unwrap();

        let path = claude_md_path(&dir);
        assert_eq!(path, dir.join(".claude/CLAUDE.md"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn claude_md_path_default() {
        let dir = temp_project("default");
        // No CLAUDE.md anywhere.
        let path = claude_md_path(&dir);
        assert_eq!(path, dir.join("CLAUDE.md"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detect_snippet_state_all_variants() {
        let dir = temp_project("detect_state");

        // Absent
        assert_eq!(detect_snippet_state(&dir), SnippetState::Absent);

        // PresentNoBlock
        fs::write(dir.join("CLAUDE.md"), "# No AKAR here\n").unwrap();
        assert_eq!(detect_snippet_state(&dir), SnippetState::PresentNoBlock);

        // PresentWithBlock
        fs::write(
            dir.join("CLAUDE.md"),
            format!("# Project\n{}", AKAR_SNIPPET),
        )
        .unwrap();
        assert_eq!(detect_snippet_state(&dir), SnippetState::PresentWithBlock);

        // Outdated — different content with marker
        fs::write(
            dir.join("CLAUDE.md"),
            "# Project\n## AKAR Session Guidance (managed by `akar init`)\nold content\n<!-- AKAR section ends -->\n",
        )
        .unwrap();
        assert_eq!(detect_snippet_state(&dir), SnippetState::Outdated);

        // Duplicate — two markers
        fs::write(
            dir.join("CLAUDE.md"),
            format!("# Project\n{}<!-- AKAR section ends -->\n", AKAR_SNIPPET),
        )
        .unwrap();
        assert_eq!(detect_snippet_state(&dir), SnippetState::Duplicate);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_snippet_creates_new_file() {
        let dir = temp_project("create_new");
        let result = apply_snippet(&dir, true);

        assert_eq!(result.action, "created");
        assert_eq!(result.prior_state, "absent");
        assert!(result.path.exists());
        let content = fs::read_to_string(&result.path).unwrap();
        assert!(content.contains(AKAR_SECTION_MARKER));
        assert!(content.contains("Before starting any coding task"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_snippet_appends_to_existing() {
        let dir = temp_project("append_existing");
        let original = "# My Project\n\nSome user content\n";
        fs::write(dir.join("CLAUDE.md"), original).unwrap();

        let result = apply_snippet(&dir, true);
        assert_eq!(result.action, "appended");
        assert_eq!(result.prior_state, "present_no_block");

        let content = fs::read_to_string(&result.path).unwrap();
        assert!(content.starts_with(original.trim_end()));
        assert!(content.contains(AKAR_SECTION_MARKER));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_snippet_replaces_outdated() {
        let dir = temp_project("replace_outdated");
        let outdated = "# Project\n## AKAR Session Guidance (managed by `akar init`)\nold text\n<!-- AKAR section ends -->\n";
        fs::write(dir.join("CLAUDE.md"), outdated).unwrap();

        let result = apply_snippet(&dir, true);
        assert_eq!(result.action, "replaced");

        let content = fs::read_to_string(&result.path).unwrap();
        assert!(!content.contains("old text"));
        assert!(content.contains("Before starting any coding task"));
        assert!(content.contains(AKAR_SECTION_MARKER));
        assert_eq!(content.matches(AKAR_SECTION_MARKER).count(), 1);

        // Backup was created.
        let bak = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_str().unwrap().contains(".bak."));
        assert!(bak.is_some(), "backup should exist");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_snippet_idempotent() {
        let dir = temp_project("idempotent");
        let result1 = apply_snippet(&dir, true);
        assert_eq!(result1.action, "created");

        let result2 = apply_snippet(&dir, true);
        assert_eq!(result2.action, "unchanged");
        assert!(result2.detail.contains("already up to date"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_snippet_cancelled_when_not_confirmed() {
        let dir = temp_project("cancelled");
        let result = apply_snippet(&dir, false);
        assert_eq!(result.action, "cancelled");
        assert!(
            !result.path.exists(),
            "should not create file when cancelled"
        );

        // Also test with existing file.
        fs::write(dir.join("CLAUDE.md"), "# Existing\n").unwrap();
        let result2 = apply_snippet(&dir, false);
        assert_eq!(result2.action, "cancelled");
        let content = fs::read_to_string(dir.join("CLAUDE.md")).unwrap();
        assert_eq!(content, "# Existing\n");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_snippet_preserves_user_content() {
        let dir = temp_project("preserve_content");
        let original = "# My Project\n\n## Setup\nRun `npm install`.\n\n## AKAR Session Guidance (managed by `akar init`)\nstale stuff\n<!-- AKAR section ends -->\n\n## License\nMIT\n";
        fs::write(dir.join("CLAUDE.md"), original).unwrap();

        let result = apply_snippet(&dir, true);
        assert_eq!(result.action, "replaced");

        let content = fs::read_to_string(&result.path).unwrap();
        assert!(content.contains("# My Project"));
        assert!(content.contains("## Setup"));
        assert!(content.contains("Run `npm install`"));
        assert!(content.contains("## License"));
        assert!(content.contains("MIT"));
        assert!(!content.contains("stale stuff"));
        assert!(content.contains("Before starting any coding task"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_snippet_handles_corrupt_marker() {
        let dir = temp_project("corrupt_marker");
        // Marker exists but no "## AKAR Session Guidance" header before it.
        let corrupt = "# Project\nSome text\n<!-- AKAR section ends -->\nMore text\n";
        fs::write(dir.join("CLAUDE.md"), corrupt).unwrap();

        let result = apply_snippet(&dir, true);
        // Should fall back to append mode.
        assert!(result.action == "appended" || result.action == "replaced");

        let content = fs::read_to_string(&result.path).unwrap();
        assert!(content.contains("# Project"));
        assert!(content.contains("Before starting any coding task"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_snippet_backs_up_before_overwrite() {
        let dir = temp_project("backup_overwrite");
        fs::write(dir.join("CLAUDE.md"), "# Original\n").unwrap();

        let result = apply_snippet(&dir, true);
        assert!(
            !result.action.contains("failed"),
            "action should not fail: {}",
            result.detail
        );

        let entries: Vec<_> = fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).collect();

        let bak_count = entries
            .iter()
            .filter(|e| e.file_name().to_str().unwrap().contains(".bak."))
            .count();
        assert!(bak_count >= 1, "should have at least one backup file");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_snippet_duplicate_markers_replaced() {
        let dir = temp_project("dup_markers");
        let duplicated = format!("# Project\n{}user content\n", AKAR_SNIPPET);
        fs::write(dir.join("CLAUDE.md"), &duplicated).unwrap();

        let result = apply_snippet(&dir, true);
        // It should handle the duplicate case.
        assert!(
            !result.action.contains("failed"),
            "should handle duplicates: {}",
            result.detail
        );

        let content = fs::read_to_string(&result.path).unwrap();
        // The user content between/after markers should survive.
        assert!(content.contains("user content"));

        let _ = fs::remove_dir_all(&dir);
    }
}
