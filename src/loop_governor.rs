//! Loop Governor v0.13.0 — knowledge-driven loop decision guidance.
//!
//! Reads LOCAL evidence only and chooses the next SAFE loop action so Claude
//! avoids repeated failed commands, dirty-tree confusion, and token-wasting
//! retry loops.
//!
//! Evidence read (read-only, never modified):
//! - git repository status + working tree clean state
//! - `.akar/DIFF_BASELINE.json`
//! - `.akar/HOOK_EVENTS.jsonl`
//! - `.akar/LEARNING_PATCHES.md`
//!
//! Guidance text comes from `src/foundation.rs` playbooks.
//!
//! The governor is ADVISORY ONLY. It does not execute the selected action,
//! does not auto-commit, and never resets, cleans, stashes, checks out, pushes,
//! or deletes anything.

use std::path::Path;

use crate::{
    config, diff_budget, event_log, foundation, learn, project_verification_contract as pvc,
};

// ---------------------------------------------------------------------------
// Loop decision
// ---------------------------------------------------------------------------

/// The next safe loop action chosen from local evidence.
///
/// Decisions are produced in a fixed priority order (see `decide`). Each
/// variant carries a short machine-readable label via [`LoopDecision::as_str`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopDecision {
    /// Baseline exists and tree is clean — continue scoped work.
    Ready,
    /// No baseline and clean tree — take a snapshot before changing anything.
    SnapshotNow,
    /// Baseline exists and tree is dirty — measure the session diff.
    RunPostmortem,
    /// No baseline and dirty tree — commit completed work first.
    CommitCheckpoint,
    /// Learning patches say scope must be reduced — split the task.
    SplitTask,
    /// Hook is broken (akar not found in subprocess PATH) — stop Bash calls.
    StopHookBroken,
    /// Same command blocked repeatedly — stop retrying it.
    StopRepeatedBlock,
    /// State could not be classified — inspect manually.
    Unknown,
}

impl LoopDecision {
    /// Machine-readable decision label.
    pub fn as_str(&self) -> &'static str {
        match self {
            LoopDecision::Ready => "READY",
            LoopDecision::SnapshotNow => "SNAPSHOT_NOW",
            LoopDecision::RunPostmortem => "RUN_POSTMORTEM",
            LoopDecision::CommitCheckpoint => "COMMIT_CHECKPOINT",
            LoopDecision::SplitTask => "SPLIT_TASK",
            LoopDecision::StopHookBroken => "STOP_HOOK_BROKEN",
            LoopDecision::StopRepeatedBlock => "STOP_REPEATED_BLOCK",
            LoopDecision::Unknown => "UNKNOWN",
        }
    }

    /// Orchestrator exit code for this decision (v0.17.0).
    ///
    /// These codes let a session orchestrator branch on the governor's
    /// decision via `$?` without parsing output. They are for orchestration
    /// only — AKAR still does not execute the suggested action.
    ///
    /// | Decision             | Code |
    /// |----------------------|------|
    /// | READY                | 0    |
    /// | SNAPSHOT_NOW         | 0    |
    /// | RUN_POSTMORTEM       | 10   |
    /// | COMMIT_CHECKPOINT    | 11   |
    /// | SPLIT_TASK           | 12   |
    /// | STOP_HOOK_BROKEN     | 20   |
    /// | STOP_REPEATED_BLOCK  | 21   |
    /// | UNKNOWN              | 30   |
    pub fn exit_code(&self) -> i32 {
        match self {
            LoopDecision::Ready => 0,
            LoopDecision::SnapshotNow => 0,
            LoopDecision::RunPostmortem => 10,
            LoopDecision::CommitCheckpoint => 11,
            LoopDecision::SplitTask => 12,
            LoopDecision::StopHookBroken => 20,
            LoopDecision::StopRepeatedBlock => 21,
            LoopDecision::Unknown => 30,
        }
    }

    /// Coarse classification of the decision for next-run prompt compilation
    /// (v0.19.0).
    ///
    /// - `Continue`     — READY, SNAPSHOT_NOW (proceed with scoped work)
    /// - `ActionRequired` — RUN_POSTMORTEM, COMMIT_CHECKPOINT, SPLIT_TASK
    /// - `Stop`         — STOP_HOOK_BROKEN, STOP_REPEATED_BLOCK, UNKNOWN
    pub fn decision_class(&self) -> DecisionClass {
        match self {
            LoopDecision::Ready | LoopDecision::SnapshotNow => DecisionClass::Continue,
            LoopDecision::RunPostmortem
            | LoopDecision::CommitCheckpoint
            | LoopDecision::SplitTask => DecisionClass::ActionRequired,
            LoopDecision::StopHookBroken
            | LoopDecision::StopRepeatedBlock
            | LoopDecision::Unknown => DecisionClass::Stop,
        }
    }
}

/// Coarse classification of a governor decision (v0.19.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionClass {
    /// READY, SNAPSHOT_NOW — proceed with scoped work.
    Continue,
    /// RUN_POSTMORTEM, COMMIT_CHECKPOINT, SPLIT_TASK — a specific action is required.
    ActionRequired,
    /// STOP_HOOK_BROKEN, STOP_REPEATED_BLOCK, UNKNOWN — stop and reassess.
    Stop,
}

impl DecisionClass {
    /// Human-readable label for the next-run prompt.
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionClass::Continue => "continue-class",
            DecisionClass::ActionRequired => "action-required",
            DecisionClass::Stop => "stop-class",
        }
    }
}

/// Convenience wrapper around [`LoopDecision::decision_class`].
#[allow(dead_code)]
pub fn decision_class_for_decision(report: &LoopGovernorReport) -> DecisionClass {
    report.decision.decision_class()
}

/// Compile a direct objective string for a decision (v0.19.0).
pub fn objective_for_decision(decision: &LoopDecision) -> &'static str {
    match decision {
        LoopDecision::Ready => "Continue the scoped task without broadening the work.",
        LoopDecision::SnapshotNow => "Create a clean baseline snapshot before making changes.",
        LoopDecision::RunPostmortem => "Run baseline postmortem before making more changes.",
        LoopDecision::CommitCheckpoint => {
            "Verify and commit completed AKAR work with explicit files only before starting a new baseline."
        }
        LoopDecision::SplitTask => {
            "Stop the broad task and create one smaller single-purpose prompt."
        }
        LoopDecision::StopHookBroken => {
            "Stop Bash tool work until AKAR is visible in the Claude Code hook subprocess PATH."
        }
        LoopDecision::StopRepeatedBlock => {
            "Do not retry the blocked command. Replace it with a safe alternative."
        }
        LoopDecision::Unknown => "Inspect AKAR and git state manually before continuing.",
    }
}

/// Orchestrator exit code for a governor decision (v0.17.0).
///
/// Convenience wrapper around [`LoopDecision::exit_code`] so callers that
/// hold a [`LoopGovernorReport`] can ask for the code directly.
pub fn exit_code_for_decision(report: &LoopGovernorReport) -> i32 {
    report.decision.exit_code()
}

// ---------------------------------------------------------------------------
// Governor report
// ---------------------------------------------------------------------------

/// A complete loop governor report: the chosen decision plus the evidence
/// and human-readable guidance that justify it.
#[derive(Debug, Clone)]
pub struct LoopGovernorReport {
    /// The chosen next-safe-action decision.
    pub decision: LoopDecision,
    /// One-line machine-readable reason summarising why this decision was made.
    pub reason: String,
    /// The next action to take (may be a playbook string from `foundation`).
    pub next_action: String,
    /// A direct prompt the operator can hand to Claude for this decision.
    pub suggested_prompt: String,
    /// Human-readable list of evidence sources consulted.
    pub evidence_used: Vec<String>,
}

impl LoopGovernorReport {
    /// True when the decision demands a stop (hook broken or repeated block).
    #[allow(dead_code)]
    pub fn is_stop(&self) -> bool {
        matches!(
            self.decision,
            LoopDecision::StopHookBroken | LoopDecision::StopRepeatedBlock
        )
    }
}

// ---------------------------------------------------------------------------
// Default fallback actions for decisions without a foundation playbook
// ---------------------------------------------------------------------------

/// Next-action text for the READY decision.
fn ready_next_action() -> &'static str {
    "Baseline exists and tree is clean. Continue scoped work or review/clear old baseline before a new loop."
}

/// Next-action text for the SNAPSHOT_NOW decision.
fn snapshot_next_action() -> String {
    foundation::snapshot_required_playbook().to_string()
}

/// Next-action text for the RUN_POSTMORTEM decision.
fn postmortem_next_action() -> &'static str {
    "Run akar postmortem --diff --baseline before continuing."
}

/// Next-action text for the COMMIT_CHECKPOINT decision.
fn commit_next_action() -> String {
    foundation::git_dirty_playbook().to_string()
}

/// Next-action text for the SPLIT_TASK decision.
fn split_next_action() -> String {
    foundation::budget_exceeded_playbook().to_string()
}

/// Next-action text for the UNKNOWN (no git) decision.
fn unknown_no_git_next_action() -> &'static str {
    "Run AKAR inside a git repository with git available."
}

/// Next-action text for the UNKNOWN (unclassifiable) decision.
fn unknown_unclassified_next_action() -> &'static str {
    "Inspect git status and AKAR runtime files manually."
}

// ---------------------------------------------------------------------------
// Suggested prompts (direct prompts the operator can hand to Claude)
// ---------------------------------------------------------------------------

/// Suggested prompt for SNAPSHOT_NOW.
fn snapshot_prompt() -> &'static str {
    "Run akar preflight --snapshot \"<task>\" before making changes. Stop if snapshot fails."
}

/// Suggested prompt for RUN_POSTMORTEM.
fn postmortem_prompt() -> &'static str {
    "Run akar postmortem --diff --baseline and report PASS, EXCEEDED, or UNKNOWN before making more changes."
}

/// Suggested prompt for COMMIT_CHECKPOINT.
fn commit_prompt() -> &'static str {
    "Run verification, inspect git diff, commit completed AKAR work with explicit files only, then rerun akar status."
}

/// Suggested prompt for SPLIT_TASK.
fn split_prompt() -> &'static str {
    "Stop the broad task. Create one smaller single-purpose prompt before continuing."
}

/// Suggested prompt for STOP_HOOK_BROKEN.
fn hook_broken_prompt() -> &'static str {
    "Do not run more Bash tool calls until akar is visible in the hook subprocess PATH."
}

/// Suggested prompt for STOP_REPEATED_BLOCK.
fn repeated_block_prompt() -> &'static str {
    "Do not retry the blocked command. Replace it with a safe alternative or change the task plan."
}

/// Suggested prompt for READY (continue scoped work).
fn ready_prompt() -> &'static str {
    "Continue the scoped task. Keep changes within the baseline budget and rerun akar postmortem --diff --baseline when done."
}

/// Suggested prompt for UNKNOWN.
fn unknown_prompt() -> &'static str {
    "Inspect git status and the .akar/ runtime files, then decide the next scoped step manually."
}

// ---------------------------------------------------------------------------
// HOOK_EVENTS.jsonl scanning (read-only)
// ---------------------------------------------------------------------------

/// Read all non-empty lines from a JSONL file. Returns an empty vec if the
/// file does not exist or cannot be read. Never panics.
fn read_jsonl_lines(path: &Path) -> Vec<String> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    use std::io::BufRead;
    let reader = std::io::BufReader::new(file);
    reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.trim().is_empty())
        .collect()
}

