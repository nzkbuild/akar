//! Learning Patch v0 — generates local, reviewable learning patch proposals
//! from postmortem evidence. Does not auto-apply or mutate global config.

use std::path::Path;
use crate::{config, event_log, postmortem};

// ---------------------------------------------------------------------------
// Learning patch lifecycle (v0.14.0)
// ---------------------------------------------------------------------------

/// The split-rule marker text. A learning patch entry that contains this text
/// in its body can trigger the loop governor's SPLIT_TASK decision — but only
/// while the entry is active (not resolved).
pub const SPLIT_RULE_MARKER: &str = "Next prompt must reduce scope or split the task.";

/// A parsed learning patch entry: its header line (`## LP-...`) and full body
/// (header + bullet lines), plus flags indicating whether it carries the
/// split-rule marker and whether it is resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningPatchEntry {
    /// The `## LP-...` header line.
    pub header: String,
    /// The full entry text, starting at the `## LP-` header and ending before
    /// the next `## LP-` header (or end of file).
    pub body: String,
    /// True if the entry body contains the split-rule marker.
    pub has_split_rule: bool,
    /// True if the entry body contains `status: resolved`.
    pub is_resolved: bool,
}

impl LearningPatchEntry {
    /// True if this entry is active (not resolved). Statusless and
    /// `status: proposed`/`status: active` entries are all considered active.
    pub fn is_active(&self) -> bool {
        !self.is_resolved
    }

    /// True if this entry is an active split-rule entry — the only kind that
    /// can trigger the loop governor's SPLIT_TASK decision.
    pub fn is_active_split_rule(&self) -> bool {
        self.has_split_rule && self.is_active()
    }
}

/// Parse `.akar/LEARNING_PATCHES.md` into individual `## LP-` entries.
///
/// Lines before the first `## LP-` header (the file header and comments) are
/// discarded. Returns an empty vec if the file is absent or unreadable.
/// Never panics.
pub fn parse_entries(patch_path: &Path) -> Vec<LearningPatchEntry> {
    let content = match std::fs::read_to_string(patch_path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let mut entries: Vec<LearningPatchEntry> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut started = false;
    for line in content.lines() {
        if line.starts_with("## LP-") {
            if started {
                let body = current.join("\n");
                entries.push(entry_from_body(body));
            }
            current.clear();
            current.push(line.to_string());
            started = true;
        } else if started {
            current.push(line.to_string());
        }
    }
    if started {
        let body = current.join("\n");
        entries.push(entry_from_body(body));
    }
    entries
}

/// Build a `LearningPatchEntry` from its full body text.
fn entry_from_body(body: String) -> LearningPatchEntry {
    let header = body.lines().next().unwrap_or("").to_string();
    let has_split_rule = body.contains(SPLIT_RULE_MARKER);
    // An entry is resolved only if it explicitly carries `status: resolved`.
    // Any other status (or no status line at all) is treated as active.
    let is_resolved = body.lines().any(|l| line_has_status(l, "resolved"));
    LearningPatchEntry {
        header,
        body,
        has_split_rule,
        is_resolved,
    }
}

/// True if a line declares a status value. Matches both `- status: <value>`
/// (bulleted) and `status: <value>` (unbulleted) forms.
fn line_has_status(line: &str, value: &str) -> bool {
    let t = line.trim();
    let needle = format!("status: {}", value);
    t == needle || t.starts_with(&format!("{} ", needle)) || t == format!("- {}", needle)
}

/// True if `.akar/LEARNING_PATCHES.md` contains at least one ACTIVE split-rule
/// entry. Resolved split-rule entries do NOT count. Used by the loop governor
/// for rule D.
pub fn has_active_split_rule_entry(patch_path: &Path) -> bool {
    parse_entries(patch_path)
        .iter()
        .any(|e| e.is_active_split_rule())
}

/// A summary of the learning patch file used by `akar learn --list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchSummary {
    /// Total number of parsed entries.
    pub total: usize,
    /// Number of active (non-resolved) entries.
    pub active: usize,
    /// Number of resolved entries.
    pub resolved: usize,
    /// Number of active entries carrying the split-rule marker.
    pub active_split_rule: usize,
}

impl PatchSummary {
    /// True if any active split-rule entry can affect the loop governor
    /// (i.e. force SPLIT_TASK).
    pub fn governor_affected(&self) -> bool {
        self.active_split_rule > 0
    }
}

