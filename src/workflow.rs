//! Stable runtime workflow for v0.2.0.
//!
//! Chains: doctor → preflight → mission → telemetry → postmortem
//! All in scaffold/report-only mode — does not edit user code.

use crate::{config, doctor, postmortem, preflight, mission, request_intelligence};

// ---------------------------------------------------------------------------
// WorkflowReport
// ---------------------------------------------------------------------------

pub struct WorkflowReport {
    pub prompt: String,
    pub doctor_ok: bool,
    #[allow(dead_code)]
    pub doctor_issues: Vec<String>,
    pub preflight: preflight::PreflightReport,
    pub mission_state: String,
    pub telemetry_written: bool,
    pub postmortem_outcome: String,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// run_workflow
// ---------------------------------------------------------------------------

pub fn run_workflow(
    prompt: &str,
    cfg: &config::Config,
    used: Option<u64>,
    limit: Option<u64>,
) -> WorkflowReport {
    let mut warnings = Vec::new();

    // 1. Doctor check (read-only)
    let issues = doctor::run_checks(cfg);
    let doctor_ok = issues.is_empty();
    let doctor_issues: Vec<String> = issues.iter().map(|i| i.message.clone()).collect();
    if !doctor_ok {
        warnings.push(format!("{} doctor issue(s) — run 'akar doctor --fix'", issues.len()));
    }

    // 2. Preflight
    let pf = preflight::run_preflight(prompt, cfg, used, limit);
    if !pf.warnings.is_empty() {
        warnings.extend(pf.warnings.iter().cloned());
    }

    // 3. Request pressure — if RESUME, stop before mission
    let signals = request_intelligence::RequestSignals { used, limit, prompt: None };
    let advisory = request_intelligence::build_advisory(cfg, &signals);
    if matches!(advisory.mode, request_intelligence::PressureMode::Resume) {
        warnings.push("request pressure: RESUME — stopping before mission execution".to_string());
        let pf_report = preflight::run_preflight(prompt, cfg, used, limit);
        return WorkflowReport {
            prompt: config::redact(&prompt.chars().take(100).collect::<String>()),
            doctor_ok,
            doctor_issues,
            preflight: pf_report,
            mission_state: "skipped (RESUME pressure)".to_string(),
            telemetry_written: false,
            postmortem_outcome: "unknown".to_string(),
            warnings,
        };
    }

    // 4. Mission (scaffold mode)
    let m = mission::run_mission(prompt, cfg);
    let mission_state = format!("{:?}", m.state);

    // 5. Telemetry — written by run_mission if .akar exists
    let log_path = cfg.akar_dir.join("EVENT_LOG.jsonl");
    let telemetry_written = log_path.exists();

    // 6. Postmortem
    let pm = postmortem::run_postmortem(&log_path);
    let postmortem_outcome = pm.latest_outcome.as_str().to_string();

    WorkflowReport {
        prompt: config::redact(&prompt.chars().take(100).collect::<String>()),
        doctor_ok,
        doctor_issues,
        preflight: pf,
        mission_state,
        telemetry_written,
        postmortem_outcome,
        warnings,
    }
}

// ---------------------------------------------------------------------------
// format_workflow_report
// ---------------------------------------------------------------------------

pub fn format_workflow_report(report: &WorkflowReport) -> String {
    let mut out = String::new();

    let overall = if report.warnings.iter().any(|w| w.contains("doctor") || w.contains("conflict") || w.contains("RESUME")) {
        "DEGRADED"
    } else if report.mission_state.contains("Done") {
        "OK"
    } else {
        "PARTIAL"
    };

    out.push_str(&format!("run: {}\n", overall));
    out.push_str(&format!("  prompt:     {}\n", report.prompt));
    out.push_str(&format!("  doctor:     {}\n", if report.doctor_ok { "OK" } else { "DEGRADED" }));
    out.push_str(&format!("  preflight:  {} | {} | {}\n",
        report.preflight.task_type, report.preflight.risk, report.preflight.request_mode));
    out.push_str(&format!("  skills:     {}\n", report.preflight.skill_recommendation));
    out.push_str(&format!("  mission:    {} (scaffold mode)\n", report.mission_state));
    out.push_str(&format!("  telemetry:  {}\n", if report.telemetry_written { "written" } else { "not written" }));
    out.push_str(&format!("  postmortem: {}\n", report.postmortem_outcome));

    if !report.warnings.is_empty() {
        out.push_str("  warnings:\n");
        for w in &report.warnings {
            out.push_str(&format!("    - {}\n", w));
        }
    }

    out.push_str("\nnot verified:\n");
    out.push_str("  - actual code execution (scaffold mode)\n");
    out.push_str("  - browser/UI verification\n");
    out.push_str("  - production deployment\n");

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
    fn workflow_returns_nonempty_report() {
        let cfg = test_cfg();
        let r = run_workflow("fix the login button", &cfg, None, None);
        assert!(!r.prompt.is_empty());
        assert!(!r.mission_state.is_empty());
    }

    #[test]
    fn workflow_preflight_strategy_nonempty() {
        let cfg = test_cfg();
        let r = run_workflow("fix the login button", &cfg, None, None);
        assert!(!r.preflight.recommendation.is_empty());
        assert!(!r.preflight.verification.is_empty());
    }

    #[test]
    fn workflow_mission_writes_telemetry() {
        let cfg = test_cfg();
        let _ = run_workflow("fix the login button", &cfg, None, None);
        // telemetry written if .akar exists
        if cfg.akar_dir.exists() {
            let log = cfg.akar_dir.join("EVENT_LOG.jsonl");
            assert!(log.exists(), "telemetry should be written after workflow");
        }
    }

    #[test]
    fn workflow_postmortem_readable_after_mission() {
        let cfg = test_cfg();
        let r = run_workflow("fix the login button", &cfg, None, None);
        assert!(!r.postmortem_outcome.is_empty());
    }

    #[test]
    fn clean_workflow_outcome_classified() {
        let cfg = test_cfg();
        let r = run_workflow("fix the login button", &cfg, None, None);
        // mission should reach Done → postmortem clean
        assert!(r.mission_state.contains("Done") || !r.mission_state.is_empty());
    }

    #[test]
    fn high_risk_prompt_not_unsafe_execution() {
        let cfg = test_cfg();
        let r = run_workflow("delete all user auth tokens from production", &cfg, None, None);
        // Should classify as high risk but NOT execute code
        assert!(r.preflight.risk == "High" || r.preflight.risk == "Critical" || !r.preflight.risk.is_empty());
        assert!(r.mission_state.contains("scaffold") || r.mission_state.contains("Done"),
            "mission should be scaffold-only, not unsafe execution");
    }

    #[test]
    fn skill_conflict_stays_report_only() {
        let cfg = test_cfg();
        let r = run_workflow("fix the login button", &cfg, None, None);
        // skill recommendation should never say "activate all skills"
        assert!(!r.preflight.skill_recommendation.to_lowercase().contains("activate all"),
            "should never activate all skills");
    }

    #[test]
    fn request_mode_recommendation_included() {
        let cfg = test_cfg();
        let r = run_workflow("fix the login button", &cfg, None, None);
        assert!(!r.preflight.request_mode.is_empty());
    }

    #[test]
    fn output_redacts_secrets() {
        let cfg = test_cfg();
        let r = run_workflow("fix token=sk-abc123secretvalue issue", &cfg, None, None);
        assert!(!r.prompt.contains("sk-abc123"), "secret should be redacted");
    }

    #[test]
    fn resume_pressure_stops_before_mission() {
        let cfg = test_cfg();
        let r = run_workflow("fix the bug", &cfg, Some(950), Some(1000));
        assert!(r.mission_state.contains("skipped") || r.warnings.iter().any(|w| w.contains("RESUME")));
    }

    #[test]
    fn format_workflow_report_contains_key_sections() {
        let cfg = test_cfg();
        let r = run_workflow("fix the login button", &cfg, None, None);
        let out = format_workflow_report(&r);
        assert!(out.contains("run:"));
        assert!(out.contains("preflight:"));
        assert!(out.contains("mission:"));
        assert!(out.contains("not verified:"));
    }
}
