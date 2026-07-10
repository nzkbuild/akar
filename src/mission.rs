//! Phase 10: Mission Runtime
//!
//! Walks a user prompt through a structured state machine, producing a
//! `Mission` that captures every decision made along the way.

use crate::config;
use crate::context_pack;
use crate::contract;
use crate::event_log;
use crate::verify;

// ---------------------------------------------------------------------------
// MissionState
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum MissionState {
    Idle,
    Intake,
    Classify,
    BuildContext,
    Contract,
    Execute,
    Verify,
    Review,
    MemoryUpdate,
    Done,
    Failed,
    Blocked,
}

// ---------------------------------------------------------------------------
// Mission
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct Mission {
    pub state: MissionState,
    pub prompt: String,
    pub contract: Option<contract::TaskContract>,
    pub context_pack: Option<context_pack::ContextPack>,
    /// Stringified results from the Verify step.
    pub verify_results: Vec<String>,
    pub warnings: Vec<String>,
    pub event_log: Vec<event_log::EventEntry>,
}

impl Mission {
    fn new() -> Self {
        Mission {
            state: MissionState::Idle,
            prompt: String::new(),
            contract: None,
            context_pack: None,
            verify_results: Vec::new(),
            warnings: Vec::new(),
            event_log: Vec::new(),
        }
    }

    /// Push a simple info entry onto the in-memory event log.
    fn log(&mut self, event_type: &str, summary: &str) {
        self.event_log.push(event_log::EventEntry {
            ts: "2026-07-04T00:00:00Z".to_string(),
            project: String::new(),
            model: String::new(),
            event: "info".to_string(),
            event_type: event_type.to_string(),
            summary: summary.to_string(),
            resolution: String::new(),
            confidence: "medium".to_string(),
        });
    }
}

// ---------------------------------------------------------------------------
// run_mission
// ---------------------------------------------------------------------------