/// Summarise the learning patch file at `patch_path`. Returns a zeroed
/// summary if the file is absent. Never panics.
pub fn summarize_patches(patch_path: &Path) -> PatchSummary {
    let entries = parse_entries(patch_path);
    let total = entries.len();
    let resolved = entries.iter().filter(|e| e.is_resolved).count();
    let active = total - resolved;
    let active_split_rule = entries.iter().filter(|e| e.is_active_split_rule()).count();
    PatchSummary {
        total,
        active,
        resolved,
        active_split_rule,
    }
}

/// Mark every active entry in `.akar/LEARNING_PATCHES.md` as resolved by
/// rewriting the file in place. For each resolved entry, sets `status: resolved`
/// and adds a `resolved_at: <iso8601>` line.
///
/// - Does NOT delete the file.
/// - Does NOT delete any entry.
/// - Does NOT apply patches or modify project source files.
///
/// Returns the number of entries resolved, or `None` if the file does not
/// exist. Never panics.
pub fn resolve_active_patches(patch_path: &Path, now_iso: &str) -> Option<usize> {
    if !patch_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(patch_path).ok()?;
    let lines: Vec<&str> = content.lines().collect();

    // Preserve everything before the first `## LP-` header verbatim.
    let mut preamble: Vec<String> = Vec::new();
    let mut first_entry_idx = lines.len();
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("## LP-") {
            first_entry_idx = i;
            break;
        }
        preamble.push(line.to_string());
    }

    let mut out: Vec<String> = preamble;
    let mut count = 0usize;
    let mut i = first_entry_idx;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("## LP-") {
            // Collect this entry's lines until the next `## LP-` header.
            let mut entry_lines: Vec<String> = vec![line.to_string()];
            let mut j = i + 1;
            while j < lines.len() && !lines[j].starts_with("## LP-") {
                entry_lines.push(lines[j].to_string());
                j += 1;
            }

            let already_resolved = entry_lines
                .iter()
                .any(|l| line_has_status(l, "resolved"));

            if already_resolved {
                out.extend(entry_lines);
            } else {
                let mut rewritten: Vec<String> = Vec::new();
                let mut status_seen = false;
                let mut resolved_at_seen = false;
                for el in &entry_lines {
                    let trimmed = el.trim();
                    let is_status_line = trimmed == "status:"
                        || trimmed.starts_with("status: ")
                        || trimmed == "- status:"
                        || trimmed.starts_with("- status: ");
                    let is_resolved_at_line = trimmed == "resolved_at:"
                        || trimmed.starts_with("resolved_at: ")
                        || trimmed == "- resolved_at:"
                        || trimmed.starts_with("- resolved_at: ");
                    if is_status_line {
                        status_seen = true;
                        rewritten.push("- status: resolved".to_string());
                        if !resolved_at_seen {
                            rewritten.push(format!("- resolved_at: {}", now_iso));
                            resolved_at_seen = true;
                        }
                    } else if is_resolved_at_line {
                        if !resolved_at_seen {
                            rewritten.push(format!("- resolved_at: {}", now_iso));
                            resolved_at_seen = true;
                        }
                        // skip old resolved_at value
                    } else {
                        rewritten.push(el.clone());
                    }
                }
                if !status_seen {
                    rewritten.push("- status: resolved".to_string());
                    if !resolved_at_seen {
                        rewritten.push(format!("- resolved_at: {}", now_iso));
                    }
                }
                out.extend(rewritten);
                count += 1;
            }
            i = j;
        } else {
            out.push(line.to_string());
            i += 1;
        }
    }

    let mut new_content = out.join("\n");
    if content.ends_with('\n') && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    let _ = std::fs::write(patch_path, new_content);
    Some(count)
}

// ---------------------------------------------------------------------------
// LearnResult
// ---------------------------------------------------------------------------

pub enum LearnResult {
    NoTelemetry,
    CleanNoAction,
    PatchProposed { path: std::path::PathBuf, id: String },
    PatchAppended { path: std::path::PathBuf, id: String },
}

// ---------------------------------------------------------------------------
// run_learn
// ---------------------------------------------------------------------------

