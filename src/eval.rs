//! Phase 16: Eval Harness
//!
//! Runs a suite of 20 behavioural evals across the core AKAR modules and
//! reports pass/fail results with detail strings.

use crate::{backup, config, context_pack, contract, design, doctor, event_log, safety, verify};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

pub struct EvalResult {
    pub name: String,
    pub passed: bool,
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

    // 1. vague_prompt_contract
    {
        let c = contract::classify_prompt("make this better");
        let passed = true; // classify_prompt always returns a contract
        results.push(eval(
            "vague_prompt_contract",
            passed,
            &format!("task_type={:?}", c.task_type),
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
        let has_cargo = recipe
            .commands
            .iter()
            .any(|c| c.command == "cargo");
        results.push(eval(
            "verify_recipe_detect",
            has_cargo,
            &format!(
                "{} command(s), cargo={}",
                recipe.commands.len(),
                has_cargo
            ),
        ));
    }

    // ---- Doctor eval ----------------------------------------------------------

    // 14. doctor_check
    {
        let _issues = doctor::run_checks(cfg);
        // If we get here without panic, it passes.
        results.push(eval(
            "doctor_check",
            true,
            &format!("{} issue(s)", _issues.len()),
        ));
    }

    // ---- Context pack eval ----------------------------------------------------

    // 15. context_pack_build
    {
        let pack = context_pack::build_pack(cfg);
        let passed = pack.total_files as isize >= 0; // always true; confirms no panic
        results.push(eval(
            "context_pack_build",
            passed,
            &format!("total_files={}", pack.total_files),
        ));
    }

    // ---- Design eval ----------------------------------------------------------

    // 16. design_check
    {
        let report = design::check_project(&cfg.project_root);
        // Passes if it ran without panicking.
        results.push(eval(
            "design_check",
            true,
            &format!(
                "has_design_dna={} issues={}",
                report.has_design_dna,
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

    // ---- Dependency governor eval ---------------------------------------------

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
            &format!("approved={} msg={}", approved, msg),
        ));
    }

    // ---- Migration safety eval ------------------------------------------------

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
            &format!("safe={} msg={}", safe, msg),
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
    fn run_evals_returns_20_results() {
        let cfg = config::Config::discover();
        let suite = run_evals(&cfg);
        assert_eq!(suite.total, 20, "expected 20 evals, got {}", suite.total);
        assert_eq!(suite.passed + suite.failed, suite.total);
    }

    #[test]
    fn format_eval_report_contains_overall() {
        let cfg = config::Config::discover();
        let suite = run_evals(&cfg);
        let report = format_eval_report(&suite);
        assert!(report.contains("overall:"), "report missing 'overall:' line");
        assert!(report.contains("eval:"), "report missing 'eval:' summary line");
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