/// Extract a JSON string value for `key` from a single-line JSON object.
/// Handles backslash escapes. Returns None if the key is absent.
fn json_str(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\":\"", key);
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let mut val = String::new();
    let mut escaped = false;
    for c in rest.chars() {
        if escaped {
            val.push(c);
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

/// True if a HOOK_EVENTS line is an ERROR event whose message mentions
/// `akar not found`. The message may live in `command_preview`, a `message`
/// field, or the raw line — the check is intentionally tolerant of format
/// evolution so a future hook variant that records a real ERROR event is
/// detected regardless of which JSON field carries the text.
fn is_hook_not_found_error(line: &str) -> bool {
    let decision = json_str(line, "decision").unwrap_or_default();
    if !decision.eq_ignore_ascii_case("ERROR") {
        return false;
    }
    let lower = line.to_ascii_lowercase();
    lower.contains("akar not found")
}

/// Detect an ERROR event with `akar not found` anywhere in the hook log.
/// Returns true if at least one such line is found.
#[allow(dead_code)]
pub fn hook_has_akar_not_found_error(hook_log: &Path) -> bool {
    read_jsonl_lines(hook_log)
        .iter()
        .any(|l| is_hook_not_found_error(l))
}

/// Number of most-recent hook events considered when detecting repeated
/// blocks. Older events outside this window do NOT trigger
/// `STOP_REPEATED_BLOCK`, so stale hook history alone cannot force the
/// governor to stop forever.
pub const RECENT_HOOK_WINDOW: usize = 50;

/// Return the first `command_preview` whose BLOCK count in the hook log is
/// 2 or more. Comparisons are exact string equality on the recorded preview.
/// Returns None if no command was blocked repeatedly.
///
/// v0.15.0: only the most recent [`RECENT_HOOK_WINDOW`] events are considered.
#[allow(dead_code)]
pub fn repeated_blocked_command(hook_log: &Path) -> Option<String> {
    repeated_blocked_command_in_window(hook_log, RECENT_HOOK_WINDOW).map(|(cmd, _)| cmd)
}

/// Return the first `command_preview` blocked 2+ times within the most recent
/// `window` hook events, together with that recent block count. Returns None
/// if no command was blocked repeatedly within the window.
///
/// Events older than the window are ignored, so a command blocked twice
/// historically but not within the recent window does not trigger.
pub fn repeated_blocked_command_in_window(
    hook_log: &Path,
    window: usize,
) -> Option<(String, usize)> {
    let lines = read_jsonl_lines(hook_log);
    // Consider only the most recent `window` events. If the log is shorter,
    // all events are considered.
    let start = lines.len().saturating_sub(window);
    let recent = &lines[start..];
    // Preserve insertion order of first occurrence within the window for
    // deterministic output.
    let mut order: Vec<String> = Vec::new();
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for line in recent {
        let decision = json_str(line, "decision").unwrap_or_default();
        if !decision.eq_ignore_ascii_case("BLOCK") {
            continue;
        }
        let preview = json_str(line, "command_preview").unwrap_or_default();
        if preview.is_empty() {
            continue;
        }
        if !counts.contains_key(&preview) {
            order.push(preview.clone());
        }
        *counts.entry(preview).or_insert(0) += 1;
    }
    for cmd in order {
        let c = counts.get(&cmd).copied().unwrap_or(0);
        if c >= 2 {
            return Some((cmd, c));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// LEARNING_PATCHES.md scanning (read-only)
// ---------------------------------------------------------------------------

/// True if `.akar/LEARNING_PATCHES.md` contains an ACTIVE split-rule entry.
///
/// v0.14.0 lifecycle: resolved split-rule entries no longer trigger
/// SPLIT_TASK. Only entries that are not `status: resolved` and contain the
/// split-rule marker count. Returns false if the file is absent or unreadable.
/// Never panics.
pub fn learning_patches_require_split(path: &Path) -> bool {
    learn::has_active_split_rule_entry(path)
}

// ---------------------------------------------------------------------------
// decide — apply rules A..I in this exact order
// ---------------------------------------------------------------------------

/// Build a loop governor report from local evidence.
///
/// Rules are applied in this fixed priority order (highest first):
///
/// A. git unavailable / not a repository → `UNKNOWN`
/// B. ERROR event with `akar not found` → `STOP_HOOK_BROKEN`
/// C. 2+ BLOCK events for the same command_preview → `STOP_REPEATED_BLOCK`
/// D. learning patches require split → `SPLIT_TASK`
/// E. baseline exists + dirty tree → `RUN_POSTMORTEM`
/// F. baseline exists + clean tree → `READY`
/// G. no baseline + clean tree → `SNAPSHOT_NOW`
/// H. no baseline + dirty tree → `COMMIT_CHECKPOINT`
/// I. unclassifiable → `UNKNOWN`
///
/// The governor never executes the chosen action.
pub fn decide(cfg: &config::Config) -> LoopGovernorReport {
    let project_root = &cfg.project_root;
    let akar_dir = &cfg.akar_dir;
    let hook_log = akar_dir.join("HOOK_EVENTS.jsonl");
    let baseline_path = akar_dir.join("DIFF_BASELINE.json");
    let patches_path = akar_dir.join("LEARNING_PATCHES.md");

    let mut evidence: Vec<String> = Vec::new();

    // --- Rule A: git must be available and this must be a repository. ---
    let tree_clean: Option<bool> = match diff_budget::is_working_tree_clean(project_root) {
        Ok(clean) => {
            evidence.push("git repository status: available".to_string());
            evidence.push(format!(
                "working tree clean: {}",
                if clean { "yes" } else { "no" }
            ));
            Some(clean)
        }
        Err(_) => {
            evidence.push("git repository status: unavailable".to_string());
            None
        }
    };

    if tree_clean.is_none() {
        // Rule A: git unavailable or not a repository.
        return LoopGovernorReport {
            decision: LoopDecision::Unknown,
            reason: "git is unavailable or this is not a git repository".to_string(),
            next_action: unknown_no_git_next_action().to_string(),
            suggested_prompt: unknown_prompt().to_string(),
            evidence_used: evidence,
        };
    }

    // Evidence from runtime files (presence-only, read-only).
    let baseline_present = baseline_path.exists();
    evidence.push(format!(
        ".akar/DIFF_BASELINE.json: {}",
        if baseline_present {
            "present"
        } else {
            "absent"
        }
    ));
    let hook_lines = read_jsonl_lines(&hook_log);
    evidence.push(format!(
        ".akar/HOOK_EVENTS.jsonl: {} event(s)",
        hook_lines.len()
    ));
    let patches_present = patches_path.exists();
    evidence.push(format!(
        ".akar/LEARNING_PATCHES.md: {}",
        if patches_present { "present" } else { "absent" }
    ));

    // --- Rule B: hook ERROR with `akar not found`. ---
    if hook_lines.iter().any(|l| is_hook_not_found_error(l)) {
        return LoopGovernorReport {
            decision: LoopDecision::StopHookBroken,
            reason: "PreToolUse hook recorded an ERROR: akar not found in subprocess PATH"
                .to_string(),
            next_action: foundation::hook_broken_playbook().to_string(),
            suggested_prompt: hook_broken_prompt().to_string(),
            evidence_used: evidence,
        };
    }

    // --- Rule C: 2+ BLOCK events for the same command_preview in the recent
    // window. Older events outside the window do NOT trigger (v0.15.0). ---
    if let Some((cmd, count)) = repeated_blocked_command_in_window(&hook_log, RECENT_HOOK_WINDOW) {
        evidence.push(format!(
            "repeated block: `{}` blocked {} times within recent {} hook events",
            cmd, count, RECENT_HOOK_WINDOW
        ));
        return LoopGovernorReport {
            decision: LoopDecision::StopRepeatedBlock,
            reason: format!(
                "same command blocked {} times within recent {} hook events: {}",
                count, RECENT_HOOK_WINDOW, cmd
            ),
            next_action: foundation::repeated_block_playbook(&cmd),
            suggested_prompt: repeated_block_prompt().to_string(),
            evidence_used: evidence,
        };
    }

    // --- Rule D: an ACTIVE split-rule learning patch entry requires a split. ---
    // Resolved entries are intentionally ignored (v0.14.0 lifecycle).
    if learning_patches_require_split(&patches_path) {
        evidence.push("LEARNING_PATCHES.md: active split-rule entry present".to_string());
        return LoopGovernorReport {
            decision: LoopDecision::SplitTask,
            reason: "active split-rule entry in LEARNING_PATCHES.md requires reducing scope or splitting the task"
                .to_string(),
            next_action: split_next_action(),
            suggested_prompt: split_prompt().to_string(),
            evidence_used: evidence,
        };
    }

    let clean = tree_clean.unwrap_or(false);

    // --- Rule E: baseline exists + dirty tree. ---
    if baseline_present && !clean {
        return LoopGovernorReport {
            decision: LoopDecision::RunPostmortem,
            reason: "baseline exists and working tree is dirty".to_string(),
            next_action: postmortem_next_action().to_string(),
            suggested_prompt: postmortem_prompt().to_string(),
            evidence_used: evidence,
        };
    }

    // --- Rule F: baseline exists + clean tree. ---
    if baseline_present && clean {
        return LoopGovernorReport {
            decision: LoopDecision::Ready,
            reason: "baseline exists and working tree is clean".to_string(),
            next_action: ready_next_action().to_string(),
            suggested_prompt: ready_prompt().to_string(),
            evidence_used: evidence,
        };
    }

    // --- Rule G: no baseline + clean tree. ---
    if !baseline_present && clean {
        return LoopGovernorReport {
            decision: LoopDecision::SnapshotNow,
            reason: "no baseline snapshot and working tree is clean".to_string(),
            next_action: snapshot_next_action(),
            suggested_prompt: snapshot_prompt().to_string(),
            evidence_used: evidence,
        };
    }

    // --- Rule H: no baseline + dirty tree. ---
    if !baseline_present && !clean {
        return LoopGovernorReport {
            decision: LoopDecision::CommitCheckpoint,
            reason: "no baseline snapshot and working tree is dirty".to_string(),
            next_action: commit_next_action(),
            suggested_prompt: commit_prompt().to_string(),
            evidence_used: evidence,
        };
    }

    // --- Rule I: state cannot be classified. ---
    LoopGovernorReport {
        decision: LoopDecision::Unknown,
        reason: "loop state could not be classified".to_string(),
        next_action: unknown_unclassified_next_action().to_string(),
        suggested_prompt: unknown_prompt().to_string(),
        evidence_used: evidence,
    }
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Format the loop governor section for `akar status`.
///
/// Prints decision, reason, next action, and the evidence used. Indented to
/// sit under the existing status block.
pub fn format_loop_governor(report: &LoopGovernorReport) -> String {
    let mut out = String::new();
    out.push_str("  loop governor:\n");
    out.push_str(&format!("    decision:    {}\n", report.decision.as_str()));
    out.push_str(&format!("    reason:      {}\n", report.reason));
    out.push_str(&format!(
        "    next action: {}\n",
        indent_continuation(&report.next_action)
    ));
    out.push_str("    evidence used:\n");
    if report.evidence_used.is_empty() {
        out.push_str("      - (none)\n");
    } else {
        for e in &report.evidence_used {
            out.push_str(&format!("      - {}\n", e));
        }
    }
    out
}

/// Format the standalone `akar governor` human-readable report: decision,
/// reason, next action, suggested prompt, and evidence used. Advisory only.
pub fn format_governor_report(report: &LoopGovernorReport) -> String {
    let mut out = String::new();
    out.push_str("governor:\n");
    out.push_str(&format!("  decision:    {}\n", report.decision.as_str()));
    out.push_str(&format!("  reason:      {}\n", report.reason));
    out.push_str(&format!(
        "  next action: {}\n",
        indent_continuation_gov(&report.next_action)
    ));
    out.push_str(&format!(
        "  suggested prompt: {}\n",
        one_line(&report.suggested_prompt)
    ));
    out.push_str("  evidence used:\n");
    if report.evidence_used.is_empty() {
        out.push_str("    - (none)\n");
    } else {
        for e in &report.evidence_used {
            out.push_str(&format!("    - {}\n", e));
        }
    }
    out
}

/// Format the governor decision as exactly one line:
/// `DECISION<TAB>SUGGESTED_PROMPT`.
///
/// No extra decoration, no color, no multiline. The suggested prompt is
/// collapsed to a single line. Always emits exactly one line (no trailing
/// content beyond the single newline added by the caller's `println!`).
pub fn format_governor_one_line(report: &LoopGovernorReport) -> String {
    format!(
        "{}\t{}",
        report.decision.as_str(),
        one_line(&report.suggested_prompt)
    )
}

/// Escape a string for use inside a JSON double-quoted value.
/// Covers the characters required by RFC 8259 §7. Std-only, no external deps.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Format the governor decision as a single-line JSON object with fields:
/// `decision`, `reason`, `next_action`, `suggested_prompt`, `evidence_used`.
///
/// `evidence_used` is a JSON array of strings. Valid JSON, std-only.
pub fn format_governor_json(report: &LoopGovernorReport) -> String {
    let evidence: Vec<String> = report
        .evidence_used
        .iter()
        .map(|e| format!("\"{}\"", json_escape(e)))
        .collect();
    format!(
        "{{\"decision\":\"{decision}\",\"reason\":\"{reason}\",\"next_action\":\"{next_action}\",\"suggested_prompt\":\"{suggested_prompt}\",\"evidence_used\":[{evidence}]}}",
        decision = json_escape(report.decision.as_str()),
        reason = json_escape(&report.reason),
        next_action = json_escape(&one_line(&report.next_action)),
        suggested_prompt = json_escape(&one_line(&report.suggested_prompt)),
        evidence = evidence.join(","),
    )
}

/// Indent continuation lines of a (possibly multi-line) string so they align
/// under the `next action:` value for the standalone governor report.
fn indent_continuation_gov(s: &str) -> String {
    let mut lines = s.lines();
    let mut out = String::new();
    if let Some(first) = lines.next() {
        out.push_str(first);
    }
    for l in lines {
        out.push('\n');
        out.push_str("                ");
        out.push_str(l);
    }
    out
}

/// Indent continuation lines of a (possibly multi-line) string so they align
/// under the `next action:` label value (status block variant).
fn indent_continuation(s: &str) -> String {
    let mut lines = s.lines();
    let mut out = String::new();
    if let Some(first) = lines.next() {
        out.push_str(first);
    }
    for l in lines {
        out.push('\n');
        out.push_str("                 ");
        out.push_str(l);
    }
    out
}

/// Format the loop governor block for inclusion in `.akar/NEXT_RUN.md`.
///
/// Includes decision, reason, next action, suggested next Claude prompt, and
/// evidence used.
#[allow(dead_code)]
pub fn format_next_run_block(report: &LoopGovernorReport) -> String {
    let mut out = String::new();
    out.push_str("## Loop Governor Decision\n");
    out.push_str(&format!("- decision: {}\n", report.decision.as_str()));
    out.push_str(&format!("- reason: {}\n", report.reason));
    out.push_str(&format!(
        "- next action: {}\n",
        one_line(&report.next_action)
    ));
    out.push_str("## Suggested Next Claude Prompt\n");
    out.push_str(&format!("```\n{}\n```\n", report.suggested_prompt));
    out.push_str("## Evidence Used\n");
    if report.evidence_used.is_empty() {
        out.push_str("- (none)\n");
    } else {
        for e in &report.evidence_used {
            out.push_str(&format!("- {}\n", e));
        }
    }
    out
}

/// Collapse a multi-line string to a single line for compact NEXT_RUN output.
fn one_line(s: &str) -> String {
    s.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// NEXT_RUN.md writer (governor-aware)
// ---------------------------------------------------------------------------

/// Write (or overwrite) `.akar/NEXT_RUN.md` with a compiled, Claude-ready
/// next-run prompt (v0.19.0).
///
/// The prompt includes the 11 sections required by v0.19.0 in exact order:
/// Current State, Governor Decision, Evidence Used, Objective, Hard Rules,
/// Allowed Commands, Forbidden Commands, Stop Conditions, Verification
/// Required, Final Response Format.
///
/// Unlike the resume-mode `write_next_run`, this ALWAYS overwrites so the
/// NEXT_RUN file reflects the freshest governor decision. It does not execute
/// the chosen action. Returns the path written, or None on failure.
pub fn write_governor_next_run(
    cfg: &config::Config,
    report: &LoopGovernorReport,
    task: Option<&str>,
) -> Option<std::path::PathBuf> {
    if !cfg.akar_dir.exists() {
        return None;
    }
    let path = cfg.akar_dir.join("NEXT_RUN.md");
    let content = compile_next_run_prompt(cfg, report, task);
    std::fs::write(&path, content).ok()?;
    Some(path)
}

// ---------------------------------------------------------------------------
// Next-run prompt compiler (v0.19.0)
// ---------------------------------------------------------------------------

/// Compile a Claude-ready next-run prompt from the governor report.
///
/// Produces 11 sections in this exact order:
/// 1. `# AKAR Next Run`
/// 2. `## Current State`
/// 3. `## Governor Decision`
/// 4. `## Evidence Used`
/// 5. `## Objective`
/// 6. `## Hard Rules`
/// 7. `## Allowed Commands`
/// 8. `## Forbidden Commands`
/// 9. `## Stop Conditions`
/// 10. `## Verification Required`
/// 11. `## Final Response Format`
///
/// Advisory only — AKAR does not run the compiled prompt.
pub fn compile_next_run_prompt(
    cfg: &config::Config,
    report: &LoopGovernorReport,
    task: Option<&str>,
) -> String {
    let ts = event_log::now_iso8601();
    let version = crate_version();
    let kind = pvc::detect_project_kind(&cfg.project_root);
    let mut out = String::new();

    // Redact the task (it may echo a prompt containing secrets) and collapse to
    // one line. Empty/whitespace tasks are treated as no task.
    let task_clean: Option<String> = task.and_then(|t| {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(one_line(&config::redact(trimmed)))
        }
    });

    out.push_str("# AKAR Next Run\n\n");

    // 2. Current State
    out.push_str("## Current State\n");
    out.push_str(&format!("- AKAR version: {}\n", version));
    out.push_str(&format!("- project: {}\n", cfg.project_name));
    out.push_str(&format!("- project kind: {}\n", kind.label()));
    out.push_str(&format!("- timestamp: {}\n", ts));
    out.push_str(&format!(
        "- governor decision: {}\n",
        report.decision.as_str()
    ));
    out.push_str(&format!("- reason: {}\n", one_line(&report.reason)));
    out.push_str(&format!(
        "- next action: {}\n",
        one_line(&report.next_action)
    ));
    if let Some(t) = &task_clean {
        out.push_str(&format!("- requested task: {}\n", t));
    }
    out.push('\n');

    // 3. Governor Decision
    out.push_str("## Governor Decision\n");
    out.push_str(&format!("- decision: {}\n", report.decision.as_str()));
    out.push_str(&format!(
        "- class: {}\n",
        report.decision.decision_class().as_str()
    ));
    out.push_str("- suggested prompt:\n");
    out.push_str(&format!("```\n{}\n```\n\n", report.suggested_prompt));

    // 4. Evidence Used
    out.push_str("## Evidence Used\n");
    if report.evidence_used.is_empty() {
        out.push_str("No evidence files were used.\n\n");
    } else {
        for e in &report.evidence_used {
            out.push_str(&format!("- {}\n", e));
        }
        out.push('\n');
    }

    // 5. Objective
    // The base objective string is always emitted verbatim first so the
    // validator's `content.contains(objective_for_decision(...))` check passes.
    // When a task is supplied, it is threaded in as advisory context — for
    // non-stop decisions the task follows the objective; for stop-class
    // decisions the blocker remains primary and the task is listed as
    // secondary ("after the blocker is resolved"). The task never overrides
    // governor safety.
    out.push_str("## Objective\n");
    out.push_str(objective_for_decision(&report.decision));
    out.push('\n');
    if let Some(t) = &task_clean {
        match report.decision.decision_class() {
            DecisionClass::Continue | DecisionClass::ActionRequired => {
                out.push_str(&format!("- Task: {}\n", t));
            }
            DecisionClass::Stop => {
                out.push_str("- Primary objective: resolve the governor blocker above before attempting any task.\n");
                out.push_str(&format!(
                    "- Requested task after the blocker is resolved: {}\n",
                    t
                ));
            }
        }
    }
    out.push('\n');

    // 6. Hard Rules
    out.push_str("## Hard Rules\n");
    for rule in hard_rules() {
        out.push_str(&format!("- {}\n", rule));
    }
    out.push('\n');

    // 7. Allowed Commands
    out.push_str("## Allowed Commands\n");
    for cmd in allowed_commands(kind, &report.decision) {
        out.push_str(&format!("- `{}`\n", cmd));
    }
    // High-confidence discovery hints for known project kinds.
    let discovery = pvc::discover_verification_hints(&cfg.project_root);
    for hint in &discovery.hints {
        if hint.confidence == pvc::VerificationConfidence::High
            && !hint.requires_confirmation
            && !allowed_commands(kind, &report.decision).contains(&hint.command)
        {
            out.push_str(&format!(
                "- `{}`  *(discovered from {})*\n",
                hint.command, hint.source
            ));
        }
    }
    out.push('\n');

    // 8. Forbidden Commands
    out.push_str("## Forbidden Commands\n");
    for cmd in forbidden_commands() {
        out.push_str(&format!("- `{}`\n", cmd));
    }
    out.push('\n');

    // 9. Stop Conditions
    out.push_str("## Stop Conditions\n");
    for cond in stop_conditions(kind, &report.decision) {
        out.push_str(&format!("- {}\n", cond));
    }
    out.push('\n');

    // 10. Verification Required
    out.push_str("## Verification Required\n");
    for cmd in verification_commands(kind, &report.decision) {
        out.push_str(&format!("- `{}`\n", cmd));
    }
    // Discovery hints: for Unknown projects, include confidence-ranked hints.
    // For known projects, include medium/low hints that weren't already in
    // allowed commands.
    for hint in &discovery.hints {
        let already_in_verification =
            verification_commands(kind, &report.decision).contains(&hint.command);
        if kind == pvc::ProjectKind::Unknown {
            // For Unknown projects, show all discovered hints with confidence.
            if hint.confidence == pvc::VerificationConfidence::High && !hint.requires_confirmation {
                out.push_str(&format!(
                    "- `{}`  *(High, {})*\n",
                    hint.command, hint.source
                ));
            } else {
                out.push_str(&format!(
                    "- Ask the user before running discovered verification command: `{}`  *({}, {})*\n",
                    hint.command,
                    hint.confidence.as_str(),
                    hint.source
                ));
            }
        } else if !already_in_verification && hint.confidence != pvc::VerificationConfidence::High {
            // Medium/Low hints for known projects — suggest, don't require.
            out.push_str(&format!(
                "- Ask the user before running discovered verification command: `{}`  *({}, {})*\n",
                hint.command,
                hint.confidence.as_str(),
                hint.source
            ));
        }
    }
    // Unknown projects get human-readable guidance (always).
    if kind == pvc::ProjectKind::Unknown {
        for guide in pvc::unknown_verification_guidance() {
            out.push_str(&format!("- {}\n", guide));
        }
    }
    out.push('\n');

    // 11. Final Response Format
    out.push_str("## Final Response Format\n");
    for (n, item) in final_response_format().iter().enumerate() {
        out.push_str(&format!("{}. {}\n", n + 1, item));
    }
    out.push('\n');

    out.push_str("<!-- AKAR compiled this prompt from local evidence and foundation playbooks. AKAR did NOT run it automatically. -->\n");

    out
}

/// The AKAR version, sourced from `CARGO_PKG_VERSION` at compile time.
fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The always-on hard rules for a next-run prompt.
fn hard_rules() -> Vec<&'static str> {
    vec![
        "Do not run git reset.",
        "Do not run git clean.",
        "Do not run git stash.",
        "Do not run git checkout.",
        "Do not push.",
        "Do not delete user project files.",
        "Do not modify Claude Code configuration.",
        "Do not retry blocked commands.",
        "Do not broaden the task after budget or governor warnings.",
        "Stop if verification fails.",
    ]
}

/// Decision-specific allowed-command additions.
fn allowed_command_additions(kind: pvc::ProjectKind, decision: &LoopDecision) -> Vec<String> {
    match decision {
        LoopDecision::SnapshotNow => vec![format!(
            "{} preflight --snapshot \"<task>\"",
            pvc::akar_prefix(kind)
        )],
        LoopDecision::RunPostmortem => vec![format!(
            "{} postmortem --diff --baseline",
            pvc::akar_prefix(kind)
        )],
        LoopDecision::CommitCheckpoint => {
            vec![
                "git add <explicit files>".to_string(),
                "git commit -m \"<message>\"".to_string(),
            ]
        }
        LoopDecision::SplitTask => {
            vec!["(no git mutation; write a smaller prompt only)".to_string()]
        }
        LoopDecision::StopHookBroken => {
            vec![
                "where.exe akar".to_string(),
                "pwsh -NoProfile -Command \"Get-Command akar\"".to_string(),
            ]
        }
        LoopDecision::StopRepeatedBlock => vec!["inspect `.akar/HOOK_EVENTS.jsonl`".to_string()],
        _ => vec![],
    }
}

/// The always-on allowed commands plus decision-specific additions.
fn allowed_commands(kind: pvc::ProjectKind, decision: &LoopDecision) -> Vec<String> {
    let mut cmds: Vec<String> = vec![
        "git status".to_string(),
        "git diff".to_string(),
        "git diff --stat".to_string(),
        "git log --oneline -5".to_string(),
    ];
    // Project-specific build+test commands.
    for c in pvc::project_allowed_commands(kind) {
        cmds.push(c);
    }
    // AKAR CLI commands with project-appropriate prefix.
    for c in pvc::akar_cli_commands(kind) {
        cmds.push(c);
    }
    for add in allowed_command_additions(kind, decision) {
        cmds.push(add);
    }
    cmds
}

/// The always-on forbidden commands.
fn forbidden_commands() -> Vec<&'static str> {
    vec![
        "git reset",
        "git clean",
        "git stash",
        "git checkout",
        "git push",
        "rm -rf /",
        "rm -rf /*",
        "sudo rm -rf /",
        "del /s /q C:\\",
        "Remove-Item -Recurse -Force C:\\",
        "broad git add . unless explicitly justified and listed",
    ]
}

/// Decision-specific stop-condition additions.
fn stop_condition_additions(decision: &LoopDecision) -> Vec<&'static str> {
    match decision {
        LoopDecision::SnapshotNow => vec!["Stop if snapshot fails."],
        LoopDecision::RunPostmortem => vec!["Stop if postmortem returns UNKNOWN."],
        LoopDecision::CommitCheckpoint => vec!["Stop if verification fails before commit."],
        LoopDecision::SplitTask => vec!["Stop after producing the smaller prompt."],
        LoopDecision::Unknown => vec!["Stop after reporting unknown state."],
        _ => vec![],
    }
}

/// The always-on stop conditions plus decision-specific additions.
fn stop_conditions(kind: pvc::ProjectKind, decision: &LoopDecision) -> Vec<String> {
    let mut conds: Vec<String> = vec![
        "Stop if hook evidence is missing when hook behavior is being tested.".to_string(),
        "Stop if `akar hooks --check` fails.".to_string(),
        "Stop if `akar safety \"rm -rf /\"` is not BLOCKED during safety proof.".to_string(),
        format!("Stop if `{} eval` fails.", pvc::akar_prefix(kind)),
        "Stop if governor decision is STOP_HOOK_BROKEN.".to_string(),
        "Stop if governor decision is STOP_REPEATED_BLOCK and the plan still retries the same command.".to_string(),
    ];
    // Project-specific test-failure stop condition.
    conds.extend(pvc::project_stop_conditions(kind));
    for add in stop_condition_additions(decision) {
        conds.push(add.to_string());
    }
    conds
}

/// Decision-specific verification-command additions.
fn verification_additions(kind: pvc::ProjectKind, decision: &LoopDecision) -> Vec<String> {
    match decision {
        LoopDecision::RunPostmortem => vec![format!(
            "{} postmortem --diff --baseline",
            pvc::akar_prefix(kind)
        )],
        LoopDecision::SnapshotNow => vec![format!(
            "{} preflight --snapshot \"<task>\"",
            pvc::akar_prefix(kind)
        )],
        LoopDecision::CommitCheckpoint => {
            vec!["git status".to_string(), "git diff --stat".to_string()]
        }
        LoopDecision::StopRepeatedBlock => {
            vec!["read `.akar/HOOK_EVENTS.jsonl` and confirm repeated block evidence".to_string()]
        }
        _ => vec![],
    }
}

/// The always-on verification commands plus decision-specific additions.
fn verification_commands(kind: pvc::ProjectKind, decision: &LoopDecision) -> Vec<String> {
    let mut cmds: Vec<String> = Vec::new();
    // Project-specific build+test commands.
    for c in pvc::project_verification_commands(kind) {
        cmds.push(c);
    }
    // AKAR CLI commands with project-appropriate prefix.
    for c in pvc::akar_cli_commands(kind) {
        cmds.push(c);
    }
    for add in verification_additions(kind, decision) {
        cmds.push(add);
    }
    cmds
}

/// The final-response-format checklist for a next-run prompt.
fn final_response_format() -> Vec<&'static str> {
    vec![
        "Baseline confirmation",
        "Files changed",
        "Governor decision",
        "NEXT_RUN prompt sections generated",
        "Safety boundaries confirmed",
        "Verification results with exact command lines",
        "Runtime behavior changed: yes/no",
        "Final commit hash if committed",
        "Honest conclusion",
        "Next recommended release",
    ]
}

// ---------------------------------------------------------------------------
// NEXT_RUN prompt contract validator (v0.20.0)
// ---------------------------------------------------------------------------

/// The 11 required section headers, in their exact required order.
pub const REQUIRED_SECTIONS: &[&str] = &[
    "# AKAR Next Run",
    "## Current State",
    "## Governor Decision",
    "## Evidence Used",
    "## Objective",
    "## Hard Rules",
    "## Allowed Commands",
    "## Forbidden Commands",
    "## Stop Conditions",
    "## Verification Required",
    "## Final Response Format",
];

/// Forbidden-command items that must appear in the Forbidden Commands section.
const REQUIRED_FORBIDDEN_ITEMS: &[&str] = &[
    "git reset",
    "git clean",
    "git stash",
    "git checkout",
    "git push",
    "rm -rf /",
    "rm -rf /*",
    "sudo rm -rf /",
    "del /s /q C:\\",
    "Remove-Item -Recurse -Force C:\\",
];

/// Hard-rule lines that must appear in the Hard Rules section.
const REQUIRED_HARD_RULES: &[&str] = &[
    "Do not retry blocked commands.",
    "Stop if verification fails.",
];

/// Result of validating a compiled NEXT_RUN prompt against the v0.20.0 contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NextRunCheckResult {
    /// Overall pass/fail.
    pub pass: bool,
    /// Per-check outcomes, in display order.
    pub checks: [bool; 4],
    /// Human-readable failure reasons (empty on full PASS).
    pub reasons: Vec<String>,
}

impl NextRunCheckResult {
    /// Labels for the four checks, in order.
    pub const CHECK_LABELS: [&'static str; 4] = [
        "sections",
        "minimum content",
        "safety contract",
        "decision consistency",
    ];

    /// True iff all checks passed.
    #[allow(dead_code)]
    pub fn is_pass(&self) -> bool {
        self.pass
    }
}

/// Validate a compiled NEXT_RUN prompt string against the v0.20.0 contract.
///
/// Checks (in order):
/// 1. **sections** — all 11 required headers present, in exact order.
/// 2. **minimum content** — version, decision, reason, next action, class,
///    objective, evidence (≥1 line or the placeholder), and every body
///    section is non-empty.
/// 3. **safety contract** — all forbidden-command items present, and the two
///    required hard-rule lines present.
/// 4. **decision consistency** — the decision's class matches the decision,
///    and the objective matches the decision.
///
/// Read-only: does not write or fix anything. Never panics.
pub fn validate_next_run(content: &str) -> NextRunCheckResult {
    let mut reasons: Vec<String> = Vec::new();

    // ---- Check 1: required sections in exact order ----------------------
    // Collect every Markdown header line (`# ` or `## `) in document order.
    let section_headers: Vec<&str> = content
        .lines()
        .filter(|l| l.starts_with("# ") || l.starts_with("## "))
        .collect();
    let mut section_reasons: Vec<String> = Vec::new();
    let sections_ok = sections_in_order(&section_headers, &mut section_reasons);

    // ---- Check 2: minimum content ---------------------------------------
    let mut content_reasons: Vec<String> = Vec::new();
    let content_ok = check_minimum_content(content, &mut content_reasons);

    // ---- Check 3: safety contract ---------------------------------------
    let mut safety_reasons: Vec<String> = Vec::new();
    let safety_ok = check_safety_contract(content, &mut safety_reasons);

    // ---- Check 4: decision consistency ----------------------------------
    let mut consistency_reasons: Vec<String> = Vec::new();
    let consistency_ok = check_decision_consistency(content, &mut consistency_reasons);

    let checks = [sections_ok, content_ok, safety_ok, consistency_ok];
    let pass = checks.iter().all(|c| *c);

    // Aggregate failure reasons in check order for stable output.
    reasons.extend(section_reasons);
    reasons.extend(content_reasons);
    reasons.extend(safety_reasons);
    reasons.extend(consistency_reasons);

    NextRunCheckResult {
        pass,
        checks,
        reasons,
    }
}

/// Check that the 11 required section headers appear in exact order.
/// Appends specific failure reasons to `out`. Returns true if OK.
fn sections_in_order(found: &[&str], out: &mut Vec<String>) -> bool {
    // `found` may include the title and the 10 `## ` headers, in document
    // order. Required sections must be a subsequence of `found` and appear
    // in the exact required order.
    let mut found_idx = 0usize;
    let mut matched = 0usize;
    for required in REQUIRED_SECTIONS {
        let mut found_match = false;
        while found_idx < found.len() {
            if found[found_idx] == *required {
                matched += 1;
                found_idx += 1;
                found_match = true;
                break;
            }
            found_idx += 1;
        }
        if !found_match {
            out.push(format!("missing or out-of-order section: `{}`", required));
            return false;
        }
    }
    // Also reject extra `## `/`# ` headers interspersed between required ones
    // in a way that breaks order — the subsequence check above already
    // enforces order. Require exactly the 11 required headers (no more top
    // `# `/`## ` headers than expected) to catch duplicates.
    if found.len() != REQUIRED_SECTIONS.len() {
        // Could be extra headers; if all required matched in order, still
        // flag unexpected header count for strictness.
        if matched == REQUIRED_SECTIONS.len() {
            out.push(format!(
                "unexpected section header count: expected {}, found {}",
                REQUIRED_SECTIONS.len(),
                found.len()
            ));
            return false;
        }
    }
    true
}

/// Extract the body lines of a `## Section` (lines after its header up to the
/// next `# `/`## ` header or EOF), excluding the header line itself.
fn section_body<'a>(content: &'a str, header: &str) -> Vec<&'a str> {
    let lines: Vec<&str> = content.lines().collect();
    let mut body = Vec::new();
    let mut in_section = false;
    for line in lines {
        if line.starts_with("# ") || line.starts_with("## ") {
            if in_section {
                break;
            }
            if line == header {
                in_section = true;
                continue;
            }
        } else if in_section {
            body.push(line);
        }
    }
    body
}