/// Read postmortem evidence and, if warranted, append a learning patch proposal
/// to `.akar/LEARNING_PATCHES.md`. Never overwrites existing content.
pub fn run_learn(cfg: &config::Config) -> LearnResult {
    let log_path = cfg.akar_dir.join("EVENT_LOG.jsonl");
    let report = postmortem::run_postmortem(&log_path);

    if !report.exists || report.total_events == 0 {
        return LearnResult::NoTelemetry;
    }

    match report.latest_outcome {
        postmortem::Outcome::Clean => LearnResult::CleanNoAction,
        _ => {
            let patch_path = cfg.akar_dir.join("LEARNING_PATCHES.md");
            let id = next_patch_id(&patch_path);
            let patch = build_patch(&id, &report, cfg);
            append_patch(&patch_path, &patch);
            if patch_path.exists() && std::fs::metadata(&patch_path).map(|m| m.len()).unwrap_or(0) > 0 {
                if id == "LP-0001" {
                    LearnResult::PatchProposed { path: patch_path, id }
                } else {
                    LearnResult::PatchAppended { path: patch_path, id }
                }
            } else {
                LearnResult::PatchProposed { path: patch_path, id }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Determine the next patch id by counting existing LP-NNNN entries in the file.
fn next_patch_id(patch_path: &Path) -> String {
    let count = if patch_path.exists() {
        std::fs::read_to_string(patch_path)
            .unwrap_or_default()
            .lines()
            .filter(|l| l.starts_with("## LP-"))
            .count()
    } else {
        0
    };
    format!("LP-{:04}", count + 1)
}

/// Build a learning patch Markdown block from postmortem evidence.
fn build_patch(id: &str, report: &postmortem::PostmortemReport, cfg: &config::Config) -> String {
    let ts = event_log::now_iso8601();
    let outcome = report.latest_outcome.as_str();

    let observed = if let Some(s) = &report.latest_summary {
        config::redact(s)
    } else {
        "no summary available".to_string()
    };

    let failure_type = match report.latest_outcome {
        postmortem::Outcome::Failed => "mission_failure",
        postmortem::Outcome::Degraded => "degraded_run",
        postmortem::Outcome::Unknown => "unknown_outcome",
        postmortem::Outcome::Clean => "none",
    };

    let rule = match report.latest_outcome {
        postmortem::Outcome::Failed => "Investigate failure before retrying. Run doctor and eval.",
        postmortem::Outcome::Degraded => "Check warnings before proceeding. Run doctor if issues persist.",
        postmortem::Outcome::Unknown => "Ensure telemetry is wired correctly. Run a mission to generate evidence.",
        postmortem::Outcome::Clean => "No rule needed.",
    };

    let eval_suggestion = match report.latest_outcome {
        postmortem::Outcome::Failed => "add eval: failed mission recovery path",
        postmortem::Outcome::Degraded => "add eval: degraded mission detection",
        postmortem::Outcome::Unknown => "add eval: unknown outcome detection",
        postmortem::Outcome::Clean => "no eval needed",
    };

    let warnings_note = if report.warnings.is_empty() {
        "none".to_string()
    } else {
        report.warnings.join("; ")
    };

    format!(
        "\n## {id}\n\
        - date: {ts}\n\
        - source: telemetry/postmortem\n\
        - project: {project}\n\
        - outcome: {outcome}\n\
        - observed: {observed}\n\
        - warnings: {warnings}\n\
        - failure_type: {failure_type}\n\
        - rule: {rule}\n\
        - eval_suggestion: {eval_suggestion}\n\
        - status: active\n",
        id = id,
        ts = ts,
        project = cfg.project_name,
        outcome = outcome,
        observed = observed,
        warnings = warnings_note,
        failure_type = failure_type,
        rule = rule,
        eval_suggestion = eval_suggestion,
    )
}

/// Append `content` to `patch_path`, creating the file with a header if new.
fn append_patch(patch_path: &Path, content: &str) {
    let needs_header = !patch_path.exists();
    let header = if needs_header {
        "# AKAR Learning Patches\n\
        <!-- Append-only. Do not delete entries. User edits are preserved. -->\n"
    } else {
        ""
    };

    use std::fs::OpenOptions;
    use std::io::Write;
    if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(patch_path) {
        let _ = write!(f, "{}{}", header, content);
    }
}

// ---------------------------------------------------------------------------
// format_learn_result
// ---------------------------------------------------------------------------

pub fn format_learn_result(result: &LearnResult) -> String {
    match result {
        LearnResult::NoTelemetry => {
            "learn: no telemetry found\n  hint: run 'akar mission \"<prompt>\"' first\n".to_string()
        }
        LearnResult::CleanNoAction => {
            "learn: last mission was clean — no learning patch needed\n".to_string()
        }
        LearnResult::PatchProposed { path, id } => {
            format!(
                "learn: patch proposed\n  id:   {}\n  file: {}\n  hint: review and edit before applying\n",
                id, path.display()
            )
        }
        LearnResult::PatchAppended { path, id } => {
            format!(
                "learn: patch appended\n  id:   {}\n  file: {}\n  hint: review and edit before applying\n",
                id, path.display()
            )
        }
    }
}

/// Format the `akar learn --list` report: counts of active and resolved
/// entries and whether any active split-rule entry can affect the loop
/// governor. Read-only — does not modify files.
pub fn format_patch_list(patch_path: &Path) -> String {
    let summary = summarize_patches(patch_path);
    let mut out = String::new();
    out.push_str("learn --list:\n");
    if !patch_path.exists() {
        out.push_str("  file: (not present)\n");
        out.push_str("  total: 0\n  active: 0\n  resolved: 0\n");
        out.push_str("  active split-rule entries: 0\n");
        out.push_str("  governor affected: no\n");
        return out;
    }
    out.push_str(&format!("  file: {}\n", patch_path.display()));
    out.push_str(&format!("  total: {}\n", summary.total));
    out.push_str(&format!("  active: {}\n", summary.active));
    out.push_str(&format!("  resolved: {}\n", summary.resolved));
    out.push_str(&format!(
        "  active split-rule entries: {}\n",
        summary.active_split_rule
    ));
    out.push_str(&format!(
        "  governor affected: {}\n",
        if summary.governor_affected() { "yes" } else { "no" }
    ));
    if summary.governor_affected() {
        out.push_str(
            "  hint: run 'akar learn --resolve' to retire active split-rule entries\n",
        );
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

    fn tmp_cfg(name: &str) -> (config::Config, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("akar_learn_test_{}", name));
        let _ = fs::create_dir_all(&dir);
        let cfg = config::Config {
            project_root: dir.clone(),
            akar_dir: dir.clone(),
            global_dir: dir.join("global"),
            project_name: "test".to_string(),
        };
        (cfg, dir)
    }

    fn write_mission_event(akar_dir: &Path, event: &str, summary: &str) {
        let log = akar_dir.join("EVENT_LOG.jsonl");
        let entry = event_log::EventEntry {
            ts: "2026-07-04T07:00:00Z".to_string(),
            project: "test".to_string(),
            model: "unknown".to_string(),
            event: event.to_string(),
            event_type: "mission".to_string(),
            summary: summary.to_string(),
            resolution: "done".to_string(),
            confidence: "medium".to_string(),
        };
        event_log::append_event(&log, &entry).unwrap();
    }

    #[test]
    fn learn_handles_missing_telemetry() {
        let (cfg, dir) = tmp_cfg("missing");
        let result = run_learn(&cfg);
        assert!(matches!(result, LearnResult::NoTelemetry));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn learn_no_patch_for_clean_outcome() {
        let (cfg, dir) = tmp_cfg("clean");
        write_mission_event(&dir, "success", "mission/done task=Bugfix risk=Low autonomy=A5 warnings=0 prompt=fix btn");
        let result = run_learn(&cfg);
        assert!(matches!(result, LearnResult::CleanNoAction));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn learn_creates_patch_for_degraded_outcome() {
        let (cfg, dir) = tmp_cfg("degraded");
        write_mission_event(&dir, "success", "mission/done task=Bugfix risk=Low autonomy=A5 warnings=2 prompt=fix btn");
        let result = run_learn(&cfg);
        assert!(matches!(result, LearnResult::PatchProposed { .. } | LearnResult::PatchAppended { .. }));
        let patch_path = dir.join("LEARNING_PATCHES.md");
        assert!(patch_path.exists());
        let content = fs::read_to_string(&patch_path).unwrap();
        assert!(content.contains("LP-0001"));
        assert!(content.contains("degraded"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn learn_creates_patch_for_failed_outcome() {
        let (cfg, dir) = tmp_cfg("failed");
        write_mission_event(&dir, "failure", "mission/failed task=Feature risk=High autonomy=A5 warnings=3 prompt=bad");
        let result = run_learn(&cfg);
        assert!(matches!(result, LearnResult::PatchProposed { .. } | LearnResult::PatchAppended { .. }));
        let content = fs::read_to_string(dir.join("LEARNING_PATCHES.md")).unwrap();
        assert!(content.contains("mission_failure"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn learn_creates_patch_for_unknown_outcome() {
        let (cfg, dir) = tmp_cfg("unknown");
        write_mission_event(&dir, "info", "mission/unknown task=Feature risk=Low autonomy=A5 warnings=0 prompt=odd");
        let result = run_learn(&cfg);
        // unknown outcome → patch
        assert!(matches!(result, LearnResult::PatchProposed { .. } | LearnResult::PatchAppended { .. } | LearnResult::CleanNoAction));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn learn_appends_without_overwriting() {
        let (cfg, dir) = tmp_cfg("append");
        // Two failed events → two patches
        write_mission_event(&dir, "failure", "mission/failed task=Bugfix risk=Low autonomy=A5 warnings=1 prompt=a");
        let _ = run_learn(&cfg);
        write_mission_event(&dir, "failure", "mission/failed task=Bugfix risk=Low autonomy=A5 warnings=2 prompt=b");
        let _ = run_learn(&cfg);
        let content = fs::read_to_string(dir.join("LEARNING_PATCHES.md")).unwrap();
        assert!(content.contains("LP-0001"));
        assert!(content.contains("LP-0002"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn learn_redacts_secrets() {
        let (cfg, dir) = tmp_cfg("redact");
        write_mission_event(&dir, "failure", "mission/failed prompt=token=sk-abc123secret warnings=1");
        let _ = run_learn(&cfg);
        let content = fs::read_to_string(dir.join("LEARNING_PATCHES.md")).unwrap();
        assert!(!content.contains("sk-abc123"), "secret should be redacted in patch");
        assert!(content.contains("[REDACTED]"));
        let _ = fs::remove_dir_all(&dir);
    }

    // -- v0.14.0 learning patch lifecycle tests ---------------------------

    fn write_patches_file(dir: &Path, contents: &str) {
        fs::write(dir.join("LEARNING_PATCHES.md"), contents).unwrap();
    }

    fn split_rule_body(status_line: Option<&str>) -> String {
        let mut s = String::from("## LP-0001\n");
        s.push_str("- date: 2026-07-06T00:00:00Z\n");
        s.push_str("- source: postmortem --diff\n");
        s.push_str("- rule: Next prompt must reduce scope or split the task.\n");
        if let Some(st) = status_line {
            s.push_str(&format!("- {}\n", st));
        }
        s
    }

    #[test]
    fn old_statusless_split_rule_entry_is_treated_as_active() {
        let (cfg, dir) = tmp_cfg("lifecycle_statusless");
        write_patches_file(&dir, &split_rule_body(None));
        let path = dir.join("LEARNING_PATCHES.md");
        let entries = parse_entries(&path);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].has_split_rule);
        assert!(!entries[0].is_resolved);
        assert!(entries[0].is_active());
        assert!(entries[0].is_active_split_rule());
        assert!(has_active_split_rule_entry(&path));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn active_split_rule_triggers_governor_check() {
        let (cfg, dir) = tmp_cfg("lifecycle_active");
        write_patches_file(&dir, &split_rule_body(Some("status: active")));
        let path = dir.join("LEARNING_PATCHES.md");
        assert!(has_active_split_rule_entry(&path));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolved_split_rule_does_not_trigger_governor_check() {
        let (cfg, dir) = tmp_cfg("lifecycle_resolved");
        let mut body = split_rule_body(Some("status: resolved"));
        body.push_str("- resolved_at: 2026-07-06T00:00:00Z\n");
        write_patches_file(&dir, &body);
        let path = dir.join("LEARNING_PATCHES.md");
        let entries = parse_entries(&path);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].has_split_rule);
        assert!(entries[0].is_resolved);
        assert!(!entries[0].is_active_split_rule());
        assert!(!has_active_split_rule_entry(&path));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn proposed_status_is_treated_as_active() {
        // Existing v0.13.0 entries used `status: proposed`; these must remain
        // active until explicitly resolved.
        let (cfg, dir) = tmp_cfg("lifecycle_proposed");
        write_patches_file(&dir, &split_rule_body(Some("status: proposed")));
        let path = dir.join("LEARNING_PATCHES.md");
        assert!(has_active_split_rule_entry(&path));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn new_exceeded_budget_patch_includes_status_active() {
        // learn.rs build_patch now writes `status: active` for new patches.
        let (cfg, dir) = tmp_cfg("lifecycle_newpatch");
        write_mission_event(&dir, "failure", "mission/failed task=Bugfix risk=Low autonomy=A5 warnings=1 prompt=a");
        let _ = run_learn(&cfg);
        let content = fs::read_to_string(dir.join("LEARNING_PATCHES.md")).unwrap();
        assert!(content.contains("- status: active"), "new patch must include status: active");
        assert!(!content.contains("- status: proposed"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_counts_active_and_resolved_entries() {
        let (cfg, dir) = tmp_cfg("lifecycle_list");
        let mut content = String::new();
        content.push_str("# AKAR Learning Patches\n<!-- Append-only. -->\n");
        // active split-rule
        content.push_str(&split_rule_body(Some("status: active")));
        // resolved split-rule
        content.push_str("## LP-0002\n");
        content.push_str("- rule: Next prompt must reduce scope or split the task.\n");
        content.push_str("- status: resolved\n");
        content.push_str("- resolved_at: 2026-07-06T00:00:00Z\n");
        // active non-split
        content.push_str("## LP-0003\n");
        content.push_str("- rule: Investigate failure before retrying.\n");
        content.push_str("- status: active\n");
        write_patches_file(&dir, &content);
        let path = dir.join("LEARNING_PATCHES.md");
        let summary = summarize_patches(&path);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.active, 2);
        assert_eq!(summary.resolved, 1);
        assert_eq!(summary.active_split_rule, 1);
        assert!(summary.governor_affected());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_marks_active_entries_resolved() {
        let (cfg, dir) = tmp_cfg("lifecycle_resolve");
        let mut content = String::new();
        content.push_str("# AKAR Learning Patches\n<!-- Append-only. -->\n");
        content.push_str(&split_rule_body(Some("status: active")));
        content.push_str("## LP-0002\n");
        content.push_str("- rule: Next prompt must reduce scope or split the task.\n");
        content.push_str("- status: proposed\n");
        write_patches_file(&dir, &content);
        let path = dir.join("LEARNING_PATCHES.md");
        let count = resolve_active_patches(&path, "2026-07-06T12:00:00Z");
        assert_eq!(count, Some(2));
        let after = fs::read_to_string(&path).unwrap();
        // Both entries now resolved.
        assert_eq!(after.matches("status: resolved").count(), 2);
        assert!(!after.contains("status: active"));
        assert!(!after.contains("status: proposed"));
        // No active split-rule entries remain.
        assert!(!has_active_split_rule_entry(&path));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_adds_resolved_timestamp() {
        let (cfg, dir) = tmp_cfg("lifecycle_resolve_ts");
        write_patches_file(&dir, &split_rule_body(Some("status: active")));
        let path = dir.join("LEARNING_PATCHES.md");
        let _ = resolve_active_patches(&path, "2026-07-06T12:34:56Z");
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("- resolved_at: 2026-07-06T12:34:56Z"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_does_not_delete_file() {
        let (cfg, dir) = tmp_cfg("lifecycle_resolve_nodelete");
        write_patches_file(&dir, &split_rule_body(Some("status: active")));
        let path = dir.join("LEARNING_PATCHES.md");
        let _ = resolve_active_patches(&path, "2026-07-06T12:00:00Z");
        assert!(path.exists(), "file must still exist after resolve");
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("## LP-0001"), "entry must still be present");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_no_file_returns_none() {
        let (cfg, dir) = tmp_cfg("lifecycle_resolve_nofile");
        let path = dir.join("LEARNING_PATCHES.md");
        assert!(!path.exists());
        let count = resolve_active_patches(&path, "2026-07-06T12:00:00Z");
        assert_eq!(count, None);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_no_active_entries_returns_zero() {
        let (cfg, dir) = tmp_cfg("lifecycle_resolve_noactive");
        let mut content = split_rule_body(Some("status: resolved"));
        content.push_str("- resolved_at: 2026-07-06T00:00:00Z\n");
        write_patches_file(&dir, &content);
        let path = dir.join("LEARNING_PATCHES.md");
        let count = resolve_active_patches(&path, "2026-07-06T12:00:00Z");
        assert_eq!(count, Some(0));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn resolve_preserves_file_header_and_comments() {
        let (cfg, dir) = tmp_cfg("lifecycle_resolve_header");
        let mut content = String::from("# AKAR Learning Patches\n<!-- Append-only. Do not delete entries. -->\n\n");
        content.push_str(&split_rule_body(Some("status: active")));
        write_patches_file(&dir, &content);
        let path = dir.join("LEARNING_PATCHES.md");
        let _ = resolve_active_patches(&path, "2026-07-06T12:00:00Z");
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.starts_with("# AKAR Learning Patches"));
        assert!(after.contains("Append-only. Do not delete entries."));
        let _ = fs::remove_dir_all(&dir);
    }
}
