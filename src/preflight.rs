//! Mission Preflight v0 — compact strategy advisor before a mission proceeds.
//!
//! Preflight does not execute the task. It combines contract classification,
//! request intelligence, skill intelligence, and verification recipe into a
//! concise strategy recommendation.

use crate::{config, contract, request_intelligence, skill_registry, verify};

// ---------------------------------------------------------------------------
// PreflightReport
// ---------------------------------------------------------------------------

pub struct PreflightReport {
    pub prompt: String,
    pub task_type: String,
    pub risk: String,
    pub autonomy: String,
    pub diff_budget: String,
    pub request_mode: String,
    pub skill_recommendation: String,
    pub verification: Vec<String>,
    pub stop_conditions: Vec<String>,
    pub recommendation: String,
    pub warnings: Vec<String>,
}

// ---------------------------------------------------------------------------
// run_preflight
// ---------------------------------------------------------------------------

pub fn run_preflight(
    prompt: &str,
    cfg: &config::Config,
    used: Option<u64>,
    limit: Option<u64>,
) -> PreflightReport {
    // 1. Contract classification
    let tc = contract::classify_prompt(prompt);
    let task_type = format!("{:?}", tc.task_type);
    let risk = format!("{:?}", tc.risk_level);
    let autonomy = format!("{:?}", tc.autonomy);
    let diff_budget = format!(
        "{}-{} files, {}-{} LOC",
        tc.diff_budget.files_min,
        tc.diff_budget.files_max,
        tc.diff_budget.loc_min,
        tc.diff_budget.loc_max
    );

    // 2. Request intelligence
    let signals = request_intelligence::RequestSignals {
        used,
        limit,
        prompt: None,
    };
    let advisory = request_intelligence::build_advisory(cfg, &signals);
    let request_mode = advisory.mode.as_str().to_string();

    // 3. Skill intelligence — scan project-local skills only to avoid noise
    // from all 200+ global skills. Full scan remains available via akar skills.
    let project_commands = cfg.project_root.join(".claude").join("commands");
    let skills = skill_registry::scan_skills(&project_commands);
    let skill_report = skill_registry::build_skill_report(&skills);
    let skill_recommendation = skill_recommendation_for_task(&tc, &skill_report);

    // 4. Verification recipe
    let recipe = verify::detect_recipe(&cfg.project_root);
    let mut verification: Vec<String> = recipe
        .commands
        .iter()
        .map(|c| format!("run: {}", c.name))
        .collect();
    verification.extend(task_specific_checks(&tc));

    // 5. Stop conditions
    let stop_conditions = if tc.stop_conditions.is_empty() {
        vec!["none defined — proceed with diff budget discipline".to_string()]
    } else {
        tc.stop_conditions.clone()
    };

    // 6. Warnings
    let mut warnings = Vec::new();
    if !skill_report.conflicts.is_empty() {
        warnings.push(format!(
            "{} skill conflict(s) detected — check 'akar skills'",
            skill_report.conflicts.len()
        ));
    }
    if matches!(
        advisory.mode,
        request_intelligence::PressureMode::Boundary | request_intelligence::PressureMode::Resume
    ) {
        warnings.push(format!(
            "request pressure: {} — complete one atomic step only",
            request_mode
        ));
    }

    // 7. Recommendation
    let recommendation = build_recommendation(&tc, &advisory.mode, &skill_report);

    PreflightReport {
        prompt: config::redact(&prompt.chars().take(100).collect::<String>()),
        task_type,
        risk,
        autonomy,
        diff_budget,
        request_mode,
        skill_recommendation,
        verification,
        stop_conditions,
        recommendation,
        warnings,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn skill_recommendation_for_task(
    tc: &contract::TaskContract,
    report: &skill_registry::SkillReport,
) -> String {
    // If conflicts exist, always recommend kernel-only.
    if report.conflicts.iter().any(|c| c.starts_with("conflict:")) {
        return "AKAR kernel only (skill conflicts detected — library-only for all)".to_string();
    }

    // Superpower recommended only for planning/spec/TDD tasks.
    let needs_methodology = matches!(
        tc.task_type,
        contract::TaskType::Feature | contract::TaskType::Greenfield | contract::TaskType::Refactor
    ) && matches!(
        tc.risk_level,
        contract::RiskLevel::Low | contract::RiskLevel::Medium
    );

    // GSD recommended only for execution sprints — never with Superpower.
    let needs_execution = false; // not recommended automatically in v0.1.9

    if needs_methodology && report.methodology_count > 0 {
        return "Superpower: library-only reference (planning/spec tasks only)".to_string();
    }
    let _ = needs_execution;

    "zero-skill mode (AKAR kernel only)".to_string()
}

fn task_specific_checks(tc: &contract::TaskContract) -> Vec<String> {
    let mut checks = Vec::new();
    match tc.task_type {
        contract::TaskType::Frontend => {
            checks.push("manual: visual browser check required".to_string());
            checks.push("manual: responsive layout check".to_string());
        }
        contract::TaskType::Security => {
            checks.push("manual: secret/credential scan before commit".to_string());
            checks.push("manual: auth/permission boundary review".to_string());
        }
        contract::TaskType::Migration => {
            checks.push("manual: rollback plan documented".to_string());
            checks.push("manual: backup exists before migration".to_string());
        }
        contract::TaskType::Inspect | contract::TaskType::Answer => {
            checks.push("manual: clarify scope before proceeding".to_string());
        }
        _ => {}
    }
    checks
}

fn build_recommendation(
    tc: &contract::TaskContract,
    mode: &request_intelligence::PressureMode,
    skill_report: &skill_registry::SkillReport,
) -> String {
    let pressure_note = match mode {
        request_intelligence::PressureMode::Normal => "",
        request_intelligence::PressureMode::Saver => " — batch reads, avoid repeated checks",
        request_intelligence::PressureMode::Compact => " — compact context only",
        request_intelligence::PressureMode::Boundary => " — finish ONE atomic step only",
        request_intelligence::PressureMode::Resume => " — write NEXT_RUN.md, stop cleanly",
    };

    let conflict_note = if skill_report
        .conflicts
        .iter()
        .any(|c| c.starts_with("conflict:"))
    {
        " — resolve skill conflicts first"
    } else {
        ""
    };

    match tc.risk_level {
        contract::RiskLevel::Critical => {
            format!(
                "STOP — critical risk task. Plan, research, and checkpoint before execution{}{}",
                pressure_note, conflict_note
            )
        }
        contract::RiskLevel::High => {
            format!(
                "Proceed with caution — high risk. Verify before and after each step{}{}",
                pressure_note, conflict_note
            )
        }
        contract::RiskLevel::Medium => {
            format!(
                "Proceed with discipline — follow diff budget and verify{}{}",
                pressure_note, conflict_note
            )
        }
        contract::RiskLevel::Low => {
            format!(
                "Proceed — low risk task. Stay within diff budget{}{}",
                pressure_note, conflict_note
            )
        }
    }
}

// ---------------------------------------------------------------------------
// format_preflight_report
// ---------------------------------------------------------------------------

pub fn format_preflight_report(report: &PreflightReport) -> String {
    let mut out = String::new();
    out.push_str("preflight:\n");
    out.push_str(&format!("  prompt:       {}\n", report.prompt));
    out.push_str(&format!("  task:         {}\n", report.task_type));
    out.push_str(&format!("  risk:         {}\n", report.risk));
    out.push_str(&format!("  autonomy:     {}\n", report.autonomy));
    out.push_str(&format!("  diff_budget:  {}\n", report.diff_budget));
    out.push_str(&format!("  request_mode: {}\n", report.request_mode));
    out.push_str(&format!(
        "  skills:       {}\n",
        report.skill_recommendation
    ));
    out.push_str("  verification:\n");
    for v in &report.verification {
        out.push_str(&format!("    - {}\n", v));
    }
    out.push_str("  stop_conditions:\n");
    for s in &report.stop_conditions {
        out.push_str(&format!("    - {}\n", s));
    }
    if !report.warnings.is_empty() {
        out.push_str("  warnings:\n");
        for w in &report.warnings {
            out.push_str(&format!("    - {}\n", w));
        }
    }
    out.push_str(&format!("  recommendation: {}\n", report.recommendation));
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
    fn bugfix_preflight_produces_small_diff_budget() {
        let cfg = test_cfg();
        let r = run_preflight("fix the login button", &cfg, None, None);
        assert_eq!(r.task_type, "Bugfix");
        // diff budget should be small
        assert!(
            r.diff_budget.contains("60") || r.diff_budget.contains("200"),
            "expected small diff budget, got: {}",
            r.diff_budget
        );
    }

    #[test]
    fn security_preflight_produces_high_risk() {
        let cfg = test_cfg();
        let r = run_preflight("rotate leaked auth token secrets", &cfg, None, None);
        assert!(
            r.risk == "High" || r.risk == "Critical",
            "expected High/Critical risk for security task, got: {}",
            r.risk
        );
        assert!(
            r.verification
                .iter()
                .any(|v| v.contains("secret") || v.contains("auth")),
            "expected security verification checks"
        );
    }

    #[test]
    fn vague_prompt_still_produces_report() {
        let cfg = test_cfg();
        let r = run_preflight("do something", &cfg, None, None);
        assert!(!r.recommendation.is_empty());
        assert!(!r.verification.is_empty());
    }

    #[test]
    fn superpower_not_activated_for_simple_bugfix() {
        let cfg = test_cfg();
        let r = run_preflight("fix button spacing", &cfg, None, None);
        assert!(
            !r.skill_recommendation.contains("Superpower")
                || r.skill_recommendation.contains("library-only")
                || r.skill_recommendation.contains("kernel only"),
            "Superpower should not be activated as controller for simple bugfix"
        );
    }

    #[test]
    fn superpower_may_be_recommended_for_planning_task() {
        let cfg = test_cfg();
        let r = run_preflight("plan a new auth system architecture", &cfg, None, None);
        // Should either recommend Superpower library-only or kernel-only (conflicts may exist)
        assert!(!r.recommendation.is_empty());
    }

    #[test]
    fn gsd_not_combined_with_superpower_as_controller() {
        let cfg = test_cfg();
        let r = run_preflight("implement new feature", &cfg, None, None);
        // skill_recommendation should never say both are active controllers
        assert!(
            !r.skill_recommendation.contains("GSD")
                || r.skill_recommendation.contains("library-only")
                || r.skill_recommendation.contains("kernel only")
        );
    }

    #[test]
    fn request_pressure_affects_strategy() {
        let cfg = test_cfg();
        let r = run_preflight("fix the bug", &cfg, Some(950), Some(1000));
        assert_eq!(r.request_mode, "RESUME");
        assert!(
            r.recommendation.contains("NEXT_RUN")
                || r.warnings.iter().any(|w| w.contains("request pressure"))
        );
    }

    #[test]
    fn verification_recommendation_is_nonempty() {
        let cfg = test_cfg();
        let r = run_preflight("fix the login bug", &cfg, None, None);
        assert!(
            !r.verification.is_empty(),
            "verification should always have at least one entry"
        );
    }

    #[test]
    fn output_redacts_secret_looking_values() {
        let cfg = test_cfg();
        let r = run_preflight("fix token=sk-abc123secretvalue issue", &cfg, None, None);
        assert!(
            !r.prompt.contains("sk-abc123"),
            "secret should be redacted in prompt preview"
        );
    }
}