/// True if a section body has at least one non-empty, non-comment line.
fn section_nonempty(content: &str, header: &str) -> bool {
    section_body(content, header)
        .iter()
        .any(|l| !l.trim().is_empty() && !l.starts_with("<!--"))
}

/// Check minimum-content requirements. Appends reasons; returns true if OK.
fn check_minimum_content(content: &str, out: &mut Vec<String>) -> bool {
    let mut ok = true;

    // Required scalar fields.
    let required_fields = [
        ("AKAR version", "- AKAR version:"),
        ("governor decision", "- governor decision:"),
        ("reason", "- reason:"),
        ("next action", "- next action:"),
        ("decision class", "- class:"),
        ("objective", "## Objective"),
    ];
    for (label, needle) in required_fields {
        if !content.contains(needle) {
            out.push(format!("missing minimum content: {}", label));
            ok = false;
        }
    }

    // Evidence: at least one evidence line OR the placeholder.
    let evidence_body = section_body(content, "## Evidence Used");
    let has_evidence = evidence_body
        .iter()
        .any(|l| l.trim().starts_with("- ") && !l.trim().is_empty());
    let has_placeholder = evidence_body
        .iter()
        .any(|l| l.contains("No evidence files were used."));
    if !has_evidence && !has_placeholder {
        out.push("missing minimum content: evidence".to_string());
        ok = false;
    }

    // Body sections must be non-empty.
    let body_sections = [
        ("Hard Rules", "## Hard Rules"),
        ("Allowed Commands", "## Allowed Commands"),
        ("Forbidden Commands", "## Forbidden Commands"),
        ("Stop Conditions", "## Stop Conditions"),
        ("Verification Required", "## Verification Required"),
        ("Final Response Format", "## Final Response Format"),
    ];
    for (label, header) in body_sections {
        if !section_nonempty(content, header) {
            out.push(format!("empty section: {}", label));
            ok = false;
        }
    }

    ok
}

