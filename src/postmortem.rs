//! Postmortem: reads EVENT_LOG.jsonl and produces a compact, honest mission review.
//!
//! Does not invent evidence. Only summarizes what the event log actually contains.

use crate::config;
use crate::event_log;
use std::path::Path;

// ---------------------------------------------------------------------------
// Outcome classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Clean,
    Degraded,
    Failed,
    Unknown,
}

impl Outcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Outcome::Clean => "clean",
            Outcome::Degraded => "degraded",
            Outcome::Failed => "failed",
            Outcome::Unknown => "unknown",
        }
    }
}

// ---------------------------------------------------------------------------
// PostmortemReport
// ---------------------------------------------------------------------------

pub struct PostmortemReport {
    pub exists: bool,
    pub total_events: usize,
    pub mission_count: usize,
    pub latest_summary: Option<String>,
    pub latest_outcome: Outcome,
    pub follow_up: Vec<String>,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// run_postmortem
// ---------------------------------------------------------------------------

/// Analyse the event log at `log_path` and produce a `PostmortemReport`.
/// Never panics. Does not write anything.
pub fn run_postmortem(log_path: &Path) -> PostmortemReport {
    let summary = event_log::summarize_log(log_path, 20);

    if !summary.exists || summary.total_events == 0 {
        return PostmortemReport {
            exists: false,
            total_events: 0,
            mission_count: 0,
            latest_summary: None,
            latest_outcome: Outcome::Unknown,
            follow_up: vec![
                "run 'akar mission \"<prompt>\"' to record your first mission".to_string(),
            ],
            warnings: Vec::new(),
        };
    }

    // Filter to mission events only.
    let mission_lines: Vec<&String> = summary
        .recent
        .iter()
        .filter(|l| l.contains("\"event_type\":\"mission\""))
        .collect();

    let mission_count = mission_lines.len();

    // Grab latest mission line for preview.
    let latest_summary = mission_lines
        .last()
        .map(|l| {
            // Extract the summary field value from JSON.
            extract_json_field(l, "summary").unwrap_or_else(|| truncate(l, 100))
        })
        .map(|s| config::redact(&s));

    // Classify outcome from the latest mission event.
    let latest_outcome = mission_lines
        .last()
        .map(|l| classify_outcome(l))
        .unwrap_or(Outcome::Unknown);

    // Build follow-up suggestions.
    let mut follow_up = Vec::new();
    match &latest_outcome {
        Outcome::Clean => {
            follow_up.push("no action needed — last mission completed cleanly".to_string());
        }
        Outcome::Degraded => {
            follow_up.push("run 'akar doctor' to check for config/memory issues".to_string());
            follow_up.push("run 'akar eval' to verify core behavior is intact".to_string());
        }
        Outcome::Failed => {
            follow_up.push("run 'akar doctor' to diagnose the failure".to_string());
            follow_up.push("run 'akar eval' to verify core behavior is intact".to_string());
            follow_up.push("add a learning patch to .akar/LESSONS.md".to_string());
        }
        Outcome::Unknown => {
            follow_up
                .push("run 'akar mission \"<prompt>\"' to generate a reviewed event".to_string());
        }
    }

    // Detect warnings: failures in the event log.
    let mut warnings = Vec::new();
    let failure_count = summary
        .recent
        .iter()
        .filter(|l| l.contains("\"event\":\"failure\""))
        .count();
    if failure_count > 0 {
        warnings.push(format!("{} failure event(s) in recent log", failure_count));
    }

    PostmortemReport {
        exists: true,
        total_events: summary.total_events,
        mission_count,
        latest_summary,
        latest_outcome,
        follow_up,
        warnings,
    }
}

/// Classify outcome from a single JSONL event line.
fn classify_outcome(line: &str) -> Outcome {
    if line.contains("\"event\":\"failure\"") {
        return Outcome::Failed;
    }
    if line.contains("\"event\":\"warning\"") {
        return Outcome::Degraded;
    }
    if line.contains("\"event\":\"success\"") {
        // Check if warnings were noted in the summary.
        if let Some(s) = extract_json_field(line, "summary") {
            if s.contains("warnings=0") {
                return Outcome::Clean;
            } else if s.contains("warnings=") {
                return Outcome::Degraded;
            }
        }
        return Outcome::Clean;
    }
    Outcome::Unknown
}

/// Extract a JSON string field value by key from a raw JSONL line.
/// Returns None if not found. Simple scan — no parser dependency.
fn extract_json_field(line: &str, key: &str) -> Option<String> {
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

// ---------------------------------------------------------------------------
// format_postmortem_report
// ---------------------------------------------------------------------------

pub fn format_postmortem_report(report: &PostmortemReport) -> String {
    let mut out = String::new();

    if !report.exists || report.total_events == 0 {
        out.push_str("postmortem: no events recorded yet\n");
        out.push_str("  hint: run 'akar mission \"<prompt>\"' to record your first mission\n");
        return out;
    }

    out.push_str(&format!(
        "postmortem: {} total event(s), {} mission(s) in recent log\n",
        report.total_events, report.mission_count
    ));
    out.push_str(&format!("  outcome: {}\n", report.latest_outcome.as_str()));

    if let Some(s) = &report.latest_summary {
        out.push_str(&format!("  latest:  {}\n", truncate(s, 100)));
    }

    if !report.warnings.is_empty() {
        out.push_str("  warnings:\n");
        for w in &report.warnings {
            out.push_str(&format!("    - {}\n", w));
        }
    }

    if !report.follow_up.is_empty() {
        out.push_str("  follow-up:\n");
        for f in &report.follow_up {
            out.push_str(&format!("    - {}\n", f));
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
    use std::fs;

    fn tmp_log(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("akar_pm_test_{}.jsonl", name))
    }

    fn write_event(path: &Path, event: &str, event_type: &str, summary: &str) {
        let entry = event_log::EventEntry {
            ts: "2026-07-04T07:00:00Z".to_string(),
            project: "test".to_string(),
            model: "unknown".to_string(),
            event: event.to_string(),
            event_type: event_type.to_string(),
            summary: summary.to_string(),
            resolution: "done".to_string(),
            confidence: "medium".to_string(),
        };
        event_log::append_event(path, &entry).unwrap();
    }

    #[test]
    fn postmortem_handles_missing_log() {
        let path = tmp_log("missing");
        let _ = fs::remove_file(&path);
        let report = run_postmortem(&path);
        assert!(!report.exists);
        assert_eq!(report.total_events, 0);
        assert_eq!(report.latest_outcome, Outcome::Unknown);
    }

    #[test]
    fn postmortem_summarizes_one_event() {
        let path = tmp_log("one");
        let _ = fs::remove_file(&path);
        write_event(
            &path,
            "success",
            "mission",
            "mission/done task=Bugfix risk=Low autonomy=A5 warnings=0 prompt=fix button",
        );
        let report = run_postmortem(&path);
        assert!(report.exists);
        assert_eq!(report.total_events, 1);
        assert_eq!(report.mission_count, 1);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn postmortem_summarizes_multiple_events() {
        let path = tmp_log("multi");
        let _ = fs::remove_file(&path);
        write_event(
            &path,
            "success",
            "mission",
            "mission/done task=Bugfix risk=Low autonomy=A5 warnings=0 prompt=fix a",
        );
        write_event(
            &path,
            "success",
            "mission",
            "mission/done task=Feature risk=Low autonomy=A5 warnings=0 prompt=add b",
        );
        write_event(
            &path,
            "failure",
            "mission",
            "mission/failed task=Feature risk=High autonomy=A5 warnings=2 prompt=risky c",
        );
        let report = run_postmortem(&path);
        assert_eq!(report.total_events, 3);
        assert_eq!(report.mission_count, 3);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn postmortem_classifies_clean_outcome() {
        let path = tmp_log("clean");
        let _ = fs::remove_file(&path);
        write_event(
            &path,
            "success",
            "mission",
            "mission/done task=Bugfix risk=Low autonomy=A5 warnings=0 prompt=fix btn",
        );
        let report = run_postmortem(&path);
        assert_eq!(report.latest_outcome, Outcome::Clean);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn postmortem_classifies_failed_outcome() {
        let path = tmp_log("failed");
        let _ = fs::remove_file(&path);
        write_event(
            &path,
            "failure",
            "mission",
            "mission/failed task=Feature risk=High autonomy=A5 warnings=3 prompt=bad",
        );
        let report = run_postmortem(&path);
        assert_eq!(report.latest_outcome, Outcome::Failed);
        assert!(!report.warnings.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn postmortem_classifies_degraded_outcome() {
        let path = tmp_log("degraded");
        let _ = fs::remove_file(&path);
        write_event(
            &path,
            "success",
            "mission",
            "mission/done task=Bugfix risk=Low autonomy=A5 warnings=2 prompt=fix x",
        );
        let report = run_postmortem(&path);
        assert_eq!(report.latest_outcome, Outcome::Degraded);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn postmortem_redacts_secret_looking_values() {
        let path = tmp_log("redact");
        let _ = fs::remove_file(&path);
        write_event(
            &path,
            "success",
            "mission",
            "mission/done prompt=token=sk-abc123secretvalue warnings=0",
        );
        let report = run_postmortem(&path);
        if let Some(s) = &report.latest_summary {
            assert!(
                !s.contains("sk-abc123"),
                "secret should be redacted in summary"
            );
        }
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn format_postmortem_report_empty_state() {
        let path = tmp_log("fmt_empty");
        let _ = fs::remove_file(&path);
        let report = run_postmortem(&path);
        let out = format_postmortem_report(&report);
        assert!(out.contains("no events recorded"));
    }

    #[test]
    fn format_postmortem_report_with_events() {
        let path = tmp_log("fmt_events");
        let _ = fs::remove_file(&path);
        write_event(
            &path,
            "success",
            "mission",
            "mission/done task=Bugfix risk=Low autonomy=A5 warnings=0 prompt=fix btn",
        );
        let report = run_postmortem(&path);
        let out = format_postmortem_report(&report);
        assert!(out.contains("postmortem:"));
        assert!(out.contains("outcome:"));
        let _ = fs::remove_file(&path);
    }
}
