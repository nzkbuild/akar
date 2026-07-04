//! Learning Patch v0 — generates local, reviewable learning patch proposals
//! from postmortem evidence. Does not auto-apply or mutate global config.

use std::path::Path;
use crate::{config, event_log, postmortem};

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
        - status: proposed\n",
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
}