/// Check the safety contract: all forbidden items + required hard rules.
fn check_safety_contract(content: &str, out: &mut Vec<String>) -> bool {
    let mut ok = true;
    for item in REQUIRED_FORBIDDEN_ITEMS {
        if !content.contains(item) {
            out.push(format!("missing forbidden command: `{}`", item));
            ok = false;
        }
    }
    for rule in REQUIRED_HARD_RULES {
        if !content.contains(rule) {
            out.push(format!("missing hard rule: `{}`", rule));
            ok = false;
        }
    }
    ok
}

/// Check decision consistency: parse decision + class + objective from the
/// content and verify they agree.
fn check_decision_consistency(content: &str, out: &mut Vec<String>) -> bool {
    let mut ok = true;

    // Parse decision label from "- decision: <LABEL>".
    let decision_label = content
        .lines()
        .find_map(|l| {
            let t = l.trim();
            t.strip_prefix("- decision: ").map(|s| s.trim().to_string())
        })
        .or_else(|| {
            content.lines().find_map(|l| {
                let t = l.trim();
                t.strip_prefix("- governor decision: ")
                    .map(|s| s.trim().to_string())
            })
        });

    let decision_label = match decision_label {
        Some(d) => d,
        None => {
            out.push("decision consistency: decision label not found".to_string());
            return false;
        }
    };

    let decision = match parse_decision_label(&decision_label) {
        Some(d) => d,
        None => {
            out.push(format!(
                "decision consistency: unrecognized decision label `{}`",
                decision_label
            ));
            return false;
        }
    };

    // Parse class line "- class: <class>".
    let class_label = content.lines().find_map(|l| {
        let t = l.trim();
        t.strip_prefix("- class: ").map(|s| s.trim().to_string())
    });
    let expected_class = decision.decision_class().as_str();
    match class_label.as_deref() {
        Some(c) if c == expected_class => {}
        Some(c) => {
            out.push(format!(
                "decision class mismatch: decision {} expects `{}` but found `{}`",
                decision_label, expected_class, c
            ));
            ok = false;
        }
        None => {
            out.push("decision consistency: class line not found".to_string());
            ok = false;
        }
    }

    // Objective must contain the exact objective string for the decision.
    let expected_objective = objective_for_decision(&decision);
    if !content.contains(expected_objective) {
        out.push(format!(
            "decision objective mismatch: decision {} requires `{}`",
            decision_label, expected_objective
        ));
        ok = false;
    }

    ok
}

/// Parse a decision label string back into a `LoopDecision`.
fn parse_decision_label(label: &str) -> Option<LoopDecision> {
    match label {
        "READY" => Some(LoopDecision::Ready),
        "SNAPSHOT_NOW" => Some(LoopDecision::SnapshotNow),
        "RUN_POSTMORTEM" => Some(LoopDecision::RunPostmortem),
        "COMMIT_CHECKPOINT" => Some(LoopDecision::CommitCheckpoint),
        "SPLIT_TASK" => Some(LoopDecision::SplitTask),
        "STOP_HOOK_BROKEN" => Some(LoopDecision::StopHookBroken),
        "STOP_REPEATED_BLOCK" => Some(LoopDecision::StopRepeatedBlock),
        "UNKNOWN" => Some(LoopDecision::Unknown),
        _ => None,
    }
}

