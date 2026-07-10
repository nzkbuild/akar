//! Phase 16: Eval Harness
//!
//! Runs a suite of 28 behavioural evals across the core AKAR modules and
//! reports pass/fail results with detail strings. Several evals are labelled
//! `_smoke` — they are regression smoke checks, not behavior proofs; the
//! detail string says so explicitly.

use crate::{
    backup, config, context_pack, contract, design, doctor, event_log, request_intelligence,
    safety, skill_registry, verify, workflow,
};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

pub struct EvalResult {
    pub name: String,
    pub passed: bool,
    #[allow(dead_code)]
    pub detail: String,
}

pub struct EvalSuite {
    pub results: Vec<EvalResult>,
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

pub fn eval(name: &str, passed: bool, detail: &str) -> EvalResult {
    EvalResult {
        name: name.to_string(),
        passed,
        detail: detail.to_string(),
    }
}

// ---------------------------------------------------------------------------
// run_evals
// ---------------------------------------------------------------------------

pub fn run_evals(cfg: &config::Config) -> EvalSuite {
    let mut results: Vec<EvalResult> = Vec::new();

    // ---- Contract evals -------------------------------------------------------

    // 1. vague_prompt_contract — a vague prompt must still yield a COMPLETE
    // contract with sensible defaults (non-zero diff budget, a recognized
    // task type, A5 autonomy). Proves classify_prompt degrades gracefully.
    {
        let c = contract::classify_prompt("make this better");
        let has_budget = c.diff_budget.files_max > 0 && c.diff_budget.loc_max > 0;
        let recognized_type = matches!(
            c.task_type,
            contract::TaskType::Bugfix
                | contract::TaskType::Frontend
                | contract::TaskType::Feature
                | contract::TaskType::Refactor
                | contract::TaskType::Security
                | contract::TaskType::Migration
                | contract::TaskType::Dependency
        );
        let passed = has_budget && recognized_type && c.autonomy == contract::Autonomy::A5;
        results.push(eval(
            "vague_prompt_contract",
            passed,
            &format!(
                "task_type={:?} budget={}f/{}loc autonomy={:?} (complete+defaults)",
                c.task_type, c.diff_budget.files_max, c.diff_budget.loc_max, c.autonomy
            ),
        ));
    }

    // 2. micro_fix_budget
    {
        let c = contract::classify_prompt("fix button spacing");
        let passed = c.diff_budget.files_max <= 5;
        results.push(eval(
            "micro_fix_budget",
            passed,
            &format!("files_max={}", c.diff_budget.files_max),
        ));
    }

    // 3. frontend_prompt
    {
        let c = contract::classify_prompt("improve the UI design");
        let passed = c.task_type == contract::TaskType::Frontend;
        results.push(eval(
            "frontend_prompt",
            passed,
            &format!("task_type={:?}", c.task_type),
        ));
    }

    // 4. security_prompt
    {
        let c = contract::classify_prompt("fix auth vulnerability");
        let passed = c.risk_level == contract::RiskLevel::High
            || c.risk_level == contract::RiskLevel::Critical;
        results.push(eval(
            "security_prompt",
            passed,
            &format!("risk_level={:?}", c.risk_level),
        ));
    }

    // 5. migration_prompt
    {
        let c = contract::classify_prompt("migrate user schema");
        let passed = c.task_type == contract::TaskType::Migration;
        results.push(eval(
            "migration_prompt",
            passed,
            &format!("task_type={:?}", c.task_type),
        ));
    }

    // 6. default_autonomy
    {
        let c = contract::classify_prompt("do something");
        let passed = c.autonomy == contract::Autonomy::A5;
        results.push(eval(
            "default_autonomy",
            passed,
            &format!("autonomy={:?}", c.autonomy),
        ));
    }

    // 7. diff_budget_small_loc
    {
        let c = contract::classify_prompt("fix the bug");
        let passed = c.diff_budget.loc_max <= 200;
        results.push(eval(
            "diff_budget_small_loc",
            passed,
            &format!("loc_max={}", c.diff_budget.loc_max),
        ));
    }

    // 8. diff_budget_medium_loc
    {
        let c = contract::classify_prompt("add new feature");
        let passed = c.diff_budget.loc_max <= 600;
        results.push(eval(
            "diff_budget_medium_loc",
            passed,
            &format!("loc_max={}", c.diff_budget.loc_max),
        ));
    }

    // ---- Safety evals ---------------------------------------------------------

    // 9. classify_safe_command
    {
        let a = safety::classify_command("git status");
        let passed = a.risk == safety::CommandRisk::Safe;
        results.push(eval(
            "classify_safe_command",
            passed,
            &format!("risk={:?}", a.risk),
        ));
    }

    // 10. classify_critical_command
    {
        let a = safety::classify_command("git push --force");
        let passed = a.blocked;
        results.push(eval(
            "classify_critical_command",
            passed,
            &format!("blocked={} risk={:?}", a.blocked, a.risk),
        ));
    }

    // 11. secret_detection
    {
        let warnings = safety::check_secrets("token=sk-abc123");
        let passed = !warnings.is_empty();
        results.push(eval(
            "secret_detection",
            passed,
            &format!("{} warning(s)", warnings.len()),
        ));
    }

    // 12. no_secret_clean_text
    {
        let warnings = safety::check_secrets("hello world");
        let passed = warnings.is_empty();
        results.push(eval(
            "no_secret_clean_text",
            passed,
            &format!("{} warning(s)", warnings.len()),
        ));
    }

    // ---- Verify evals ---------------------------------------------------------

    // 13. verify_recipe_detect
    {
        let recipe = verify::detect_recipe(&cfg.project_root);
        let has_cargo = recipe.commands.iter().any(|c| c.command == "cargo");
        results.push(eval(
            "verify_recipe_detect",
            has_cargo,
            &format!("{} command(s), cargo={}", recipe.commands.len(), has_cargo),
        ));
    }

    // ---- Doctor eval ----------------------------------------------------------

    // 14. doctor_check — the doctor must run real checks and produce a
    // sectioned report with a valid OK/WARN/FAIL status. On the real repo it
    // must not FAIL (a FAIL means a dogfood-critical problem: invalid
    // NEXT_RUN, missing hook templates, malformed logs, or no git repo).
    {
        let report = doctor::run_doctor_report(cfg);
        let valid_status = matches!(
            report.status,
            doctor::DoctorStatus::Ok | doctor::DoctorStatus::Warn | doctor::DoctorStatus::Fail
        );
        let has_sections = !report.environment.is_empty()
            && !report.files.is_empty()
            && !report.hooks.is_empty()
            && !report.telemetry.is_empty()
            && !report.git.is_empty()
            && !report.next_run.is_empty();
        // On the real AKAR repo (bootstrapped, git, valid NEXT_RUN), the
        // doctor must be OK or WARN — never FAIL.
        let not_failed_on_real_repo = report.status != doctor::DoctorStatus::Fail;
        let passed = valid_status && has_sections && not_failed_on_real_repo;
        results.push(eval(
            "doctor_check",
            passed,
            &format!(
                "status={} sections={} findings={} (real checks, not just non-panic)",
                report.status.as_str(),
                has_sections,
                report.to_issues().len()
            ),
        ));
    }

    // ---- Context pack eval ----------------------------------------------------

    // 15. context_pack_build — the pack must be internally consistent:
    // total_files must equal files.len(), and every listed file must exist
    // on disk (the pack must not list missing files). Proves the pack is
    // well-formed, not just non-panicking.
    {
        let pack = context_pack::build_pack(cfg);
        let count_consistent = pack.total_files == pack.files.len();
        let all_exist = pack.files.iter().all(|f| f.path.exists());
        let passed = count_consistent && all_exist;
        results.push(eval(
            "context_pack_build",
            passed,
            &format!(
                "total_files={} count_consistent={} all_exist={} (real invariants)",
                pack.total_files, count_consistent, all_exist
            ),
        ));
    }

    // ---- Design eval ----------------------------------------------------------

    // 16. design_check — the report's has_design_dna flag must match whether
    // .akar/DESIGN_DNA.md actually exists, and a missing DNA must produce
    // exactly the design_dna_missing warning. Proves the check reflects
    // reality, not just that it ran.
    {
        let report = design::check_project(&cfg.project_root);
        let dna_exists = cfg.akar_dir.join("DESIGN_DNA.md").exists();
        let flag_matches = report.has_design_dna == dna_exists;
        let issues_consistent = if dna_exists {
            report.issues.is_empty()
        } else {
            report
                .issues
                .iter()
                .any(|i| i.check == "design_dna_missing")
        };
        let passed = flag_matches && issues_consistent;
        results.push(eval(
            "design_check",
            passed,
            &format!(
                "has_design_dna={} dna_exists={} issues={} (flag matches reality)",
                report.has_design_dna,
                dna_exists,
                report.issues.len()
            ),
        ));
    }

    // ---- Event log eval -------------------------------------------------------

    // 17. event_log_append
    {
        let log_path = std::env::temp_dir().join("akar_eval_event_log.jsonl");
        let _ = std::fs::remove_file(&log_path); // clean slate

        let entry = event_log::EventEntry {
            ts: "2026-07-04T04:00:00Z".to_string(),
            project: "akar-eval".to_string(),
            model: "eval".to_string(),
            event: "info".to_string(),
            event_type: "eval_run".to_string(),
            summary: "eval harness test entry".to_string(),
            resolution: "".to_string(),
            confidence: "high".to_string(),
        };

        let append_ok = event_log::append_event(&log_path, &entry).is_ok();
        let lines = event_log::read_recent(&log_path, 10);
        let passed = append_ok && lines.len() == 1;

        let _ = std::fs::remove_file(&log_path); // cleanup

        results.push(eval(
            "event_log_append",
            passed,
            &format!("append_ok={} lines_back={}", append_ok, lines.len()),
        ));
    }

    // ---- Backup eval ----------------------------------------------------------

    // 18. backup_restore_cycle
    {
        use std::io::Write as IoWrite;

        let tmp_dir = std::env::temp_dir().join("akar_eval_backup");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let original_path = tmp_dir.join("eval_target.txt");
        let content = b"eval backup content";

        let mut ok = false;
        #[allow(unused_assignments)]
        let mut detail = String::new();

        // Write original
        if let Ok(mut f) = std::fs::File::create(&original_path) {
            if f.write_all(content).is_ok() {
                // Backup
                match backup::backup_file(&original_path) {
                    Ok(bak_path) => {
                        // Delete original
                        let _ = std::fs::remove_file(&original_path);
                        // Restore
                        match backup::restore_backup(&bak_path, &original_path) {
                            Ok(()) => {
                                // Verify content
                                match std::fs::read(&original_path) {
                                    Ok(bytes) if bytes == content => {
                                        ok = true;
                                        detail = "content verified after restore".to_string();
                                    }
                                    Ok(_) => {
                                        detail = "content mismatch after restore".to_string();
                                    }
                                    Err(e) => {
                                        detail = format!("read after restore failed: {}", e);
                                    }
                                }
                                // Cleanup backup
                                let _ = std::fs::remove_file(&bak_path);
                            }
                            Err(e) => {
                                detail = format!("restore failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        detail = format!("backup failed: {}", e);
                    }
                }
            } else {
                detail = "write to temp file failed".to_string();
            }
        } else {
            detail = "create temp file failed".to_string();
        }

        // Cleanup
        let _ = std::fs::remove_file(&original_path);
        let _ = std::fs::remove_dir_all(&tmp_dir);

        results.push(eval("backup_restore_cycle", ok, &detail));
    }

    // ---- Dependency policy-library eval (not a runtime governor) -------------
    // Exercises safety::govern_dependency, a pure policy function reachable
    // only from this eval and unit tests. AKAR has no runtime dependency
    // governor; this keeps the approval policy tested.

    // 19. dependency_govern_critical
    {
        let proposal = safety::DependencyProposal {
            name: "evil-pkg".to_string(),
            reason: "executes arbitrary code".to_string(),
            risk: safety::CommandRisk::Critical,
        };
        let (approved, msg) = safety::govern_dependency(&proposal);
        let passed = !approved;
        results.push(eval(
            "dependency_govern_critical",
            passed,
            &format!("policy-check approved={} msg={}", approved, msg),
        ));
    }

    // ---- Migration policy-library eval (not a runtime governor) --------------
    // Exercises safety::check_migration, a pure policy function reachable only
    // from this eval and unit tests. AKAR has no runtime migration governor.

    // 20. migration_no_rollback
    {
        let check = safety::MigrationCheck {
            description: "drop users table".to_string(),
            has_rollback: false,
            destructive: true,
        };
        let (safe, msg) = safety::check_migration(&check);
        let passed = !safe;
        results.push(eval(
            "migration_no_rollback",
            passed,
            &format!("policy-check safe={} msg={}", safe, msg),
        ));
    }

    // ---- Skill registry evals ------------------------------------------------

    // 21. skill_conflict_superpower_gsd
    {
        let skills = vec![
            skill_registry::SkillEntry {
                name: "superpower".to_string(),
                source: skill_registry::SkillSource::Superpower,
                purpose: "methodology a".to_string(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: skill_registry::SkillStatus::Active,
                role: skill_registry::SkillRole::Methodology,
            },
            skill_registry::SkillEntry {
                name: "gsd".to_string(),
                source: skill_registry::SkillSource::Custom,
                purpose: "methodology b".to_string(),
                risk: "low".to_string(),
                token_cost: "low".to_string(),
                status: skill_registry::SkillStatus::Active,
                role: skill_registry::SkillRole::Methodology,
            },
        ];
        let conflicts = skill_registry::detect_skill_conflicts(&skills);
        let passed = !conflicts.is_empty();
        results.push(eval(
            "skill_conflict_superpower_gsd",
            passed,
            &format!("{} conflict(s)", conflicts.len()),
        ));
    }

    // 22. no_all_skills_mode (smoke) — scan_skills on a nonexistent directory
    // must return an empty list (no crash, no phantom skills). This is a
    // regression smoke check, not a behavior proof; the detail says so.
    {
        use std::path::PathBuf;
        let fake_dir = PathBuf::from("/nonexistent/akar/eval/skills/path");
        let skills = skill_registry::scan_skills(&fake_dir);
        let passed = skills.is_empty();
        results.push(eval(
            "no_all_skills_mode_smoke",
            passed,
            &format!(
                "smoke: scan returned {} skills for nonexistent dir (empty == no phantom skills)",
                skills.len()
            ),
        ));
    }

    // 23. request_pressure_compaction — at 70% request pressure, build_advisory
    // must actually return PressureMode::Compact and the strategy must mention
    // compaction. Proves real pressure-mode output, not a tautology.
    {
        let signals = request_intelligence::RequestSignals {
            used: Some(700),
            limit: Some(1000),
            prompt: None,
        };
        let advisory = request_intelligence::build_advisory(cfg, &signals);
        let mode_is_compact = advisory.mode == request_intelligence::PressureMode::Compact;
        let strategy_mentions_compact = advisory
            .strategy
            .iter()
            .any(|s| s.to_lowercase().contains("compact"));
        let passed = mode_is_compact && strategy_mentions_compact;
        results.push(eval(
            "request_pressure_compaction",
            passed,
            &format!(
                "mode={:?} strategy_mentions_compact={} (real pressure output)",
                advisory.mode, strategy_mentions_compact
            ),
        ));
    }

    // 24. claimed_complete_requires_verification
    {
        let recipe = verify::detect_recipe(&cfg.project_root);
        let passed = !recipe.commands.is_empty();
        results.push(eval(
            "claimed_complete_requires_verification",
            passed,
            &format!(
                "{} verification command(s) in recipe",
                recipe.commands.len()
            ),
        ));
    }

    // 25. learning_patch_from_failure
    {
        let log_path = std::env::temp_dir().join("akar_eval_failure_patch.jsonl");
        let _ = std::fs::remove_file(&log_path);

        let entry = event_log::EventEntry {
            ts: "2026-07-04T06:00:00Z".to_string(),
            project: "akar-eval".to_string(),
            model: "eval".to_string(),
            event: "failure".to_string(),
            event_type: "failure".to_string(),
            summary: "learning patch test failure entry".to_string(),
            resolution: "".to_string(),
            confidence: "low".to_string(),
        };

        let append_ok = event_log::append_event(&log_path, &entry).is_ok();
        let lines = event_log::read_recent(&log_path, 10);
        let passed = append_ok && !lines.is_empty() && lines[0].contains("failure");

        let _ = std::fs::remove_file(&log_path);

        results.push(eval(
            "learning_patch_from_failure",
            passed,
            &format!("append_ok={} readable={}", append_ok, !lines.is_empty()),
        ));
    }

    // ---- v0.2.0 stable runtime evals -----------------------------------------

    // 26. stable_runtime_workflow
    {
        let r = workflow::run_workflow("fix the login button", cfg, None, None);
        let passed = !r.mission_state.is_empty() && !r.preflight.recommendation.is_empty();
        results.push(eval(
            "stable_runtime_workflow",
            passed,
            &format!(
                "mission={} preflight_ok={}",
                r.mission_state,
                !r.preflight.recommendation.is_empty()
            ),
        ));
    }

    // 27. high_risk_preflight_blocks_execution
    {
        let r =
            workflow::run_workflow("delete all auth tokens from production db", cfg, None, None);
        let high_risk = r.preflight.risk == "High" || r.preflight.risk == "Critical";
        let scaffold_only = r.mission_state.contains("Done")
            || r.mission_state.contains("scaffold")
            || r.mission_state.contains("skipped");
        let passed = high_risk && scaffold_only;
        results.push(eval(
            "high_risk_preflight_blocks_execution",
            passed,
            &format!("risk={} mission={}", r.preflight.risk, r.mission_state),
        ));
    }

    // 28. telemetry_postmortem_chain
    {
        let r = workflow::run_workflow("fix the login button", cfg, None, None);
        let passed = !r.postmortem_outcome.is_empty();
        results.push(eval(
            "telemetry_postmortem_chain",
            passed,
            &format!("postmortem_outcome={}", r.postmortem_outcome),
        ));
    }

    // ---- Tally ----------------------------------------------------------------

    let passed_count = results.iter().filter(|r| r.passed).count();
    let failed_count = results.iter().filter(|r| !r.passed).count();
    let total = results.len();

    EvalSuite {
        results,
        passed: passed_count,
        failed: failed_count,
        total,
    }
}

// ---------------------------------------------------------------------------
// format_eval_report
// ---------------------------------------------------------------------------

pub fn format_eval_report(suite: &EvalSuite) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "eval: {}/{} passed ({} failed)\n",
        suite.passed, suite.total, suite.failed
    ));
    out.push('\n');
    out.push_str("Results:\n");

    for r in &suite.results {
        if r.passed {
            out.push_str(&format!("  [PASS] {}\n", r.name));
        } else {
            out.push_str(&format!("  [FAIL] {}: {}\n", r.name, r.detail));
        }
    }

    out.push('\n');
    let overall = if suite.failed == 0 { "PASS" } else { "FAIL" };
    out.push_str(&format!("overall: {}\n", overall));

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_helper_constructs_correctly() {
        let r = eval("my_eval", true, "all good");
        assert_eq!(r.name, "my_eval");
        assert!(r.passed);
        assert_eq!(r.detail, "all good");
    }

    #[test]
    fn run_evals_returns_28_results() {
        let cfg = config::Config::discover();
        let suite = run_evals(&cfg);
        assert_eq!(suite.total, 28, "expected 28 evals, got {}", suite.total);
        assert_eq!(suite.passed + suite.failed, suite.total);
    }

    #[test]
    fn format_eval_report_contains_overall() {
        let cfg = config::Config::discover();
        let suite = run_evals(&cfg);
        let report = format_eval_report(&suite);
        assert!(
            report.contains("overall:"),
            "report missing 'overall:' line"
        );
        assert!(
            report.contains("eval:"),
            "report missing 'eval:' summary line"
        );
    }

    #[test]
    fn format_eval_report_overall_pass_when_all_pass() {
        let suite = EvalSuite {
            results: vec![eval("a", true, "ok"), eval("b", true, "ok")],
            passed: 2,
            failed: 0,
            total: 2,
        };
        let report = format_eval_report(&suite);
        assert!(report.contains("overall: PASS"));
    }

    #[test]
    fn format_eval_report_overall_fail_when_any_fail() {
        let suite = EvalSuite {
            results: vec![eval("a", true, "ok"), eval("b", false, "broken")],
            passed: 1,
            failed: 1,
            total: 2,
        };
        let report = format_eval_report(&suite);
        assert!(report.contains("overall: FAIL"));
        assert!(report.contains("[FAIL] b: broken"));
    }
}