/// Walk `prompt` through the mission state machine and return the final `Mission`.
pub fn run_mission(prompt: &str, cfg: &config::Config) -> Mission {
    let mut m = Mission::new();

    // --- Intake ---
    m.state = MissionState::Intake;
    m.prompt = prompt.to_string();
    m.log("intake", &format!("received prompt: {}", prompt));

    // --- Classify ---
    m.state = MissionState::Classify;
    let tc = contract::classify_prompt(prompt);
    m.log(
        "classify",
        &format!("classified as {:?}, risk {:?}", tc.task_type, tc.risk_level),
    );
    m.contract = Some(tc);

    // --- BuildContext ---
    m.state = MissionState::BuildContext;
    let pack = context_pack::build_pack(cfg);
    let hot_count = pack
        .files
        .iter()
        .filter(|f| f.tier == context_pack::ContextTier::Hot)
        .count();
    m.log(
        "build_context",
        &format!("context pack built: {} hot file(s)", hot_count),
    );
    for w in &pack.warnings {
        m.warnings.push(w.clone());
    }
    m.context_pack = Some(pack);

    // --- Contract ---
    m.state = MissionState::Contract;
    let contract_ok = m
        .contract
        .as_ref()
        .map(|c| c.diff_budget.files_max > 0)
        .unwrap_or(false);

    if !contract_ok {
        m.warnings
            .push("contract sanity check failed: files_max == 0".to_string());
        m.log("contract", "FAILED: contract sanity check");
        m.state = MissionState::Failed;
        write_telemetry_event(&m, cfg);
        return m;
    }
    m.log("contract", "contract sanity check passed");

    // --- Execute ---
    m.state = MissionState::Execute;
    m.log("execute", "execute: skipped in scaffold mode");

    // --- Verify ---
    m.state = MissionState::Verify;
    let project_root = cfg.project_root.clone();
    let recipe = verify::detect_recipe(&project_root);

    let recipe_label = if recipe.commands.is_empty() {
        "manual-only".to_string()
    } else {
        recipe
            .commands
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    m.verify_results.push(format!(
        "scaffold mode (commands not executed): recipe=[{}]",
        recipe_label
    ));
    m.log("verify", &format!("recipe detected: {}", recipe_label));

    // --- Review ---
    m.state = MissionState::Review;
    if let Some(tc) = &m.contract {
        if !tc.stop_conditions.is_empty() {
            m.log(
                "review",
                &format!(
                    "{} stop condition(s) pending review",
                    tc.stop_conditions.len()
                ),
            );
        } else {
            m.log("review", "no stop conditions — review clean");
        }
    }

    // --- MemoryUpdate ---
    m.state = MissionState::MemoryUpdate;
    m.log("memory_update", "memory update: skipped in scaffold mode");

    // --- Done ---
    m.state = MissionState::Done;
    m.log("done", "mission complete");

    write_telemetry_event(&m, cfg);
    m
}

/// Append a compact telemetry event to `.akar/EVENT_LOG.jsonl`.
/// Silently skips if .akar/ does not exist — never crashes the caller.
fn write_telemetry_event(m: &Mission, cfg: &config::Config) {
    // Only write if .akar/ exists — don't auto-create dirs from mission.
    if !cfg.akar_dir.exists() {
        return;
    }
    let log_path = cfg.akar_dir.join("EVENT_LOG.jsonl");

    let (task_type, risk, autonomy) = if let Some(tc) = &m.contract {
        (
            format!("{:?}", tc.task_type),
            format!("{:?}", tc.risk_level),
            format!("{:?}", tc.autonomy),
        )
    } else {
        (
            "unknown".to_string(),
            "unknown".to_string(),
            "unknown".to_string(),
        )
    };

    // Truncate prompt to 80 chars and redact potential secrets.
    let prompt_preview = crate::config::redact(&m.prompt.chars().take(80).collect::<String>());

    let state_str = match m.state {
        MissionState::Done => "done",
        MissionState::Failed => "failed",
        MissionState::Blocked => "blocked",
        _ => "unknown",
    };

    let summary = format!(
        "mission/{} task={} risk={} autonomy={} warnings={} prompt={}",
        state_str,
        task_type,
        risk,
        autonomy,
        m.warnings.len(),
        prompt_preview
    );

    let entry = event_log::EventEntry {
        ts: event_log::now_iso8601(),
        project: cfg.project_name.clone(),
        model: "unknown".to_string(),
        event: if m.state == MissionState::Done {
            "success".to_string()
        } else {
            "failure".to_string()
        },
        event_type: "mission".to_string(),
        summary,
        resolution: state_str.to_string(),
        confidence: "medium".to_string(),
    };

    // Best-effort — ignore write errors so mission output is not disrupted.
    let _ = event_log::append_event(&log_path, &entry);
}

// ---------------------------------------------------------------------------
// format_mission_report
// ---------------------------------------------------------------------------

/// Produce the AKAR final-response format for a completed (or failed) mission.
pub fn format_mission_report(mission: &Mission) -> String {
    let mut out = String::new();

    // Lead with an unmistakable advisory banner. `akar mission` walks the
    // state machine in scaffold mode only — it records a telemetry event and
    // prints strategy, but never executes the task. See v0.22 audit.
    out.push_str(
        "ADVISORY ONLY — `akar mission` walks the state machine in scaffold mode. It does NOT:\n",
    );
    out.push_str("  - execute code\n");
    out.push_str("  - edit files\n");
    out.push_str("  - call models\n");
    out.push_str("  - run the mission\n");
    out.push_str("For a Claude-ready next-run prompt, use `akar request`.\n");
    out.push_str("\n");

    // Header line
    let header = match mission.state {
        MissionState::Done => "Done.",
        MissionState::Failed => "Failed.",
        MissionState::Blocked => "Blocked.",
        ref s => &format!("{:?}.", s),
    };
    out.push_str(&format!("{}\n", header));

    // --- Mission section ---
    out.push_str("\nMission:\n");
    out.push_str(&format!("- prompt: {}\n", mission.prompt));

    if let Some(tc) = &mission.contract {
        out.push_str(&format!("- type: {:?}\n", tc.task_type));
        out.push_str(&format!("- risk: {:?}\n", tc.risk_level));
        out.push_str(&format!("- autonomy: {:?}\n", tc.autonomy));
        out.push_str(&format!(
            "- diff budget: {} files, {} LOC\n",
            tc.diff_budget.files_max, tc.diff_budget.loc_max
        ));
    } else {
        out.push_str("- type: unknown\n");
        out.push_str("- risk: unknown\n");
        out.push_str("- autonomy: unknown\n");
        out.push_str("- diff budget: unknown\n");
    }

    // --- Context section ---
    out.push_str("\nContext:\n");
    if let Some(pack) = &mission.context_pack {
        let hot = pack
            .files
            .iter()
            .filter(|f| f.tier == context_pack::ContextTier::Hot)
            .count();
        out.push_str(&format!("- hot: {} files\n", hot));
    } else {
        out.push_str("- hot: 0 files\n");
    }
    let state_label = match mission.state {
        MissionState::Done => "done",
        MissionState::Failed => "failed",
        MissionState::Blocked => "blocked",
        _ => "in-progress",
    };
    out.push_str(&format!("- state: {}\n", state_label));

    // --- Verified section ---
    out.push_str("\nVerified:\n");
    if mission.verify_results.is_empty() {
        out.push_str("- scaffold mode (commands not executed)\n");
    } else {
        for r in &mission.verify_results {
            out.push_str(&format!("- {}\n", r));
        }
    }

    // --- Not verified section ---
    out.push_str("\nNot verified:\n");
    out.push_str("- actual execution\n");
    out.push_str("- browser click-through\n");

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cfg() -> config::Config {
        config::Config::discover()
    }

    #[test]
    fn run_mission_reaches_done_state() {
        let cfg = test_cfg();
        let m = run_mission("fix the login bug", &cfg);
        assert_eq!(
            m.state,
            MissionState::Done,
            "mission should reach Done, got {:?}",
            m.state
        );
    }

    #[test]
    fn run_mission_produces_a_contract() {
        let cfg = test_cfg();
        let m = run_mission("fix the login bug", &cfg);
        assert!(m.contract.is_some(), "mission should have a contract");
        let tc = m.contract.unwrap();
        assert_eq!(tc.task_type, contract::TaskType::Bugfix);
    }

    #[test]
    fn format_mission_report_contains_expected_sections() {
        let cfg = test_cfg();
        let m = run_mission("fix the login bug", &cfg);
        let report = format_mission_report(&m);

        assert!(report.contains("Done."), "report should start with Done.");
        assert!(
            report.starts_with("ADVISORY ONLY"),
            "mission report must lead with advisory banner"
        );
        assert!(
            report.contains("- run the mission"),
            "banner must state it does not run the mission"
        );
        assert!(
            report.contains("akar request"),
            "banner must point to akar request"
        );
        assert!(
            report.contains("Mission:"),
            "report should contain Mission: section"
        );
        assert!(
            report.contains("Context:"),
            "report should contain Context: section"
        );
        assert!(
            report.contains("Verified:"),
            "report should contain Verified: section"
        );
        assert!(
            report.contains("Not verified:"),
            "report should contain Not verified: section"
        );
        assert!(
            report.contains("fix the login bug"),
            "report should echo the prompt"
        );
    }

    #[test]
    fn security_prompt_gets_high_risk() {
        let cfg = test_cfg();
        let m = run_mission("audit the auth token validation", &cfg);
        assert!(m.contract.is_some());
        let tc = m.contract.unwrap();
        assert_eq!(
            tc.risk_level,
            contract::RiskLevel::High,
            "security/auth prompt should yield High risk"
        );
    }

    #[test]
    fn run_mission_has_context_pack() {
        let cfg = test_cfg();
        let m = run_mission("add dark mode", &cfg);
        assert!(
            m.context_pack.is_some(),
            "mission should have a context pack"
        );
    }

    #[test]
    fn run_mission_event_log_nonempty() {
        let cfg = test_cfg();
        let m = run_mission("refactor the parser", &cfg);
        assert!(
            !m.event_log.is_empty(),
            "event log should have entries after a full mission run"
        );
    }

    #[test]
    fn format_mission_report_shows_failed_when_state_failed() {
        let mut m = Mission::new();
        m.state = MissionState::Failed;
        m.prompt = "some prompt".to_string();
        let report = format_mission_report(&m);
        // The advisory banner leads, then the state header.
        assert!(
            report.contains("Failed."),
            "report should contain Failed. header"
        );
        assert!(
            report.starts_with("ADVISORY ONLY"),
            "report should lead with advisory banner"
        );
    }
}