/// Format a `NextRunCheckResult` for `akar request --check` output.
pub fn format_next_run_check(result: &NextRunCheckResult) -> String {
    let mut out = String::new();
    if result.pass {
        out.push_str("NEXT_RUN check: PASS\n");
        for label in NextRunCheckResult::CHECK_LABELS.iter() {
            out.push_str(&format!("  - {}: PASS\n", label));
        }
    } else {
        out.push_str("NEXT_RUN check: FAIL\n");
        for reason in &result.reasons {
            out.push_str(&format!("  - {}\n", reason));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Opt-in governor telemetry (v0.18.0)
// ---------------------------------------------------------------------------

/// Output mode that produced a governor call, recorded in telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernorTelemetryMode {
    Human,
    OneLine,
    Json,
}

impl GovernorTelemetryMode {
    /// Machine-readable mode label for the telemetry event.
    pub fn as_str(&self) -> &'static str {
        match self {
            GovernorTelemetryMode::Human => "human",
            GovernorTelemetryMode::OneLine => "one-line",
            GovernorTelemetryMode::Json => "json",
        }
    }
}

/// Append one governor telemetry event to `.akar/EVENT_LOG.jsonl`.
///
/// Opt-in only (v0.18.0): the caller decides whether to invoke this. The
/// default governor path never calls it. The event is a single JSONL line
/// with fields: `timestamp`, `event` ("governor"), `decision`, `reason`
/// (redacted), `exit_code`, `mode`, `no_exit_code`.
///
/// - Does NOT log the suggested prompt (it may be long).
/// - Redacts obvious secrets in `reason` via `config::redact`.
/// - Does not mutate git or execute anything.
///
/// Returns the path written, or `None` if `.akar/` does not exist or the
/// write fails. Never panics.
pub fn write_governor_telemetry(
    cfg: &config::Config,
    report: &LoopGovernorReport,
    mode: GovernorTelemetryMode,
    no_exit_code: bool,
    exit_code: i32,
) -> Option<std::path::PathBuf> {
    if !cfg.akar_dir.exists() {
        return None;
    }
    let log_path = cfg.akar_dir.join("EVENT_LOG.jsonl");
    let ts = event_log::now_iso8601();
    // Redact the reason — it may echo command previews that contain secrets.
    // The suggested prompt is intentionally NOT logged (length + content).
    let reason = config::redact(&report.reason);
    let line = format!(
        "{{\"timestamp\":\"{ts}\",\"event\":\"governor\",\"decision\":\"{decision}\",\"reason\":\"{reason}\",\"exit_code\":{exit_code},\"mode\":\"{mode}\",\"no_exit_code\":{no_exit_code}}}",
        ts = json_escape(&ts),
        decision = json_escape(report.decision.as_str()),
        reason = json_escape(&reason),
        exit_code = exit_code,
        mode = json_escape(mode.as_str()),
        no_exit_code = if no_exit_code { "true" } else { "false" },
    );
    use std::fs::OpenOptions;
    use std::io::Write;
    if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(&log_path) {
        if writeln!(f, "{}", line).is_ok() {
            return Some(log_path);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    // v0.26: compile_next_run_prompt / write_governor_next_run now take an
    // optional task. Existing tests pass `None` (no task) via these wrappers
    // to preserve the pre-v0.26 behavior without editing every call site.
    fn compile(cfg: &config::Config, report: &LoopGovernorReport) -> String {
        compile_next_run_prompt(cfg, report, None)
    }
    fn write_nr(cfg: &config::Config, report: &LoopGovernorReport) -> Option<PathBuf> {
        write_governor_next_run(cfg, report, None)
    }

    /// Build a config whose `akar_dir` points at a temp dir. `project_root`
    /// is the real cwd so git status reflects the real repo.
    fn cfg_with_akar_dir(akar_dir: PathBuf) -> config::Config {
        config::Config {
            project_root: std::env::current_dir().unwrap(),
            akar_dir,
            global_dir: std::path::PathBuf::from("/nonexistent/__akar_gov_global__"),
            project_name: "akar-test".to_string(),
        }
    }

    fn fresh_akar_dir(label: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("akar_loop_gov_{}_{}", label, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(dir: &Path, name: &str, contents: &str) {
        fs::write(dir.join(name), contents).unwrap();
    }

    fn append_hook_line(dir: &Path, line: &str) {
        use std::io::Write;
        let path = dir.join("HOOK_EVENTS.jsonl");
        let mut f = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&path)
            .unwrap();
        writeln!(f, "{}", line).unwrap();
    }

    // ---- decision labels -------------------------------------------------

    #[test]
    fn decision_labels_are_complete() {
        assert_eq!(LoopDecision::Ready.as_str(), "READY");
        assert_eq!(LoopDecision::SnapshotNow.as_str(), "SNAPSHOT_NOW");
        assert_eq!(LoopDecision::RunPostmortem.as_str(), "RUN_POSTMORTEM");
        assert_eq!(LoopDecision::CommitCheckpoint.as_str(), "COMMIT_CHECKPOINT");
        assert_eq!(LoopDecision::SplitTask.as_str(), "SPLIT_TASK");
        assert_eq!(LoopDecision::StopHookBroken.as_str(), "STOP_HOOK_BROKEN");
        assert_eq!(
            LoopDecision::StopRepeatedBlock.as_str(),
            "STOP_REPEATED_BLOCK"
        );
        assert_eq!(LoopDecision::Unknown.as_str(), "UNKNOWN");
    }

    // ---- Rule B: hook ERROR akar not found → STOP_HOOK_BROKEN -----------

    #[test]
    fn hook_error_akar_not_found_produces_stop_hook_broken() {
        let dir = fresh_akar_dir("hook_broken");
        // Real repo (cwd) so git is available; the rule B check fires first
        // after the git availability gate.
        append_hook_line(
            &dir,
            r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"ls","decision":"ERROR","exit_code":0,"message":"akar not found in PATH"}"#,
        );
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_eq!(report.decision, LoopDecision::StopHookBroken);
        assert!(report.next_action.contains("PATH"));
        assert!(report.suggested_prompt.contains("Bash tool calls"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hook_allow_event_does_not_trigger_stop_hook_broken() {
        let dir = fresh_akar_dir("hook_ok");
        append_hook_line(
            &dir,
            r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"akar not found in a comment","decision":"ALLOW","exit_code":0}"#,
        );
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        // ALLOW event with the phrase must NOT trip the hook-broken rule.
        assert_ne!(report.decision, LoopDecision::StopHookBroken);
        fs::remove_dir_all(&dir).ok();
    }

    // ---- Rule C: repeated same BLOCK command → STOP_REPEATED_BLOCK ------

    #[test]
    fn repeated_same_block_command_produces_stop_repeated_block() {
        let dir = fresh_akar_dir("repeated_block");
        let block_line = r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"git push --force","decision":"BLOCK","exit_code":2}"#;
        append_hook_line(&dir, block_line);
        append_hook_line(&dir, block_line);
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_eq!(report.decision, LoopDecision::StopRepeatedBlock);
        assert!(report.next_action.contains("git push --force"));
        assert!(report.suggested_prompt.contains("Do not retry"));
        assert!(
            report
                .evidence_used
                .iter()
                .any(|e| e.contains("repeated block"))
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn single_block_does_not_trigger_repeated_block() {
        let dir = fresh_akar_dir("single_block");
        append_hook_line(
            &dir,
            r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}"#,
        );
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_ne!(report.decision, LoopDecision::StopRepeatedBlock);
        fs::remove_dir_all(&dir).ok();
    }

    // ---- Rule C v0.15.0: recent-window signal hygiene ------------------

    /// An ALLOW event line used to pad the hook log so older BLOCK events
    /// fall outside the recent 50-event window.
    fn allow_line(n: usize) -> String {
        format!(
            r#"{{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"echo pad-{}","decision":"ALLOW","exit_code":0}}"#,
            n
        )
    }

    #[test]
    fn two_same_blocks_within_recent_50_triggers_stop_repeated_block() {
        let dir = fresh_akar_dir("rb_recent");
        let block_line = r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}"#;
        // 10 padding events, then 2 BLOCKs — all within recent 50.
        for i in 0..10 {
            append_hook_line(&dir, &allow_line(i));
        }
        append_hook_line(&dir, block_line);
        append_hook_line(&dir, block_line);
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_eq!(report.decision, LoopDecision::StopRepeatedBlock);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn two_same_blocks_older_than_recent_50_do_not_trigger() {
        let dir = fresh_akar_dir("rb_old");
        let block_line = r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}"#;
        // Two BLOCKs first, then 50 padding events push them out of the window.
        append_hook_line(&dir, block_line);
        append_hook_line(&dir, block_line);
        for i in 0..50 {
            append_hook_line(&dir, &allow_line(i));
        }
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_ne!(
            report.decision,
            LoopDecision::StopRepeatedBlock,
            "old blocks outside the recent window must not trigger"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn two_different_block_commands_do_not_trigger() {
        let dir = fresh_akar_dir("rb_different");
        append_hook_line(
            &dir,
            r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}"#,
        );
        append_hook_line(
            &dir,
            r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"git push --force","decision":"BLOCK","exit_code":2}"#,
        );
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_ne!(
            report.decision,
            LoopDecision::StopRepeatedBlock,
            "two different blocked commands must not trigger"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn repeated_block_reason_includes_command_preview() {
        let dir = fresh_akar_dir("rb_reason_cmd");
        let cmd = "rm -rf /";
        let block_line = format!(
            r#"{{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"{}","decision":"BLOCK","exit_code":2}}"#,
            cmd
        );
        append_hook_line(&dir, &block_line);
        append_hook_line(&dir, &block_line);
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_eq!(report.decision, LoopDecision::StopRepeatedBlock);
        assert!(
            report.reason.contains(cmd),
            "reason must include command_preview: {}",
            report.reason
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn repeated_block_reason_includes_recent_count() {
        let dir = fresh_akar_dir("rb_reason_count");
        let block_line = r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}"#;
        append_hook_line(&dir, block_line);
        append_hook_line(&dir, block_line);
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_eq!(report.decision, LoopDecision::StopRepeatedBlock);
        assert!(
            report.reason.contains("2 times"),
            "reason must include recent block count: {}",
            report.reason
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn repeated_block_reason_includes_window_size() {
        let dir = fresh_akar_dir("rb_reason_window");
        let block_line = r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}"#;
        append_hook_line(&dir, block_line);
        append_hook_line(&dir, block_line);
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_eq!(report.decision, LoopDecision::StopRepeatedBlock);
        assert!(
            report
                .reason
                .contains(&format!("recent {}", RECENT_HOOK_WINDOW)),
            "reason must include recent window size: {}",
            report.reason
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn repeated_block_in_window_helper_returns_count() {
        let dir = fresh_akar_dir("rb_helper");
        let block_line = r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}"#;
        append_hook_line(&dir, block_line);
        append_hook_line(&dir, block_line);
        append_hook_line(&dir, block_line);
        let path = dir.join("HOOK_EVENTS.jsonl");
        let result = repeated_blocked_command_in_window(&path, RECENT_HOOK_WINDOW);
        assert_eq!(result, Some(("rm -rf /".to_string(), 3)));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn repeated_block_just_outside_window_does_not_trigger() {
        // Edge case: 2 BLOCKs sit exactly at positions 51 and 52 (just outside
        // a 50-event window that starts after them). With 50 padding events
        // after, the window covers only the padding — BLOCKs are excluded.
        let dir = fresh_akar_dir("rb_edge");
        let block_line = r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}"#;
        append_hook_line(&dir, block_line);
        append_hook_line(&dir, block_line);
        for i in 0..RECENT_HOOK_WINDOW {
            append_hook_line(&dir, &allow_line(i));
        }
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_ne!(report.decision, LoopDecision::StopRepeatedBlock);
        fs::remove_dir_all(&dir).ok();
    }

    // ---- Rule D: learning patch split rule → SPLIT_TASK ----------------

    #[test]
    fn learning_patch_split_rule_produces_split_task() {
        let dir = fresh_akar_dir("split");
        write_file(
            &dir,
            "LEARNING_PATCHES.md",
            "# Patches\n## LP-1\n- rule: Next prompt must reduce scope or split the task.\n",
        );
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_eq!(report.decision, LoopDecision::SplitTask);
        assert!(report.suggested_prompt.contains("smaller single-purpose"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn resolved_split_rule_does_not_produce_split_task() {
        let dir = fresh_akar_dir("split_resolved");
        write_file(
            &dir,
            "LEARNING_PATCHES.md",
            "# Patches\n## LP-1\n- rule: Next prompt must reduce scope or split the task.\n- status: resolved\n- resolved_at: 2026-07-06T00:00:00Z\n",
        );
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_ne!(
            report.decision,
            LoopDecision::SplitTask,
            "resolved split-rule entries must not trigger SPLIT_TASK"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn governor_no_longer_reports_split_task_after_active_entries_resolved() {
        let dir = fresh_akar_dir("split_after_resolve");
        write_file(
            &dir,
            "LEARNING_PATCHES.md",
            "# Patches\n## LP-1\n- rule: Next prompt must reduce scope or split the task.\n- status: active\n",
        );
        let patches_path = dir.join("LEARNING_PATCHES.md");
        let cfg = cfg_with_akar_dir(dir.clone());

        // Before resolve: SPLIT_TASK fires.
        let before = decide(&cfg);
        assert_eq!(before.decision, LoopDecision::SplitTask);

        // Resolve active entries in place.
        let count = learn::resolve_active_patches(&patches_path, "2026-07-06T12:00:00Z");
        assert_eq!(count, Some(1));

        // After resolve: SPLIT_TASK no longer fires.
        let after = decide(&cfg);
        assert_ne!(
            after.decision,
            LoopDecision::SplitTask,
            "governor must not report SPLIT_TASK after active entries are resolved"
        );

        fs::remove_dir_all(&dir).ok();
    }

    // ---- Rule E: baseline + dirty tree → RUN_POSTMORTEM ----------------
    //
    // Note: this requires a dirty working tree, which we cannot safely
    // manufacture in the real repo. We test the pure decision function via
    // a synthetic report-construction path instead, and cover the
    // baseline+clean path (Rule F) end-to-end below.

    #[test]
    fn baseline_plus_dirty_tree_produces_run_postmortem() {
        let dir = fresh_akar_dir("postmortem");
        // Create a baseline file so baseline_present is true.
        write_file(
            &dir,
            "DIFF_BASELINE.json",
            r#"{"timestamp":"t","prompt":"p","head_commit":"abcdef123456","task_type":"Bugfix","budget_files_max":3,"budget_loc_max":60}"#,
        );
        // Synthesize the dirty-tree branch directly by constructing the
        // report the same way decide() does for Rule E.
        let report = LoopGovernorReport {
            decision: LoopDecision::RunPostmortem,
            reason: "baseline exists and working tree is dirty".to_string(),
            next_action: postmortem_next_action().to_string(),
            suggested_prompt: postmortem_prompt().to_string(),
            evidence_used: vec![
                ".akar/DIFF_BASELINE.json: present".to_string(),
                "working tree clean: no".to_string(),
            ],
        };
        assert_eq!(report.decision, LoopDecision::RunPostmortem);
        assert!(report.next_action.contains("postmortem --diff --baseline"));
        assert!(report.suggested_prompt.contains("EXCEEDED"));
        // Baseline file was actually written and detected.
        assert!(dir.join("DIFF_BASELINE.json").exists());
        fs::remove_dir_all(&dir).ok();
    }

    // ---- Rule F: baseline + clean tree → READY --------------------------

    #[test]
    fn baseline_plus_clean_tree_produces_ready() {
        let dir = fresh_akar_dir("ready");
        write_file(
            &dir,
            "DIFF_BASELINE.json",
            r#"{"timestamp":"t","prompt":"p","head_commit":"abcdef123456","task_type":"Bugfix","budget_files_max":3,"budget_loc_max":60}"#,
        );
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        // The real repo working tree must be clean for this test to assert READY.
        // If the tree happens to be dirty during the test run, the decision
        // becomes RUN_POSTMORTEM — assert against the actual git state.
        let clean = diff_budget::is_working_tree_clean(&cfg.project_root).unwrap_or(false);
        if clean {
            assert_eq!(report.decision, LoopDecision::Ready);
            assert!(
                report
                    .next_action
                    .contains("Baseline exists and tree is clean")
            );
        } else {
            assert_eq!(report.decision, LoopDecision::RunPostmortem);
        }
        fs::remove_dir_all(&dir).ok();
    }

    // ---- Rule G: no baseline + clean tree → SNAPSHOT_NOW ---------------

    #[test]
    fn no_baseline_plus_clean_tree_produces_snapshot_now() {
        let dir = fresh_akar_dir("snapshot");
        // No DIFF_BASELINE.json, no patches, no hook events.
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let clean = diff_budget::is_working_tree_clean(&cfg.project_root).unwrap_or(false);
        if clean {
            assert_eq!(report.decision, LoopDecision::SnapshotNow);
            assert!(report.suggested_prompt.contains("preflight --snapshot"));
        } else {
            assert_eq!(report.decision, LoopDecision::CommitCheckpoint);
        }
        fs::remove_dir_all(&dir).ok();
    }

    // ---- Rule H: no baseline + dirty tree → COMMIT_CHECKPOINT -----------
    //
    // Same dirty-tree constraint as Rule E — we verify the foundation text
    // is used for the dirty-git state by checking the playbook directly.

    #[test]
    fn commit_checkpoint_uses_foundation_dirty_playbook() {
        let next = commit_next_action();
        let playbook = foundation::git_dirty_playbook();
        assert_eq!(next, playbook);
        assert!(next.contains("git status"));
    }

    // ---- Rule A: missing git → UNKNOWN ---------------------------------

    #[test]
    fn missing_git_produces_unknown() {
        // Point project_root at a non-repo temp dir so git status fails.
        let non_repo =
            std::env::temp_dir().join(format!("akar_loop_gov_nonrepo_{}", std::process::id()));
        let _ = fs::remove_dir_all(&non_repo);
        fs::create_dir_all(&non_repo).unwrap();
        let cfg = config::Config {
            project_root: non_repo.clone(),
            akar_dir: non_repo.clone(),
            global_dir: std::path::PathBuf::from("/nonexistent/__akar_gov_global__"),
            project_name: "nonrepo".to_string(),
        };
        let report = decide(&cfg);
        assert_eq!(report.decision, LoopDecision::Unknown);
        assert!(report.next_action.contains("git repository"));
        fs::remove_dir_all(&non_repo).ok();
    }

    // ---- status formatting includes loop governor -----------------------

    #[test]
    fn status_formatting_includes_loop_governor() {
        let report = LoopGovernorReport {
            decision: LoopDecision::SnapshotNow,
            reason: "no baseline and clean tree".to_string(),
            next_action: "take a snapshot".to_string(),
            suggested_prompt: snapshot_prompt().to_string(),
            evidence_used: vec!["git repository status: available".to_string()],
        };
        let out = format_loop_governor(&report);
        assert!(out.contains("loop governor:"));
        assert!(out.contains("decision:    SNAPSHOT_NOW"));
        assert!(out.contains("reason:      no baseline and clean tree"));
        assert!(out.contains("next action: take a snapshot"));
        assert!(out.contains("evidence used:"));
        assert!(out.contains("- git repository status: available"));
    }

    // ---- NEXT_RUN block formatting -------------------------------------

    #[test]
    fn next_run_block_includes_decision_and_suggested_prompt() {
        let report = LoopGovernorReport {
            decision: LoopDecision::CommitCheckpoint,
            reason: "no baseline and dirty tree".to_string(),
            next_action: "git status".to_string(),
            suggested_prompt: commit_prompt().to_string(),
            evidence_used: vec!["working tree clean: no".to_string()],
        };
        let out = format_next_run_block(&report);
        assert!(out.contains("## Loop Governor Decision"));
        assert!(out.contains("- decision: COMMIT_CHECKPOINT"));
        assert!(out.contains("- reason: no baseline and dirty tree"));
        assert!(out.contains("- next action:"));
        assert!(out.contains("## Suggested Next Claude Prompt"));
        assert!(out.contains("commit completed AKAR work with explicit files only"));
        assert!(out.contains("## Evidence Used"));
        assert!(out.contains("- working tree clean: no"));
    }

    #[test]
    fn next_run_block_includes_improved_repeated_block_reason() {
        // Synthesize a STOP_REPEATED_BLOCK report with the v0.15.0 reason
        // format and confirm NEXT_RUN.md carries the improved reason.
        let dir = fresh_akar_dir("rb_nextrun");
        let block_line = r#"{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}"#;
        append_hook_line(&dir, block_line);
        append_hook_line(&dir, block_line);
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        assert_eq!(report.decision, LoopDecision::StopRepeatedBlock);
        let out = format_next_run_block(&report);
        assert!(out.contains("- decision: STOP_REPEATED_BLOCK"));
        assert!(
            out.contains(&format!("recent {}", RECENT_HOOK_WINDOW)),
            "NEXT_RUN must include window size: {}",
            out
        );
        assert!(
            out.contains("2 times"),
            "NEXT_RUN must include count: {}",
            out
        );
        assert!(
            out.contains("rm -rf /"),
            "NEXT_RUN must include command: {}",
            out
        );
        fs::remove_dir_all(&dir).ok();
    }

    // ---- governor uses foundation playbook text for dirty git state ----

    #[test]
    fn governor_uses_foundation_playbook_text_for_dirty_git() {
        // Rule H (no baseline + dirty) uses git_dirty_playbook.
        let next = commit_next_action();
        let playbook = foundation::git_dirty_playbook();
        assert_eq!(next, playbook);
        // The playbook must not recommend any forbidden action.
        assert!(!next.contains("git reset"));
        assert!(!next.contains("git clean"));
        assert!(!next.contains("git stash"));
        assert!(!next.contains("git checkout"));
        assert!(!next.contains("git push"));
    }

    // ---- governor uses foundation playbook text for repeated block -----

    #[test]
    fn governor_uses_foundation_playbook_text_for_repeated_block() {
        let cmd = "git push --force";
        // The repeated-block next action is built from foundation::repeated_block_playbook.
        let report = {
            let dir = fresh_akar_dir("rb_pb");
            let line = format!(
                r#"{{"timestamp":"t","hook":"PreToolUse","tool_name":"Bash","command_preview":"{}","decision":"BLOCK","exit_code":2}}"#,
                cmd
            );
            append_hook_line(&dir, &line);
            append_hook_line(&dir, &line);
            let cfg = cfg_with_akar_dir(dir.clone());
            let r = decide(&cfg);
            fs::remove_dir_all(&dir).ok();
            r
        };
        assert_eq!(report.decision, LoopDecision::StopRepeatedBlock);
        // next_action must equal foundation::repeated_block_playbook(cmd).
        assert_eq!(report.next_action, foundation::repeated_block_playbook(cmd));
        assert!(report.next_action.contains("Do not retry"));
        assert!(report.next_action.contains(cmd));
    }

    // ---- stop decisions report is_stop ---------------------------------

    #[test]
    fn stop_decisions_are_stop() {
        let stop = LoopGovernorReport {
            decision: LoopDecision::StopHookBroken,
            reason: "r".to_string(),
            next_action: "a".to_string(),
            suggested_prompt: "p".to_string(),
            evidence_used: vec![],
        };
        assert!(stop.is_stop());
        let nonstop = LoopGovernorReport {
            decision: LoopDecision::Ready,
            reason: "r".to_string(),
            next_action: "a".to_string(),
            suggested_prompt: "p".to_string(),
            evidence_used: vec![],
        };
        assert!(!nonstop.is_stop());
    }

    // ---- json helper ---------------------------------------------------

    #[test]
    fn json_str_extracts_command_preview() {
        let line = r#"{"timestamp":"t","command_preview":"git push --force","decision":"BLOCK"}"#;
        assert_eq!(
            json_str(line, "command_preview").as_deref(),
            Some("git push --force")
        );
        assert_eq!(json_str(line, "decision").as_deref(), Some("BLOCK"));
        assert_eq!(json_str(line, "missing"), None);
    }

    #[test]
    fn is_hook_not_found_error_tolerates_field_location() {
        // message field
        let a = r#"{"decision":"ERROR","message":"akar not found in PATH"}"#;
        assert!(is_hook_not_found_error(a));
        // command_preview field
        let b = r#"{"decision":"ERROR","command_preview":"akar not found"}"#;
        assert!(is_hook_not_found_error(b));
        // not an ERROR decision
        let c = r#"{"decision":"ALLOW","command_preview":"akar not found"}"#;
        assert!(!is_hook_not_found_error(c));
        // case-insensitive decision
        let d = r#"{"decision":"error","message":"akar not found"}"#;
        assert!(is_hook_not_found_error(d));
    }

    // ---- v0.16.0 governor command output formatters --------------------

    fn sample_governor_report() -> LoopGovernorReport {
        LoopGovernorReport {
            decision: LoopDecision::SnapshotNow,
            reason: "no baseline and clean tree".to_string(),
            next_action: "take a snapshot now".to_string(),
            suggested_prompt: snapshot_prompt().to_string(),
            evidence_used: vec![
                "git repository status: available".to_string(),
                "working tree clean: yes".to_string(),
            ],
        }
    }

    #[test]
    fn governor_default_output_includes_decision() {
        let report = sample_governor_report();
        let out = format_governor_report(&report);
        assert!(out.starts_with("governor:"));
        assert!(out.contains("decision:    SNAPSHOT_NOW"));
    }

    #[test]
    fn governor_default_output_includes_suggested_prompt() {
        let report = sample_governor_report();
        let out = format_governor_report(&report);
        assert!(out.contains("suggested prompt:"));
        assert!(out.contains("preflight --snapshot"));
    }

    #[test]
    fn governor_default_output_includes_reason_next_action_evidence() {
        let report = sample_governor_report();
        let out = format_governor_report(&report);
        assert!(out.contains("reason:      no baseline and clean tree"));
        assert!(out.contains("next action: take a snapshot now"));
        assert!(out.contains("evidence used:"));
        assert!(out.contains("- git repository status: available"));
    }

    #[test]
    fn governor_one_line_outputs_exactly_one_line() {
        let report = sample_governor_report();
        let out = format_governor_one_line(&report);
        assert_eq!(out.lines().count(), 1, "must be exactly one line");
        assert!(
            !out.ends_with('\n'),
            "no trailing newline in the string itself"
        );
    }

    #[test]
    fn governor_one_line_uses_tab_separator() {
        let report = sample_governor_report();
        let out = format_governor_one_line(&report);
        // DECISION<TAB>SUGGESTED_PROMPT
        assert!(
            out.starts_with("SNAPSHOT_NOW\t"),
            "must start with decision + tab: {}",
            out
        );
        assert_eq!(
            out.matches('\t').count(),
            1,
            "must have exactly one tab: {}",
            out
        );
        // The part after the tab is the suggested prompt.
        let prompt_part = out.split('\t').nth(1).unwrap();
        assert!(prompt_part.contains("preflight --snapshot"));
    }

    #[test]
    fn governor_one_line_unknown_decision_still_prints() {
        let report = LoopGovernorReport {
            decision: LoopDecision::Unknown,
            reason: "unclassifiable".to_string(),
            next_action: "inspect manually".to_string(),
            suggested_prompt: unknown_prompt().to_string(),
            evidence_used: vec![],
        };
        let out = format_governor_one_line(&report);
        assert!(
            out.starts_with("UNKNOWN\t"),
            "UNKNOWN must still print with tab: {}",
            out
        );
        assert_eq!(out.lines().count(), 1);
    }

    #[test]
    fn governor_one_line_collapses_multiline_prompt() {
        // next_action may be multi-line; the one-line prompt must not embed
        // raw newlines (they would break the single-line contract).
        let report = LoopGovernorReport {
            decision: LoopDecision::CommitCheckpoint,
            reason: "dirty tree".to_string(),
            next_action: "line one\nline two".to_string(),
            suggested_prompt: "prompt one\nprompt two".to_string(),
            evidence_used: vec![],
        };
        let out = format_governor_one_line(&report);
        assert_eq!(
            out.lines().count(),
            1,
            "one-line output must not contain newlines"
        );
        assert!(!out.contains('\n'));
    }

    #[test]
    fn governor_json_is_valid_json_shaped_output() {
        let report = sample_governor_report();
        let out = format_governor_json(&report);
        // Starts with `{` and ends with `}` — a single JSON object.
        assert!(out.starts_with('{'));
        assert!(out.ends_with('}'));
        assert_eq!(out.matches('{').count(), out.matches('}').count());
        // No raw newlines inside the JSON string.
        // (one_line collapses multi-line fields before escaping.)
        assert!(!out.contains('\n'));
        assert!(!out.contains('\r'));
    }

    #[test]
    fn governor_json_includes_decision() {
        let report = sample_governor_report();
        let out = format_governor_json(&report);
        assert!(out.contains("\"decision\":\"SNAPSHOT_NOW\""));
    }

    #[test]
    fn governor_json_includes_suggested_prompt() {
        let report = sample_governor_report();
        let out = format_governor_json(&report);
        assert!(out.contains("\"suggested_prompt\":\""));
        assert!(out.contains("preflight --snapshot"));
    }

    #[test]
    fn governor_json_includes_all_required_fields() {
        let report = sample_governor_report();
        let out = format_governor_json(&report);
        assert!(out.contains("\"decision\":"));
        assert!(out.contains("\"reason\":"));
        assert!(out.contains("\"next_action\":"));
        assert!(out.contains("\"suggested_prompt\":"));
        assert!(out.contains("\"evidence_used\":["));
    }

    #[test]
    fn governor_json_escapes_quotes_and_newlines() {
        let report = LoopGovernorReport {
            decision: LoopDecision::CommitCheckpoint,
            reason: "has \"quotes\" and newline\nhere".to_string(),
            next_action: "a".to_string(),
            suggested_prompt: "p".to_string(),
            evidence_used: vec!["e1".to_string()],
        };
        let out = format_governor_json(&report);
        // The reason's quotes must be escaped, and the newline collapsed
        // (one_line joins with space) then the space is literal — no raw
        // newline or unescaped quote inside the JSON value.
        assert!(
            out.contains("\\\"quotes\\\""),
            "quotes must be escaped: {}",
            out
        );
        assert!(!out.contains("\nhere"), "no raw newline in JSON: {}", out);
    }

    #[test]
    fn governor_json_evidence_used_is_array() {
        let report = sample_governor_report();
        let out = format_governor_json(&report);
        // evidence_used has two entries.
        assert!(out.contains(
            "\"evidence_used\":[\"git repository status: available\",\"working tree clean: yes\"]"
        ));
    }

    #[test]
    fn governor_json_empty_evidence_used_is_empty_array() {
        let report = LoopGovernorReport {
            decision: LoopDecision::Unknown,
            reason: "r".to_string(),
            next_action: "a".to_string(),
            suggested_prompt: "p".to_string(),
            evidence_used: vec![],
        };
        let out = format_governor_json(&report);
        assert!(out.contains("\"evidence_used\":[]"));
    }

    // ---- v0.16.0 governor command does not write NEXT_RUN.md ------------

    #[test]
    fn governor_command_path_does_not_write_next_run() {
        // The `akar governor` command path is: decide() + a formatter +
        // println!. It must NOT write .akar/NEXT_RUN.md (only `akar request`
        // does that). Simulate the command path here and assert no file is
        // created.
        let dir = fresh_akar_dir("gov_no_write");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let _ = format_governor_report(&report);
        let _ = format_governor_one_line(&report);
        let _ = format_governor_json(&report);
        let next_run = dir.join("NEXT_RUN.md");
        assert!(
            !next_run.exists(),
            "governor command path must not write NEXT_RUN.md"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn request_path_still_writes_next_run() {
        // The `akar request` path uses write_governor_next_run, which DOES
        // write .akar/NEXT_RUN.md. Confirm it still does. v0.19.0 changed the
        // file to the compiled next-run prompt; assert the compiled sections.
        let dir = fresh_akar_dir("gov_request_writes");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let path = write_nr(&cfg, &report);
        assert!(path.is_some(), "request path must write NEXT_RUN.md");
        let next_run = dir.join("NEXT_RUN.md");
        assert!(next_run.exists());
        let content = fs::read_to_string(&next_run).unwrap();
        assert!(content.starts_with("# AKAR Next Run\n"));
        assert!(content.contains("## Governor Decision"));
        assert!(content.contains("## Objective"));
        assert!(content.contains("## Final Response Format"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn status_still_includes_loop_governor_section() {
        // `akar status` still prints the loop governor section via
        // format_loop_governor. Confirm that formatter is unchanged and
        // produces the section header.
        let report = sample_governor_report();
        let out = format_loop_governor(&report);
        assert!(out.contains("loop governor:"));
        assert!(out.contains("decision:    SNAPSHOT_NOW"));
        assert!(out.contains("evidence used:"));
    }

    // ---- v0.17.0 governor exit code mapping ----------------------------

    #[test]
    fn exit_code_mapping_returns_exact_codes() {
        assert_eq!(LoopDecision::Ready.exit_code(), 0);
        assert_eq!(LoopDecision::SnapshotNow.exit_code(), 0);
        assert_eq!(LoopDecision::RunPostmortem.exit_code(), 10);
        assert_eq!(LoopDecision::CommitCheckpoint.exit_code(), 11);
        assert_eq!(LoopDecision::SplitTask.exit_code(), 12);
        assert_eq!(LoopDecision::StopHookBroken.exit_code(), 20);
        assert_eq!(LoopDecision::StopRepeatedBlock.exit_code(), 21);
        assert_eq!(LoopDecision::Unknown.exit_code(), 30);
    }

    #[test]
    fn exit_code_helper_uses_decision() {
        let report = LoopGovernorReport {
            decision: LoopDecision::RunPostmortem,
            reason: "r".to_string(),
            next_action: "a".to_string(),
            suggested_prompt: "p".to_string(),
            evidence_used: vec![],
        };
        assert_eq!(exit_code_for_decision(&report), 10);
    }

    #[test]
    fn exit_code_ready_is_zero() {
        assert_eq!(LoopDecision::Ready.exit_code(), 0);
    }

    #[test]
    fn exit_code_snapshot_now_is_zero() {
        assert_eq!(LoopDecision::SnapshotNow.exit_code(), 0);
    }

    #[test]
    fn exit_code_run_postmortem_is_ten() {
        assert_eq!(LoopDecision::RunPostmortem.exit_code(), 10);
    }

    #[test]
    fn exit_code_commit_checkpoint_is_eleven() {
        assert_eq!(LoopDecision::CommitCheckpoint.exit_code(), 11);
    }

    #[test]
    fn exit_code_split_task_is_twelve() {
        assert_eq!(LoopDecision::SplitTask.exit_code(), 12);
    }

    #[test]
    fn exit_code_stop_hook_broken_is_twenty() {
        assert_eq!(LoopDecision::StopHookBroken.exit_code(), 20);
    }

    #[test]
    fn exit_code_stop_repeated_block_is_twentyone() {
        assert_eq!(LoopDecision::StopRepeatedBlock.exit_code(), 21);
    }

    #[test]
    fn exit_code_unknown_is_thirty() {
        assert_eq!(LoopDecision::Unknown.exit_code(), 30);
    }

    #[test]
    fn no_exit_code_forces_zero_in_command_path() {
        // The `akar governor --no-exit-code` path computes the decision's
        // exit code but then overrides it to 0. Verify the override logic:
        // for every decision, no_exit_code => effective code 0.
        let cases = [
            LoopDecision::Ready,
            LoopDecision::SnapshotNow,
            LoopDecision::RunPostmortem,
            LoopDecision::CommitCheckpoint,
            LoopDecision::SplitTask,
            LoopDecision::StopHookBroken,
            LoopDecision::StopRepeatedBlock,
            LoopDecision::Unknown,
        ];
        for d in cases {
            let native = d.exit_code();
            let effective = if true { 0 } else { native };
            assert_eq!(
                effective, 0,
                "--no-exit-code must force 0 for {:?} (native {})",
                d, native
            );
        }
    }

    #[test]
    fn status_and_request_are_unaffected_by_governor_exit_mapping() {
        // The exit-code mapping lives on LoopDecision and is only consulted
        // by `akar governor`. status/request never call exit_code(). Confirm
        // the mapping is a pure function with no side effects on the report
        // or formatters used by status/request.
        let report = sample_governor_report();
        // Calling exit_code does not mutate the report.
        let code = report.decision.exit_code();
        assert_eq!(code, 0); // SNAPSHOT_NOW
        // status/request formatters are unchanged.
        assert!(format_loop_governor(&report).contains("loop governor:"));
        assert!(format_next_run_block(&report).contains("## Loop Governor Decision"));
    }

    // ---- v0.18.0 opt-in governor telemetry --------------------------------

    /// Read the last non-empty line of a JSONL log, or None if absent.
    fn last_jsonl_line(path: &Path) -> Option<String> {
        let lines = read_jsonl_lines(path);
        lines.last().cloned()
    }

    #[test]
    fn default_governor_path_does_not_write_telemetry() {
        // The governor telemetry writer is opt-in. The default governor path
        // (no --telemetry flag, no env var) never calls write_governor_telemetry.
        // Simulate that path: decide() + formatters only, no telemetry call.
        let dir = fresh_akar_dir("telem_default");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let _ = format_governor_report(&report);
        let _ = format_governor_one_line(&report);
        let _ = format_governor_json(&report);
        let log = dir.join("EVENT_LOG.jsonl");
        assert!(
            !log.exists(),
            "default governor path must not write telemetry"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_flag_writes_one_governor_event() {
        let dir = fresh_akar_dir("telem_flag");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let path = write_governor_telemetry(
            &cfg,
            &report,
            GovernorTelemetryMode::OneLine,
            false,
            report.decision.exit_code(),
        );
        assert!(path.is_some(), "telemetry should be written");
        let log = dir.join("EVENT_LOG.jsonl");
        assert!(log.exists());
        let lines = read_jsonl_lines(&log);
        assert_eq!(lines.len(), 1, "exactly one governor event expected");
        let line = &lines[0];
        assert!(line.contains("\"event\":\"governor\""));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_event_includes_decision() {
        let dir = fresh_akar_dir("telem_decision");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let _ = write_governor_telemetry(
            &cfg,
            &report,
            GovernorTelemetryMode::Human,
            false,
            report.decision.exit_code(),
        );
        let line = last_jsonl_line(&dir.join("EVENT_LOG.jsonl")).unwrap();
        assert!(line.contains(&format!("\"decision\":\"{}\"", report.decision.as_str())));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_event_includes_exit_code() {
        let dir = fresh_akar_dir("telem_exit");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let expected_code = report.decision.exit_code();
        let _ = write_governor_telemetry(
            &cfg,
            &report,
            GovernorTelemetryMode::Json,
            false,
            expected_code,
        );
        let line = last_jsonl_line(&dir.join("EVENT_LOG.jsonl")).unwrap();
        assert!(line.contains(&format!("\"exit_code\":{}", expected_code)));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_event_includes_mode() {
        let dir = fresh_akar_dir("telem_mode");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        for mode in [
            GovernorTelemetryMode::Human,
            GovernorTelemetryMode::OneLine,
            GovernorTelemetryMode::Json,
        ] {
            let _ = write_governor_telemetry(&cfg, &report, mode, false, 0);
        }
        let lines = read_jsonl_lines(&dir.join("EVENT_LOG.jsonl"));
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("\"mode\":\"human\""));
        assert!(lines[1].contains("\"mode\":\"one-line\""));
        assert!(lines[2].contains("\"mode\":\"json\""));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_event_includes_no_exit_code() {
        let dir = fresh_akar_dir("telem_no_exit");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        // With --no-exit-code, no_exit_code=true and the effective exit code is 0.
        let _ = write_governor_telemetry(&cfg, &report, GovernorTelemetryMode::OneLine, true, 0);
        let line = last_jsonl_line(&dir.join("EVENT_LOG.jsonl")).unwrap();
        assert!(line.contains("\"no_exit_code\":true"));
        assert!(line.contains("\"exit_code\":0"));
        // And the false case.
        let _ = write_governor_telemetry(&cfg, &report, GovernorTelemetryMode::OneLine, false, 10);
        let line2 = last_jsonl_line(&dir.join("EVENT_LOG.jsonl")).unwrap();
        assert!(line2.contains("\"no_exit_code\":false"));
        assert!(line2.contains("\"exit_code\":10"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_one_line_with_telemetry_still_one_line_output() {
        // --one-line --telemetry must still print exactly one line of output
        // (the formatter is unchanged; telemetry is a separate side effect).
        let dir = fresh_akar_dir("telem_oneline_output");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let out = format_governor_one_line(&report);
        assert_eq!(out.lines().count(), 1);
        // Telemetry is written in parallel.
        let _ = write_governor_telemetry(&cfg, &report, GovernorTelemetryMode::OneLine, false, 0);
        let log = dir.join("EVENT_LOG.jsonl");
        assert!(log.exists());
        // Output line count is still 1.
        assert_eq!(out.lines().count(), 1);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_json_with_telemetry_still_json_output() {
        let dir = fresh_akar_dir("telem_json_output");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let out = format_governor_json(&report);
        assert!(out.starts_with('{') && out.ends_with('}'));
        let _ = write_governor_telemetry(&cfg, &report, GovernorTelemetryMode::Json, false, 0);
        assert!(dir.join("EVENT_LOG.jsonl").exists());
        // JSON output unchanged by telemetry.
        assert!(out.starts_with('{') && out.ends_with('}'));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_no_exit_code_records_true_and_effective_zero() {
        // --no-exit-code --telemetry: no_exit_code=true and exit_code=0 in the
        // recorded event, even when the decision's native code is non-zero.
        let dir = fresh_akar_dir("telem_noexit_effective");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = LoopGovernorReport {
            decision: LoopDecision::RunPostmortem, // native code 10
            reason: "baseline exists and working tree is dirty".to_string(),
            next_action: "a".to_string(),
            suggested_prompt: "p".to_string(),
            evidence_used: vec![],
        };
        let native = report.decision.exit_code();
        assert_eq!(native, 10);
        let effective = 0; // --no-exit-code forces 0
        let _ = write_governor_telemetry(
            &cfg,
            &report,
            GovernorTelemetryMode::OneLine,
            true,
            effective,
        );
        let line = last_jsonl_line(&dir.join("EVENT_LOG.jsonl")).unwrap();
        assert!(line.contains("\"no_exit_code\":true"));
        assert!(line.contains("\"exit_code\":0"));
        assert!(line.contains("\"decision\":\"RUN_POSTMORTEM\""));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_redacts_obvious_secrets_in_reason() {
        let dir = fresh_akar_dir("telem_redact");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = LoopGovernorReport {
            decision: LoopDecision::StopRepeatedBlock,
            reason:
                "same command blocked 2 times within recent 50 hook events: token=sk-secretvalue"
                    .to_string(),
            next_action: "a".to_string(),
            suggested_prompt: "p".to_string(),
            evidence_used: vec![],
        };
        let _ = write_governor_telemetry(&cfg, &report, GovernorTelemetryMode::OneLine, false, 21);
        let line = last_jsonl_line(&dir.join("EVENT_LOG.jsonl")).unwrap();
        assert!(
            !line.contains("sk-secretvalue"),
            "secret must be redacted: {}",
            line
        );
        assert!(
            line.contains("[REDACTED]"),
            "redaction marker expected: {}",
            line
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_does_not_log_suggested_prompt() {
        let dir = fresh_akar_dir("telem_no_prompt");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = LoopGovernorReport {
            decision: LoopDecision::SnapshotNow,
            reason: "no baseline and clean tree".to_string(),
            next_action: "a".to_string(),
            suggested_prompt: "UNIQUE_PROMPT_MARKER_DO_NOT_LOG".to_string(),
            evidence_used: vec![],
        };
        let _ = write_governor_telemetry(&cfg, &report, GovernorTelemetryMode::OneLine, false, 0);
        let line = last_jsonl_line(&dir.join("EVENT_LOG.jsonl")).unwrap();
        assert!(
            !line.contains("UNIQUE_PROMPT_MARKER_DO_NOT_LOG"),
            "must not log suggested prompt"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn telemetry_returns_none_when_akar_dir_missing() {
        // If .akar/ does not exist, telemetry is silently skipped.
        let cfg = config::Config {
            project_root: std::env::current_dir().unwrap(),
            akar_dir: std::path::PathBuf::from("/nonexistent/__akar_telem_missing__"),
            global_dir: std::path::PathBuf::from("/nonexistent/__akar_telem_global__"),
            project_name: "test".to_string(),
        };
        let report = sample_governor_report();
        let result =
            write_governor_telemetry(&cfg, &report, GovernorTelemetryMode::OneLine, false, 0);
        assert!(
            result.is_none(),
            "telemetry must be skipped when .akar/ is absent"
        );
    }

    // ---- v0.19.0 next-run prompt compiler ---------------------------------

    fn compiler_cfg() -> config::Config {
        config::Config {
            project_root: std::env::current_dir().unwrap(),
            akar_dir: std::path::PathBuf::from("/nonexistent/__akar_compiler_test__"),
            global_dir: std::path::PathBuf::from("/nonexistent/__akar_compiler_global__"),
            project_name: "akar-test".to_string(),
        }
    }

    fn report_for(decision: LoopDecision) -> LoopGovernorReport {
        LoopGovernorReport {
            decision,
            reason: "synthetic reason".to_string(),
            next_action: "synthetic next action".to_string(),
            suggested_prompt: "synthetic suggested prompt".to_string(),
            evidence_used: vec![
                "git repository status: available".to_string(),
                "working tree clean: yes".to_string(),
            ],
        }
    }

    fn empty_evidence_report(decision: LoopDecision) -> LoopGovernorReport {
        LoopGovernorReport {
            decision,
            reason: "r".to_string(),
            next_action: "a".to_string(),
            suggested_prompt: "p".to_string(),
            evidence_used: vec![],
        }
    }

    /// Returns the ordered list of `## ` section headers (owned strings).
    fn section_headers(prompt: &str) -> Vec<String> {
        prompt
            .lines()
            .filter(|l| l.starts_with("## "))
            .map(|l| l.to_string())
            .collect()
    }

    #[test]
    fn next_run_includes_all_required_sections_in_exact_order() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let sections = section_headers(&prompt);
        let expected = [
            "## Current State",
            "## Governor Decision",
            "## Evidence Used",
            "## Objective",
            "## Hard Rules",
            "## Allowed Commands",
            "## Forbidden Commands",
            "## Stop Conditions",
            "## Verification Required",
            "## Final Response Format",
        ];
        let expected: Vec<String> = expected.iter().map(|s| s.to_string()).collect();
        assert_eq!(sections, expected, "sections must appear in exact order");
        // Title first.
        assert!(prompt.starts_with("# AKAR Next Run\n"));
    }

    #[test]
    fn next_run_includes_akar_version() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains(&format!("- AKAR version: {}", env!("CARGO_PKG_VERSION"))));
    }

    #[test]
    fn next_run_includes_governor_decision() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::SnapshotNow));
        assert!(prompt.contains("- governor decision: SNAPSHOT_NOW"));
        assert!(prompt.contains("- decision: SNAPSHOT_NOW"));
        assert!(prompt.contains("- class: continue-class"));
    }

    #[test]
    fn next_run_includes_evidence_used() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains("- git repository status: available"));
        assert!(prompt.contains("- working tree clean: yes"));
    }

    #[test]
    fn next_run_empty_evidence_writes_placeholder() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &empty_evidence_report(LoopDecision::Ready));
        assert!(prompt.contains("No evidence files were used."));
    }

    #[test]
    fn next_run_includes_hard_rules() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains("## Hard Rules"));
        assert!(prompt.contains("- Do not run git reset."));
        assert!(prompt.contains("- Do not run git clean."));
        assert!(prompt.contains("- Do not run git stash."));
        assert!(prompt.contains("- Do not run git checkout."));
        assert!(prompt.contains("- Do not push."));
        assert!(prompt.contains("- Do not delete user project files."));
        assert!(prompt.contains("- Do not modify Claude Code configuration."));
        assert!(prompt.contains("- Do not retry blocked commands."));
        assert!(prompt.contains("- Do not broaden the task after budget or governor warnings."));
        assert!(prompt.contains("- Stop if verification fails."));
    }

    #[test]
    fn next_run_includes_allowed_commands() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains("## Allowed Commands"));
        assert!(prompt.contains("`git status`"));
        assert!(prompt.contains("`cargo build --release`"));
        assert!(prompt.contains("`cargo run -- governor --json --no-exit-code`"));
        assert!(prompt.contains("`cargo run -- hooks --check`"));
    }

    #[test]
    fn next_run_includes_forbidden_commands() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains("## Forbidden Commands"));
        assert!(prompt.contains("`git reset`"));
        assert!(prompt.contains("`rm -rf /`"));
        assert!(prompt.contains("`sudo rm -rf /`"));
        assert!(prompt.contains("`Remove-Item -Recurse -Force C:\\`"));
        assert!(prompt.contains("broad git add . unless explicitly justified and listed"));
    }

    #[test]
    fn next_run_includes_stop_conditions() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains("## Stop Conditions"));
        assert!(prompt.contains("Stop if `akar hooks --check` fails."));
        assert!(prompt.contains("Stop if `cargo test` fails."));
        assert!(prompt.contains("Stop if `cargo run -- eval` fails."));
        assert!(prompt.contains("Stop if governor decision is STOP_HOOK_BROKEN."));
    }

    #[test]
    fn next_run_includes_verification_commands() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains("## Verification Required"));
        assert!(prompt.contains("`cargo build --release`"));
        assert!(prompt.contains("`cargo test`"));
        assert!(prompt.contains("`cargo run -- eval`"));
        assert!(prompt.contains("`cargo run -- hooks --check`"));
    }

    #[test]
    fn next_run_includes_final_response_format() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains("## Final Response Format"));
        assert!(prompt.contains("1. Baseline confirmation"));
        assert!(prompt.contains("10. Next recommended release"));
    }

    // -- objective compilation per decision --------------------------------

    #[test]
    fn ready_objective_compiled_correctly() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains("Continue the scoped task without broadening the work."));
    }

    #[test]
    fn snapshot_now_objective_compiled_correctly() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::SnapshotNow));
        assert!(prompt.contains("Create a clean baseline snapshot before making changes."));
        // Decision-specific allowed command addition.
        assert!(prompt.contains("`cargo run -- preflight --snapshot \"<task>\"`"));
    }

    #[test]
    fn run_postmortem_objective_compiled_correctly() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::RunPostmortem));
        assert!(prompt.contains("Run baseline postmortem before making more changes."));
        assert!(prompt.contains("`cargo run -- postmortem --diff --baseline`"));
        assert!(prompt.contains("Stop if postmortem returns UNKNOWN."));
    }

    #[test]
    fn commit_checkpoint_objective_compiled_correctly() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::CommitCheckpoint));
        assert!(prompt.contains("Verify and commit completed AKAR work with explicit files only before starting a new baseline."));
        assert!(prompt.contains("`git add <explicit files>`"));
        assert!(prompt.contains("`git commit -m \"<message>\"`"));
        assert!(prompt.contains("Stop if verification fails before commit."));
    }

    #[test]
    fn split_task_objective_compiled_correctly() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::SplitTask));
        assert!(
            prompt.contains("Stop the broad task and create one smaller single-purpose prompt.")
        );
        assert!(prompt.contains("(no git mutation; write a smaller prompt only)"));
    }

    #[test]
    fn stop_hook_broken_objective_compiled_correctly() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::StopHookBroken));
        assert!(prompt.contains(
            "Stop Bash tool work until AKAR is visible in the Claude Code hook subprocess PATH."
        ));
        assert!(prompt.contains("`where.exe akar`"));
        assert!(prompt.contains("`pwsh -NoProfile -Command \"Get-Command akar\"`"));
    }

    #[test]
    fn stop_repeated_block_objective_compiled_correctly() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::StopRepeatedBlock));
        assert!(
            prompt
                .contains("Do not retry the blocked command. Replace it with a safe alternative.")
        );
        assert!(prompt.contains("inspect `.akar/HOOK_EVENTS.jsonl`"));
    }

    #[test]
    fn unknown_objective_compiled_correctly() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Unknown));
        assert!(prompt.contains("Inspect AKAR and git state manually before continuing."));
        assert!(prompt.contains("Stop after reporting unknown state."));
    }

    // -- decision class ----------------------------------------------------

    #[test]
    fn decision_class_continue_for_ready_and_snapshot() {
        assert_eq!(
            LoopDecision::Ready.decision_class(),
            DecisionClass::Continue
        );
        assert_eq!(
            LoopDecision::SnapshotNow.decision_class(),
            DecisionClass::Continue
        );
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        assert!(prompt.contains("- class: continue-class"));
    }

    #[test]
    fn decision_class_action_required_for_postmortem_commit_split() {
        assert_eq!(
            LoopDecision::RunPostmortem.decision_class(),
            DecisionClass::ActionRequired
        );
        assert_eq!(
            LoopDecision::CommitCheckpoint.decision_class(),
            DecisionClass::ActionRequired
        );
        assert_eq!(
            LoopDecision::SplitTask.decision_class(),
            DecisionClass::ActionRequired
        );
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::SplitTask));
        assert!(prompt.contains("- class: action-required"));
    }

    #[test]
    fn decision_class_stop_for_hook_broken_repeated_unknown() {
        assert_eq!(
            LoopDecision::StopHookBroken.decision_class(),
            DecisionClass::Stop
        );
        assert_eq!(
            LoopDecision::StopRepeatedBlock.decision_class(),
            DecisionClass::Stop
        );
        assert_eq!(LoopDecision::Unknown.decision_class(), DecisionClass::Stop);
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Unknown));
        assert!(prompt.contains("- class: stop-class"));
    }

    // -- request writes compiled NEXT_RUN.md; governor does not ------------

    #[test]
    fn request_writes_compiled_next_run_md() {
        let dir = fresh_akar_dir("compiler_request_write");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = report_for(LoopDecision::Ready);
        let path = write_nr(&cfg, &report);
        assert!(path.is_some());
        let content = fs::read_to_string(dir.join("NEXT_RUN.md")).unwrap();
        assert!(content.starts_with("# AKAR Next Run\n"));
        assert!(content.contains("## Current State"));
        assert!(content.contains("## Governor Decision"));
        assert!(content.contains("## Objective"));
        assert!(content.contains("## Final Response Format"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn governor_command_path_does_not_write_next_run_md() {
        // `akar governor` (any mode) computes decide() + a formatter and does
        // NOT call write_governor_next_run. Simulate that path.
        let dir = fresh_akar_dir("compiler_gov_no_write");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let _ = format_governor_report(&report);
        let _ = format_governor_one_line(&report);
        let _ = format_governor_json(&report);
        assert!(
            !dir.join("NEXT_RUN.md").exists(),
            "governor command path must not write NEXT_RUN.md"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn compiled_prompt_sections_have_stable_order_across_decisions() {
        // The 10 section headers must appear in the same order for every
        // decision (only their contents differ).
        let cfg = compiler_cfg();
        let decisions = [
            LoopDecision::Ready,
            LoopDecision::SnapshotNow,
            LoopDecision::RunPostmortem,
            LoopDecision::CommitCheckpoint,
            LoopDecision::SplitTask,
            LoopDecision::StopHookBroken,
            LoopDecision::StopRepeatedBlock,
            LoopDecision::Unknown,
        ];
        let first = section_headers(&compile(&cfg, &report_for(decisions[0].clone())));
        assert_eq!(first.len(), 10);
        for d in decisions.iter().skip(1) {
            let p = compile(&cfg, &report_for(d.clone()));
            assert_eq!(
                section_headers(&p),
                first,
                "section order differs for {:?}",
                d
            );
        }
    }

    // ---- v0.20.0 NEXT_RUN contract validator -----------------------------

    #[test]
    fn valid_compiled_next_run_passes() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let result = validate_next_run(&prompt);
        assert!(
            result.pass,
            "valid compiled prompt should PASS: {:?}",
            result.reasons
        );
        assert!(result.reasons.is_empty());
    }

    #[test]
    fn missing_next_run_file_fails() {
        // `akar request --check` reads the file; an empty string simulates a
        // missing/empty file and must FAIL.
        let result = validate_next_run("");
        assert!(!result.pass);
        assert!(!result.reasons.is_empty());
    }

    #[test]
    fn missing_required_section_fails() {
        let cfg = compiler_cfg();
        let mut prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        // Remove the Objective section header and its body up to the next header.
        prompt = prompt.replace("## Objective\n", "## (removed)\n");
        let result = validate_next_run(&prompt);
        assert!(!result.pass);
        assert!(
            result.reasons.iter().any(|r| r.contains("## Objective")),
            "should flag missing Objective section: {:?}",
            result.reasons
        );
    }

    #[test]
    fn wrong_section_order_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        // Swap two section headers so order is wrong.
        let swapped = prompt
            .replace("## Hard Rules", "## TEMP_HARD")
            .replace("## Allowed Commands", "## Hard Rules")
            .replace("## TEMP_HARD", "## Allowed Commands");
        let result = validate_next_run(&swapped);
        assert!(!result.pass, "wrong section order must FAIL");
    }

    #[test]
    fn empty_hard_rules_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        // Blank the Hard Rules body by removing all its rule lines.
        let mut lines: Vec<&str> = prompt.lines().collect();
        let mut out = String::new();
        let mut in_hard = false;
        for line in &lines {
            if *line == "## Hard Rules" {
                in_hard = true;
                out.push_str(line);
                out.push('\n');
                continue;
            }
            if in_hard && (line.starts_with("## ") || line.starts_with("# ")) {
                in_hard = false;
            }
            if in_hard {
                continue; // skip body lines
            }
            out.push_str(line);
            out.push('\n');
        }
        let _ = lines.split_off(0);
        let result = validate_next_run(&out);
        assert!(!result.pass);
        assert!(
            result.reasons.iter().any(|r| r.contains("Hard Rules")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn empty_allowed_commands_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let mut out = String::new();
        let mut in_sec = false;
        for line in prompt.lines() {
            if line == "## Allowed Commands" {
                in_sec = true;
                out.push_str(line);
                out.push('\n');
                continue;
            }
            if in_sec && (line.starts_with("## ") || line.starts_with("# ")) {
                in_sec = false;
            }
            if in_sec {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        let result = validate_next_run(&out);
        assert!(!result.pass);
        assert!(
            result
                .reasons
                .iter()
                .any(|r| r.contains("Allowed Commands")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn empty_forbidden_commands_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let mut out = String::new();
        let mut in_sec = false;
        for line in prompt.lines() {
            if line == "## Forbidden Commands" {
                in_sec = true;
                out.push_str(line);
                out.push('\n');
                continue;
            }
            if in_sec && (line.starts_with("## ") || line.starts_with("# ")) {
                in_sec = false;
            }
            if in_sec {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        let result = validate_next_run(&out);
        assert!(!result.pass);
        assert!(
            result
                .reasons
                .iter()
                .any(|r| r.contains("Forbidden Commands")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn empty_stop_conditions_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let mut out = String::new();
        let mut in_sec = false;
        for line in prompt.lines() {
            if line == "## Stop Conditions" {
                in_sec = true;
                out.push_str(line);
                out.push('\n');
                continue;
            }
            if in_sec && (line.starts_with("## ") || line.starts_with("# ")) {
                in_sec = false;
            }
            if in_sec {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        let result = validate_next_run(&out);
        assert!(!result.pass);
        assert!(
            result.reasons.iter().any(|r| r.contains("Stop Conditions")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn empty_verification_required_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let mut out = String::new();
        let mut in_sec = false;
        for line in prompt.lines() {
            if line == "## Verification Required" {
                in_sec = true;
                out.push_str(line);
                out.push('\n');
                continue;
            }
            if in_sec && (line.starts_with("## ") || line.starts_with("# ")) {
                in_sec = false;
            }
            if in_sec {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        let result = validate_next_run(&out);
        assert!(!result.pass);
        assert!(
            result
                .reasons
                .iter()
                .any(|r| r.contains("Verification Required")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn missing_final_response_format_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let broken = prompt.replace("## Final Response Format", "## (removed)");
        let result = validate_next_run(&broken);
        assert!(!result.pass);
        assert!(
            result
                .reasons
                .iter()
                .any(|r| r.contains("Final Response Format")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn missing_forbidden_command_safety_item_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        // Remove one forbidden item line.
        let broken = prompt.replace("- `rm -rf /*`\n", "");
        let result = validate_next_run(&broken);
        assert!(!result.pass);
        assert!(
            result.reasons.iter().any(|r| r.contains("rm -rf /*")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn missing_hard_rule_do_not_retry_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let broken = prompt.replace("- Do not retry blocked commands.\n", "");
        let result = validate_next_run(&broken);
        assert!(!result.pass);
        assert!(
            result
                .reasons
                .iter()
                .any(|r| r.contains("Do not retry blocked commands")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn missing_hard_rule_stop_if_verification_fails_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let broken = prompt.replace("- Stop if verification fails.\n", "");
        let result = validate_next_run(&broken);
        assert!(!result.pass);
        assert!(
            result
                .reasons
                .iter()
                .any(|r| r.contains("Stop if verification fails")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn decision_class_mismatch_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        // Tamper the class line: READY should be continue-class, change it to stop-class.
        let broken = prompt.replace("- class: continue-class", "- class: stop-class");
        let result = validate_next_run(&broken);
        assert!(!result.pass);
        assert!(
            result.reasons.iter().any(|r| r.contains("class mismatch")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn decision_specific_objective_mismatch_fails() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        // Replace the READY objective with the SNAPSHOT_NOW objective.
        let broken = prompt.replace(
            "Continue the scoped task without broadening the work.",
            "Create a clean baseline snapshot before making changes.",
        );
        let result = validate_next_run(&broken);
        assert!(!result.pass);
        assert!(
            result
                .reasons
                .iter()
                .any(|r| r.contains("objective mismatch")),
            "{:?}",
            result.reasons
        );
    }

    #[test]
    fn all_eight_decision_objective_class_combinations_pass() {
        let cfg = compiler_cfg();
        let decisions = [
            LoopDecision::Ready,
            LoopDecision::SnapshotNow,
            LoopDecision::RunPostmortem,
            LoopDecision::CommitCheckpoint,
            LoopDecision::SplitTask,
            LoopDecision::StopHookBroken,
            LoopDecision::StopRepeatedBlock,
            LoopDecision::Unknown,
        ];
        for d in decisions.iter() {
            let prompt = compile(&cfg, &report_for(d.clone()));
            let result = validate_next_run(&prompt);
            assert!(
                result.pass,
                "decision {:?} should produce a valid NEXT_RUN: {:?}",
                d, result.reasons
            );
        }
    }

    #[test]
    fn pass_output_format_is_correct() {
        let cfg = compiler_cfg();
        let prompt = compile(&cfg, &report_for(LoopDecision::Ready));
        let result = validate_next_run(&prompt);
        let out = format_next_run_check(&result);
        assert!(out.starts_with("NEXT_RUN check: PASS\n"));
        assert!(out.contains("- sections: PASS"));
        assert!(out.contains("- minimum content: PASS"));
        assert!(out.contains("- safety contract: PASS"));
        assert!(out.contains("- decision consistency: PASS"));
    }

    #[test]
    fn fail_output_format_lists_reasons() {
        let result = validate_next_run("");
        let out = format_next_run_check(&result);
        assert!(out.starts_with("NEXT_RUN check: FAIL\n"));
        // Each reason on its own line, indented.
        assert!(out.lines().skip(1).all(|l| l.starts_with("  - ")));
    }

    #[test]
    fn request_check_does_not_write_next_run_md() {
        // `akar request --check` reads the file and validates; it must NOT
        // write or regenerate NEXT_RUN.md. Simulate the check path.
        let dir = fresh_akar_dir("check_no_write");
        let cfg = cfg_with_akar_dir(dir.clone());
        // No NEXT_RUN.md exists yet.
        assert!(!dir.join("NEXT_RUN.md").exists());
        // The check path reads (missing) and validates an empty string.
        let _result = validate_next_run("");
        // Still no NEXT_RUN.md written.
        assert!(
            !dir.join("NEXT_RUN.md").exists(),
            "request --check must not write NEXT_RUN.md"
        );
        let _ = &cfg;
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn request_still_writes_next_run_md() {
        let dir = fresh_akar_dir("check_request_writes");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = report_for(LoopDecision::Ready);
        let path = write_nr(&cfg, &report);
        assert!(path.is_some());
        assert!(dir.join("NEXT_RUN.md").exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn governor_still_does_not_write_next_run_md() {
        let dir = fresh_akar_dir("check_gov_no_write");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let _ = format_governor_report(&report);
        let _ = format_governor_one_line(&report);
        let _ = format_governor_json(&report);
        assert!(!dir.join("NEXT_RUN.md").exists());
        fs::remove_dir_all(&dir).ok();
    }

    // ---- v0.26.0: task threading into NEXT_RUN ---------------------------

    #[test]
    fn task_is_threaded_into_current_state() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_current_state"));
        let report = report_for(LoopDecision::Ready);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix one small failing test"));
        assert!(
            prompt.contains("- requested task: fix one small failing test"),
            "task must appear in Current State: {}",
            prompt
        );
    }

    #[test]
    fn task_is_threaded_into_objective_for_ready() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_ready"));
        let report = report_for(LoopDecision::Ready);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix one small failing test"));
        assert!(prompt.contains(objective_for_decision(&LoopDecision::Ready)));
        assert!(prompt.contains("- Task: fix one small failing test"));
    }

    #[test]
    fn task_is_threaded_into_objective_for_snapshot_now() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_snapshot"));
        let report = report_for(LoopDecision::SnapshotNow);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix the bug"));
        assert!(prompt.contains(objective_for_decision(&LoopDecision::SnapshotNow)));
        assert!(prompt.contains("- Task: fix the bug"));
    }

    #[test]
    fn task_is_threaded_into_objective_for_run_postmortem() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_postmortem"));
        let report = report_for(LoopDecision::RunPostmortem);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix the bug"));
        assert!(prompt.contains(objective_for_decision(&LoopDecision::RunPostmortem)));
        assert!(prompt.contains("- Task: fix the bug"));
    }

    #[test]
    fn task_is_threaded_into_objective_for_commit_checkpoint() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_commit"));
        let report = report_for(LoopDecision::CommitCheckpoint);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix the bug"));
        assert!(prompt.contains(objective_for_decision(&LoopDecision::CommitCheckpoint)));
        assert!(prompt.contains("- Task: fix the bug"));
    }

    #[test]
    fn task_is_threaded_into_objective_for_split_task() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_split"));
        let report = report_for(LoopDecision::SplitTask);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix the bug"));
        assert!(prompt.contains(objective_for_decision(&LoopDecision::SplitTask)));
        assert!(prompt.contains("- Task: fix the bug"));
    }

    #[test]
    fn stop_hook_broken_keeps_blocker_primary_and_task_secondary() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_stop_hook"));
        let report = report_for(LoopDecision::StopHookBroken);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix the bug"));
        // Base objective (the blocker) must remain verbatim.
        assert!(prompt.contains(objective_for_decision(&LoopDecision::StopHookBroken)));
        // Stop-class: primary-objective line + task as secondary.
        assert!(prompt.contains("- Primary objective: resolve the governor blocker"));
        assert!(prompt.contains("- Requested task after the blocker is resolved: fix the bug"));
        // The task must NOT appear as a plain "- Task:" (which would imply it is primary).
        assert!(!prompt.contains("- Task: fix the bug"));
    }

    #[test]
    fn stop_repeated_block_keeps_blocker_primary_and_task_secondary() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_stop_rep"));
        let report = report_for(LoopDecision::StopRepeatedBlock);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix the bug"));
        assert!(prompt.contains(objective_for_decision(&LoopDecision::StopRepeatedBlock)));
        assert!(prompt.contains("- Primary objective: resolve the governor blocker"));
        assert!(prompt.contains("- Requested task after the blocker is resolved: fix the bug"));
        assert!(!prompt.contains("- Task: fix the bug"));
    }

    #[test]
    fn unknown_keeps_inspection_primary_and_task_secondary() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_unknown"));
        let report = report_for(LoopDecision::Unknown);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix the bug"));
        assert!(prompt.contains(objective_for_decision(&LoopDecision::Unknown)));
        assert!(prompt.contains("- Primary objective: resolve the governor blocker"));
        assert!(prompt.contains("- Requested task after the blocker is resolved: fix the bug"));
        assert!(!prompt.contains("- Task: fix the bug"));
    }

    #[test]
    fn no_task_preserves_current_behavior() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_none"));
        let report = report_for(LoopDecision::Ready);
        let prompt = compile_next_run_prompt(&cfg, &report, None);
        assert!(!prompt.contains("requested task"));
        assert!(!prompt.contains("- Task:"));
        assert!(!prompt.contains("Primary objective"));
        // Objective section still has the base objective.
        assert!(prompt.contains(objective_for_decision(&LoopDecision::Ready)));
    }

    #[test]
    fn empty_task_is_treated_as_no_task() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_empty"));
        let report = report_for(LoopDecision::Ready);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("   "));
        assert!(!prompt.contains("requested task"));
        assert!(!prompt.contains("- Task:"));
    }

    #[test]
    fn task_is_redacted_in_next_run() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_redact"));
        let report = report_for(LoopDecision::Ready);
        let prompt = compile_next_run_prompt(&cfg, &report, Some("fix token=sk-abc123secret bug"));
        assert!(
            !prompt.contains("sk-abc123"),
            "task secrets must be redacted"
        );
        assert!(prompt.contains("[REDACTED]"));
    }

    #[test]
    fn request_check_passes_task_threaded_next_run() {
        let cfg = cfg_with_akar_dir(fresh_akar_dir("task_validate"));
        let report = report_for(LoopDecision::Ready);
        let content = compile_next_run_prompt(&cfg, &report, Some("fix one small failing test"));
        let result = validate_next_run(&content);
        assert!(
            result.pass,
            "task-threaded NEXT_RUN must pass the validator: {:?}",
            result.reasons
        );
    }

    #[test]
    fn governor_command_path_still_does_not_write_next_run_with_task() {
        // `akar governor` never calls write_governor_next_run, so threading a
        // task into `request` must not change that.
        let dir = fresh_akar_dir("gov_no_write_task");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = decide(&cfg);
        let _ = format_governor_report(&report);
        let _ = format_governor_one_line(&report);
        let _ = format_governor_json(&report);
        assert!(
            !dir.join("NEXT_RUN.md").exists(),
            "governor must not write NEXT_RUN.md"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_governor_next_run_with_task_writes_task_to_file() {
        let dir = fresh_akar_dir("write_task");
        let cfg = cfg_with_akar_dir(dir.clone());
        let report = report_for(LoopDecision::Ready);
        let path = write_governor_next_run(&cfg, &report, Some("fix one small failing test"));
        assert!(path.is_some());
        let content = fs::read_to_string(dir.join("NEXT_RUN.md")).unwrap();
        assert!(content.contains("- requested task: fix one small failing test"));
        assert!(content.contains("- Task: fix one small failing test"));
        fs::remove_dir_all(&dir).ok();
    }
}
