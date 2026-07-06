mod backup;
mod bootstrap;
mod diff_budget;
mod foundation;
mod hooks;
mod init;
mod config;
mod context_pack;
mod contract;
mod design;
mod doctor;
mod eval;
mod event_log;
mod learn;
mod loop_governor;
mod mission;
mod model_profile;
mod postmortem;
mod preflight;
mod request_intelligence;
mod safe_fix;
mod safety;
mod skill_registry;
mod verify;
mod workflow;

use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");
#[allow(dead_code)]
const RAM_BUDGET_MB: u64 = 150;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "--version" | "-V" => {
            println!("akar {}", VERSION);
        }
        "--help" | "-h" => {
            print_usage();
        }
        "status" => cmd_status(),
        "governor" => {
            let one_line = args.iter().any(|a| a == "--one-line");
            let json_mode = args.iter().any(|a| a == "--json");
            let no_exit_code = args.iter().any(|a| a == "--no-exit-code");
            // Telemetry is opt-in: --telemetry flag OR AKAR_GOVERNOR_TELEMETRY=1.
            // Default (neither set) writes nothing.
            let telemetry_flag = args.iter().any(|a| a == "--telemetry");
            let telemetry_env = std::env::var("AKAR_GOVERNOR_TELEMETRY")
                .map(|v| v == "1")
                .unwrap_or(false);
            let telemetry = telemetry_flag || telemetry_env;
            let code = cmd_governor(one_line, json_mode, no_exit_code, telemetry);
            // Exit codes are for orchestration only. AKAR still does not
            // execute the suggested action. --no-exit-code forces 0 so the
            // command is safe to call in pipelines that check $?
            process::exit(code);
        }
        "doctor" => {
            let fix_mode = args.get(2).map(|s| s.as_str()) == Some("--fix");
            cmd_doctor(fix_mode);
        }
        "init" => {
            let skip = args.iter().any(|a| a == "--skip");
            let claude = args.iter().any(|a| a == "--claude");
            cmd_init(skip, claude);
        }
        "bootstrap" => cmd_bootstrap(),
        "verify" => cmd_verify(),
        "eval" => {
            let prompt = args.get(2).map(|s| s.as_str());
            cmd_eval(prompt);
        }
        "hooks" => {
            let check = args.iter().any(|a| a == "--check");
            let install = args.iter().any(|a| a == "--install");
            cmd_hooks(check, install);
        }
        "safety" => {
            let command = args.get(2).map(|s| s.as_str());
            cmd_safety(command);
        }
        "mission" => {
            let prompt_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
            if prompt_args.is_empty() {
                println!("akar mission <prompt>");
                println!("  example: akar mission \"fix the login bug\"");
            } else {
                let prompt = prompt_args.join(" ");
                cmd_mission(&prompt);
            }
        }
        "skills" => cmd_skills(),
        "calibrate" => cmd_calibrate(),
        "postmortem" => {
            let diff_mode = args.iter().any(|a| a == "--diff");
            let baseline_mode = args.iter().any(|a| a == "--baseline");
            let task_name = parse_flag_str(&args, "--task");
            cmd_postmortem(diff_mode, baseline_mode, task_name);
        }
        "telemetry" => cmd_telemetry(),
        "learn" => {
            let list_mode = args.iter().any(|a| a == "--list");
            let resolve_mode = args.iter().any(|a| a == "--resolve");
            cmd_learn(list_mode, resolve_mode);
        }
        "run" => {
            let prompt_args: Vec<&str> = args[2..].iter()
                .take_while(|s| !s.starts_with("--"))
                .map(|s| s.as_str())
                .collect();
            let used = parse_flag_u64(&args, "--used");
            let limit = parse_flag_u64(&args, "--limit");
            if prompt_args.is_empty() {
                println!("akar run \"<task prompt>\"");
                println!("  example: akar run \"fix the login button\"");
                println!("  runs: doctor → preflight → mission → telemetry → postmortem");
            } else {
                cmd_run(&prompt_args.join(" "), used, limit);
            }
        }
        "preflight" => {
            let snapshot = args.iter().any(|a| a == "--snapshot");
            let prompt_args: Vec<&str> = args[2..].iter()
                .filter(|s| !s.starts_with("--"))
                .map(|s| s.as_str())
                .collect();
            let used = parse_flag_u64(&args, "--used");
            let limit = parse_flag_u64(&args, "--limit");
            if prompt_args.is_empty() {
                println!("akar preflight \"<task prompt>\"");
                println!("  akar preflight --snapshot \"<task>\"  — write diff baseline before session");
                println!("  example: akar preflight \"fix the login button\"");
            } else {
                cmd_preflight(&prompt_args.join(" "), used, limit, snapshot);
            }
        }
        "request" => {
            let check_mode = args.iter().any(|a| a == "--check");
            if check_mode {
                let code = cmd_request_check();
                process::exit(code);
            }
            let used = parse_flag_u64(&args, "--used");
            let limit = parse_flag_u64(&args, "--limit");
            let prompt = parse_flag_str(&args, "--prompt");
            cmd_request(used, limit, prompt);
        }
        other => {
            eprintln!("akar: unknown command '{}'", other);
            eprintln!("Run 'akar --help' for usage.");
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("akar {} — Adaptive Knowledge & Action Runtime", VERSION);
    println!();
    println!("USAGE:");
    println!("  akar <command>");
    println!();
    println!("COMMANDS:");
    println!("  init            First-run onboarding: bootstrap + doctor + next-steps guide");
    println!("  init --claude   Include Claude Code integration instructions");
    println!("  status      Show runtime health and current session state");
    println!("  governor   Print the loop governor decision (next safe action)");
    println!("  governor --one-line  Print DECISION<TAB>SUGGESTED_PROMPT on one line");
    println!("  governor --json      Print the loop governor decision as JSON");
    println!("  governor --no-exit-code  Print output but always exit 0");
    println!("  governor --telemetry     Also record the decision in .akar/EVENT_LOG.jsonl (opt-in)");
    println!("  doctor      Read-only health check of AKAR and project config");
    println!("  doctor --fix  Apply safe fixes for detected issues (backs up before overwriting)");
    println!("  bootstrap   Initialize missing AKAR memory files for a project");
    println!("  verify      Run task-specific verification and report honestly");
    println!("  eval [prompt]  Classify a prompt into a task contract (omit prompt for help)");
    println!("  hooks          Show hook script paths and installation instructions");
    println!("  safety <cmd>   Classify a shell command's risk level (Safe/Medium/High/Critical)");
    println!("  mission <prompt>  Run the full mission state machine for a prompt");
    println!("  skills            List registered skills and check for kernel conflicts");
    println!("  calibrate         Show model/gateway profile for the current session");
    println!("  postmortem        Analyze mission failures and generate learning patches");
    println!("  telemetry         Show compact operational metrics from EVENT_LOG.jsonl");
    println!("  learn             Generate a learning patch from latest postmortem evidence");
    println!("  run <task>        Stable workflow: doctor → preflight → mission → postmortem");
    println!("  preflight <task>  Show mission strategy before executing a task");
    println!("  request           Show request pressure mode and strategy advisory");
    println!("  request --used N --limit M  Supply explicit request counts for pressure mode");
    println!("  request --check   Validate .akar/NEXT_RUN.md (read-only; exit 0 on PASS, non-zero on FAIL)");
    println!();
    println!("FLAGS:");
    println!("  --version   Print version");
    println!("  --help      Print this help");
}

fn cmd_init(skip: bool, claude_integration: bool) {
    let result = init::run_init(skip, claude_integration);
    print!("{}", init::format_init_report(&result));
}

fn cmd_status() {
    let cfg = config::Config::discover();

    // Doctor state
    let issues = doctor::run_checks(&cfg);
    let doctor_state = if issues.is_empty() { "OK" } else { "DEGRADED" };

    // Bootstrap state
    let bootstrap_state = if cfg.akar_dir.exists() { "OK" } else { "not bootstrapped" };

    // Telemetry count
    let log_path = cfg.akar_dir.join("EVENT_LOG.jsonl");
    let telem_summary = event_log::summarize_log(&log_path, 1);
    let telem_state = if telem_summary.exists {
        format!("{} event(s)", telem_summary.total_events)
    } else {
        "none".to_string()
    };

    // Postmortem outcome
    let pm = postmortem::run_postmortem(&log_path);
    let pm_outcome = pm.latest_outcome.as_str();

    // Skill conflicts (cheap — scan project only)
    let project_commands = cfg.project_root.join(".claude").join("commands");
    let skills = skill_registry::scan_skills(&project_commands);
    let skill_report = skill_registry::build_skill_report(&skills);
    let skill_state = if skill_report.conflicts.is_empty() {
        "OK".to_string()
    } else {
        format!("{} conflict(s)", skill_report.conflicts.len())
    };

    // Request mode
    let signals = request_intelligence::RequestSignals { used: None, limit: None, prompt: None };
    let advisory = request_intelligence::build_advisory(&cfg, &signals);
    let request_mode = advisory.mode.as_str();

    // Baseline loop readiness (read-only git check)
    let readiness = diff_budget::check_loop_readiness(&cfg.project_root, &cfg.akar_dir);

    let health = if issues.is_empty() { "HEALTHY" } else { "DEGRADED" };
    println!("status: {}", health);
    println!("  runtime:    akar {}", VERSION);
    println!("  project:    {}", cfg.project_name);
    println!("  doctor:     {}", doctor_state);
    println!("  bootstrap:  {}", bootstrap_state);
    println!("  telemetry:  {}", telem_state);
    println!("  postmortem: {}", pm_outcome);
    println!("  skills:     {}", skill_state);
    println!("  request:    {}", request_mode);
    println!("  ram_budget: <{} MB target", RAM_BUDGET_MB);
    println!();
    print!("{}", diff_budget::format_loop_readiness(&readiness));

    if matches!(readiness.readiness, diff_budget::LoopReadiness::Blocked) {
        println!("  guidance: {}", foundation::git_dirty_playbook());
    }

    // Knowledge-driven loop governor: chooses the next safe loop action from
    // local evidence + foundation playbooks. Advisory only — never executes.
    let governor = loop_governor::decide(&cfg);
    print!("{}", loop_governor::format_loop_governor(&governor));

    if !issues.is_empty() {
        println!("  issues:");
        for issue in &issues {
            println!("    - {}", issue.message);
        }
    }
}

/// `akar governor` — concise, machine-readable loop governor decision.
///
/// Reads the same governor decision as `akar status` but prints it in a
/// standalone, scrape-friendly form. Supports `--one-line` (a single
/// `DECISION<TAB>SUGGESTED_PROMPT` line) and `--json` (a single JSON object).
///
/// Returns an orchestrator exit code based on the decision (v0.17.0):
/// READY/SNAPSHOT_NOW=0, RUN_POSTMORTEM=10, COMMIT_CHECKPOINT=11,
/// SPLIT_TASK=12, STOP_HOOK_BROKEN=20, STOP_REPEATED_BLOCK=21, UNKNOWN=30.
/// `no_exit_code` forces the return to 0 while keeping identical output.
///
/// `telemetry` (v0.18.0, opt-in via `--telemetry` or `AKAR_GOVERNOR_TELEMETRY=1`)
/// appends one governor event to `.akar/EVENT_LOG.jsonl`. Default writes
/// nothing.
///
/// Advisory only: does not mutate git, does not execute the suggested action.
fn cmd_governor(one_line: bool, json_mode: bool, no_exit_code: bool, telemetry: bool) -> i32 {
    let cfg = config::Config::discover();
    let report = loop_governor::decide(&cfg);

    let mode = if one_line {
        loop_governor::GovernorTelemetryMode::OneLine
    } else if json_mode {
        loop_governor::GovernorTelemetryMode::Json
    } else {
        loop_governor::GovernorTelemetryMode::Human
    };

    if one_line {
        println!("{}", loop_governor::format_governor_one_line(&report));
    } else if json_mode {
        println!("{}", loop_governor::format_governor_json(&report));
    } else {
        print!("{}", loop_governor::format_governor_report(&report));
    }

    let exit_code = if no_exit_code {
        0
    } else {
        loop_governor::exit_code_for_decision(&report)
    };

    if telemetry {
        // Opt-in telemetry: record this governor call in the local event log.
        // The suggested prompt is intentionally NOT logged.
        let _ = loop_governor::write_governor_telemetry(&cfg, &report, mode, no_exit_code, exit_code);
    }

    exit_code
}

fn cmd_run(prompt: &str, used: Option<u64>, limit: Option<u64>) {
    let cfg = config::Config::discover();
    let report = workflow::run_workflow(prompt, &cfg, used, limit);
    print!("{}", workflow::format_workflow_report(&report));
}

fn cmd_doctor(fix_mode: bool) {
    let cfg = config::Config::discover();
    let issues = doctor::run_checks(&cfg);

    if issues.is_empty() {
        println!("doctor: OK");
        return;
    }

    println!(
        "doctor: {} issue(s) found{}",
        issues.len(),
        if fix_mode { " — applying safe fixes" } else { "" }
    );

    for issue in &issues {
        let severity = match issue.severity {
            doctor::Severity::Error => "ERROR",
            doctor::Severity::Warning => "WARN",
            doctor::Severity::Info => "INFO",
        };
        println!("  [{}] {}", severity, issue.message);
    }

    if !fix_mode {
        println!("  hint: run 'akar doctor --fix' to apply safe fixes");
        return;
    }

    // --fix mode: build a SafeFix for each fixable issue and apply it.
    // Template dir defaults to the project .akar/templates directory.
    let template_dir = cfg.akar_dir.join("templates");

    println!();
    println!("fixes:");
    let mut fixed = 0usize;
    let mut failed = 0usize;

    for issue in &issues {
        let fix = match &issue.fix_hint {
            Some(doctor::FixHint::CreateDir(path)) => {
                Some(safe_fix::SafeFix::CreateMissingDir(path.clone()))
            }
            Some(doctor::FixHint::CreateFromTemplate { dest, template_name }) => {
                Some(safe_fix::SafeFix::CreateMissingTemplate {
                    dest: dest.clone(),
                    template_name: template_name.clone(),
                })
            }
            None => None,
        };

        match fix {
            Some(f) => match safe_fix::apply_safe_fix(&f, &template_dir) {
                Ok(msg) => {
                    println!("  ok:   {}", msg);
                    fixed += 1;
                }
                Err(e) => {
                    eprintln!("  fail: {}", e);
                    failed += 1;
                }
            },
            None => {
                println!("  skip: no automatic fix for: {}", issue.message);
            }
        }
    }

    println!();
    println!(
        "doctor --fix: {} fixed, {} failed, {} skipped",
        fixed,
        failed,
        issues.len() - fixed - failed
    );

    if failed > 0 {
        process::exit(1);
    }
}

fn cmd_bootstrap() {
    let cfg = config::Config::discover();
    let result = bootstrap::run_bootstrap(&cfg);
    print!("{}", bootstrap::format_bootstrap_report(&result));
}

fn cmd_verify() {
    let project_root = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    let recipe = verify::detect_recipe(&project_root);
    let results = verify::run_recipe(&recipe, &project_root);
    print!("{}", verify::format_results(&results, &recipe));
}

fn cmd_eval(prompt: Option<&str>) {
    match prompt {
        Some(p) => {
            let contract = contract::classify_prompt(p);
            println!("{}", contract::format_contract(&contract));
        }
        None => {
            let cfg = config::Config::discover();
            let suite = eval::run_evals(&cfg);
            print!("{}", eval::format_eval_report(&suite));
            if suite.failed > 0 {
                process::exit(1);
            }
        }
    }
}

fn cmd_safety(command: Option<&str>) {
    match command {
        Some(cmd) => {
            let assessment = safety::classify_command(cmd);
            let risk = format!("{:?}", assessment.risk);
            let status = if assessment.blocked { "BLOCKED" } else { "allowed" };
            println!("safety assessment:");
            println!("  command: {}", assessment.command);
            println!("  risk:    {}", risk);
            println!("  status:  {}", status);
            println!("  reason:  {}", assessment.reason);

            let leaks = safety::check_secrets(cmd);
            if !leaks.is_empty() {
                println!("  secret warnings:");
                for w in &leaks {
                    println!("    - {}", w);
                }
            }

            if assessment.blocked {
                println!("  guidance: {}", foundation::blocked_shell_playbook(cmd));
                process::exit(2);
            }
        }
        None => {
            println!("safety: no command given");
            println!("  usage: akar safety \"<shell command>\"");
            println!("  example: akar safety \"git push --force\"");
        }
    }
}

fn cmd_hooks(check: bool, install: bool) {
    let cfg = config::Config::discover();

    if check {
        let result = hooks::check_hooks(&cfg);
        print!("{}", hooks::format_hooks_check(&result));
        if !result.all_valid {
            process::exit(1);
        }
        return;
    }

    if install {
        let dest = cfg.akar_dir.join("hooks");
        println!("hooks install:");
        println!("  This will copy hook templates into: {}", dest.display());
        println!("  Files: pre-tool-call.sh, pre-tool-call.ps1");
        println!("  Existing files will be backed up before overwrite.");
        println!("  AKAR will NOT modify ~/.claude/settings.json.");
        println!("  You must connect these hooks to Claude Code manually.");
        println!();
        println!("  Type INSTALL to confirm, or anything else to cancel:");

        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        let confirmed = input.trim() == "INSTALL";

        if !confirmed {
            println!("  cancelled — no changes made");
            return;
        }

        let result = hooks::install_hooks(&cfg, true);
        print!("{}", hooks::format_hooks_install(&result));
        if result.cancelled {
            process::exit(1);
        }
        return;
    }

    print!("{}", hooks::format_hooks_help());
}

fn cmd_skills() {
    let cfg = config::Config::discover();
    let claude_dir = cfg.global_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| config::home_dir().join(".claude"));

    let skills = skill_registry::scan_multi(&claude_dir, &cfg.project_root);
    let report = skill_registry::build_skill_report(&skills);
    print!("{}", skill_registry::format_skill_report(&report));

    // Write inventory to .akar/SKILL_INVENTORY.md if .akar exists.
    if let Some(path) = skill_registry::write_skill_inventory(&cfg, &skills, &report) {
        println!("  inventory: {}", path.display());
    }
}

fn cmd_mission(prompt: &str) {
    let cfg = config::Config::discover();
    let m = mission::run_mission(prompt, &cfg);
    print!("{}", mission::format_mission_report(&m));
}

fn cmd_calibrate() {
    let (model_id, gateway) = model_profile::detect_model();
    let mut profile = model_profile::default_profile(&model_id);
    profile.gateway = gateway;
    print!("{}", model_profile::format_profile(&profile));
}

fn cmd_postmortem(diff_mode: bool, baseline_mode: bool, task_name: Option<String>) {
    let cfg = config::Config::discover();
    let log_path = cfg.akar_dir.join("EVENT_LOG.jsonl");
    let report = postmortem::run_postmortem(&log_path);
    print!("{}", postmortem::format_postmortem_report(&report));

    if !diff_mode {
        return;
    }

    // --baseline mode: use saved baseline commit and budget.
    if baseline_mode {
        let baseline = match diff_budget::read_baseline(&cfg.akar_dir) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("postmortem --diff --baseline: {}", e);
                process::exit(1);
            }
        };

        println!("postmortem --diff --baseline:");
        println!("  baseline timestamp: {}", baseline.timestamp);
        println!("  baseline task:      {}", baseline.task_type);
        println!("  baseline head:      {}", baseline.head_commit);
        println!("  baseline budget:    {} files, {} LOC", baseline.budget_files_max, baseline.budget_loc_max);

        let measurement = diff_budget::measure_diff_from_commit(
            &cfg.project_root,
            &baseline.head_commit,
        );
        let verdict = diff_budget::compare_budget(
            &measurement,
            baseline.budget_files_max,
            baseline.budget_loc_max,
        );

        let diff_report = diff_budget::DiffReport {
            measurement: measurement.clone(),
            verdict: verdict.clone(),
            budget_files_max: baseline.budget_files_max,
            budget_loc_max: baseline.budget_loc_max,
            task_type: baseline.task_type.clone(),
        };
        print!("{}", diff_budget::format_diff_report(&diff_report));

        if matches!(verdict, diff_budget::BudgetVerdict::Exceeded { .. }) {
            println!("  guidance: {}", foundation::budget_exceeded_playbook());
        }

        if matches!(verdict, diff_budget::BudgetVerdict::Exceeded { .. }) && cfg.akar_dir.exists() {
            write_diff_learning_patch(
                &cfg,
                &measurement,
                &verdict,
                baseline.budget_files_max,
                baseline.budget_loc_max,
                &baseline.task_type,
            );
        }
        return;
    }

    // Resolve task budget.
    let (files_max, loc_max, canonical_name, is_default) = match task_name.as_deref() {
        None => (3usize, 60usize, "Bugfix", true),
        Some(t) => match diff_budget::budget_for_task_name(t) {
            Ok((f, l, name)) => (f, l, name, false),
            Err(e) => {
                eprintln!("postmortem --diff: {}", e);
                process::exit(1);
            }
        },
    };

    if is_default {
        println!("postmortem --diff: using Bugfix budget by default");
        println!("  hint: use --task <type> for a different budget");
        println!("  valid: bugfix, feature, refactor, security, migration, dependency, frontend, docs, test, config");
    }

    let measurement = diff_budget::measure_diff(&cfg.project_root);
    let verdict = diff_budget::compare_budget(&measurement, files_max, loc_max);

    let task_label = if is_default {
        format!("{} (default)", canonical_name)
    } else {
        canonical_name.to_string()
    };

    let diff_report = diff_budget::DiffReport {
        measurement: measurement.clone(),
        verdict: verdict.clone(),
        budget_files_max: files_max,
        budget_loc_max: loc_max,
        task_type: task_label,
    };
    print!("{}", diff_budget::format_diff_report(&diff_report));

    if matches!(verdict, diff_budget::BudgetVerdict::Exceeded { .. }) {
        println!("  guidance: {}", foundation::budget_exceeded_playbook());
    }

    // Append learning patch when budget exceeded.
    if matches!(verdict, diff_budget::BudgetVerdict::Exceeded { .. }) && cfg.akar_dir.exists() {
        write_diff_learning_patch(&cfg, &measurement, &verdict, files_max, loc_max, canonical_name);
    }
}

fn write_diff_learning_patch(
    cfg: &config::Config,
    measurement: &diff_budget::DiffMeasurement,
    verdict: &diff_budget::BudgetVerdict,
    files_max: usize,
    loc_max: usize,
    task: &str,
) {
    use std::fs::OpenOptions;
    use std::io::Write;

    let patch_path = cfg.akar_dir.join("LEARNING_PATCHES.md");
    let ts = event_log::now_iso8601();
    let reason = if let diff_budget::BudgetVerdict::Exceeded { reason } = verdict {
        reason.clone()
    } else {
        String::new()
    };
    let patch = format!(
        "\n## LP-DIFF-{ts}\n\
        - date: {ts}\n\
        - source: postmortem --diff\n\
        - project: {project}\n\
        - task: {task}\n\
        - budget: {bf} files, {bl} LOC\n\
        - actual: {af} files, {al} total changed LOC\n\
        - exceeded: {reason}\n\
        - rule: Next prompt must reduce scope or split the task.\n\
        - status: active\n",
        ts = ts,
        project = cfg.project_name,
        task = task,
        bf = files_max,
        bl = loc_max,
        af = measurement.file_count,
        al = measurement.total_changed_lines,
        reason = reason,
    );
    let needs_header = !patch_path.exists();
    if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(&patch_path) {
        if needs_header {
            let _ = write!(f, "# AKAR Learning Patches\n<!-- Append-only. -->\n");
        }
        let _ = write!(f, "{}", patch);
        println!("  learning patch written: {}", patch_path.display());
    }
}

fn cmd_learn(list_mode: bool, resolve_mode: bool) {
    let cfg = config::Config::discover();

    if list_mode {
        let patch_path = cfg.akar_dir.join("LEARNING_PATCHES.md");
        print!("{}", learn::format_patch_list(&patch_path));
        return;
    }

    if resolve_mode {
        let patch_path = cfg.akar_dir.join("LEARNING_PATCHES.md");
        let now = event_log::now_iso8601();
        match learn::resolve_active_patches(&patch_path, &now) {
            None => {
                println!("learn --resolve: no LEARNING_PATCHES.md found");
                println!("  nothing to resolve.");
            }
            Some(0) => {
                println!("learn --resolve: no active entries found");
                println!("  file: {}", patch_path.display());
                println!("  all entries are already resolved.");
            }
            Some(n) => {
                println!("learn --resolve: {} active entr{} resolved", n, if n == 1 { "y" } else { "ies" });
                println!("  file: {}", patch_path.display());
                println!("  resolved_at: {}", now);
                println!("  resolved entries stay recorded but no longer affect the loop governor.");
            }
        }
        return;
    }

    let result = learn::run_learn(&cfg);
    print!("{}", learn::format_learn_result(&result));
}

fn cmd_preflight(prompt: &str, used: Option<u64>, limit: Option<u64>, snapshot: bool) {
    let cfg = config::Config::discover();
    let report = preflight::run_preflight(prompt, &cfg, used, limit);
    print!("{}", preflight::format_preflight_report(&report));

    if snapshot {
        // Refuse to write baseline if working tree is dirty.
        match diff_budget::is_working_tree_clean(&cfg.project_root) {
            Err(e) => {
                eprintln!("preflight --snapshot: {}", e);
                process::exit(1);
            }
            Ok(false) => {
                eprintln!("preflight --snapshot: working tree is dirty");
                eprintln!("  AKAR needs a clean baseline to measure session work.");
                eprintln!("  Commit or stash changes first, then run preflight --snapshot.");
                process::exit(1);
            }
            Ok(true) => {}
        }

        let head = match diff_budget::get_head_commit(&cfg.project_root) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("preflight --snapshot: {}", e);
                process::exit(1);
            }
        };

        // Parse budget from contract classification.
        let tc = contract::classify_prompt(prompt);
        let baseline = diff_budget::DiffBaseline {
            timestamp: event_log::now_iso8601(),
            prompt: config::redact(&prompt.chars().take(200).collect::<String>()),
            head_commit: head.clone(),
            task_type: format!("{:?}", tc.task_type),
            budget_files_max: tc.diff_budget.files_max,
            budget_loc_max: tc.diff_budget.loc_max,
        };

        if !cfg.akar_dir.exists() {
            eprintln!("preflight --snapshot: .akar/ not found — run 'akar bootstrap' first");
            process::exit(1);
        }

        match diff_budget::write_baseline(&cfg.akar_dir, &baseline) {
            Ok(()) => {
                println!("snapshot: baseline written");
                println!("  head:   {}", head);
                println!("  task:   {}", baseline.task_type);
                println!("  budget: {} files, {} LOC", baseline.budget_files_max, baseline.budget_loc_max);
                println!("  file:   {}", cfg.akar_dir.join("DIFF_BASELINE.json").display());
                println!("  next:   run your Claude Code session, then 'akar postmortem --diff --baseline'");
            }
            Err(e) => {
                eprintln!("preflight --snapshot: {}", e);
                process::exit(1);
            }
        }
    }
}

fn cmd_request(used: Option<u64>, limit: Option<u64>, prompt: Option<String>) {
    let cfg = config::Config::discover();
    let signals = request_intelligence::RequestSignals { used, limit, prompt };
    let advisory = request_intelligence::build_advisory(&cfg, &signals);
    print!("{}", request_intelligence::format_advisory(&advisory));

    // Knowledge-driven loop governor: surface the next safe loop action and
    // write a governor-aware NEXT_RUN.md. Advisory only — never executes.
    //
    // `akar request` writes `.akar/NEXT_RUN.md` exactly once, through the
    // compiled 11-section prompt writer (`write_governor_next_run`). The
    // older resume-mode `write_next_run` writer is intentionally NOT called
    // here: it was shadowed by this unconditional overwrite (v0.21 audit
    // §7c.3), so its "never overwrite" guard was moot and its output was
    // always discarded. `akar request --check` is read-only and does not
    // write; `akar governor` does not write NEXT_RUN.md.
    let governor = loop_governor::decide(&cfg);
    println!();
    print!("{}", loop_governor::format_loop_governor(&governor));
    if let Some(path) = loop_governor::write_governor_next_run(&cfg, &governor) {
        println!("  wrote: {}", path.display());
    }
}

/// `akar request --check` — read-only validator for `.akar/NEXT_RUN.md`.
///
/// Reads the compiled next-run prompt and validates it against the v0.20.0
/// contract (sections, minimum content, safety contract, decision
/// consistency). Prints PASS/FAIL. Does not write, regenerate, or auto-fix
/// anything. Returns 0 on PASS, non-zero on FAIL (or when the file is
/// missing).
fn cmd_request_check() -> i32 {
    let cfg = config::Config::discover();
    let path = cfg.akar_dir.join("NEXT_RUN.md");
    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            println!("NEXT_RUN check: FAIL");
            println!("  - file not found: {}", path.display());
            println!("    hint: run 'akar request' to generate it first");
            return 1;
        }
    };
    let result = loop_governor::validate_next_run(&content);
    print!("{}", loop_governor::format_next_run_check(&result));
    if result.pass {
        0
    } else {
        1
    }
}

fn parse_flag_u64(args: &[String], flag: &str) -> Option<u64> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse::<u64>().ok())
}

fn parse_flag_str(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].clone())
}

fn cmd_telemetry() {
    let cfg = config::Config::discover();
    let log_path = cfg.akar_dir.join("EVENT_LOG.jsonl");
    let summary = event_log::summarize_log(&log_path, 10);

    if !summary.exists {
        println!("telemetry: no events recorded yet");
        println!("  log: {}", log_path.display());
        println!("  hint: run 'akar mission <prompt>' to record your first event");
        return;
    }

    println!("telemetry: {} event(s) total", summary.total_events);
    println!("  log: {}", log_path.display());
    if !summary.recent.is_empty() {
        println!("  recent ({}):", summary.recent.len());
        for line in &summary.recent {
            // Print a trimmed preview of each JSON line
            let preview = if line.len() > 120 { &line[..120] } else { line };
            println!("    {}", preview);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_constant_is_nonempty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn version_matches_cargo_pkg_version() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }

    // -- foundation integration: safety BLOCKED output -----------------------

    #[test]
    fn safety_blocked_output_includes_safe_alternative() {
        let cmd = "rm -rf /some/path";
        let assessment = safety::classify_command(cmd);
        if assessment.blocked {
            let guidance = foundation::blocked_shell_playbook(cmd);
            assert!(
                guidance.contains("safe alternative") || guidance.contains("inspect") || guidance.contains("local"),
                "blocked safety output must include safe alternative guidance"
            );
        }
    }

    #[test]
    fn safety_blocked_shell_playbook_does_not_retry() {
        let guidance = foundation::blocked_shell_playbook("git push --force");
        assert!(
            !guidance.to_lowercase().contains("retry"),
            "guidance must not suggest retrying the blocked command"
        );
    }

    // -- foundation integration: status BLOCKED git guidance -----------------

    #[test]
    fn status_blocked_git_guidance_mentions_commit() {
        let guidance = foundation::git_dirty_playbook();
        assert!(
            guidance.contains("git commit") || guidance.contains("commit"),
            "status BLOCKED guidance must mention committing work"
        );
    }

    #[test]
    fn status_blocked_git_guidance_does_not_mention_reset() {
        let guidance = foundation::git_dirty_playbook();
        assert!(!guidance.contains("git reset"), "must not suggest git reset");
    }

    // -- foundation integration: postmortem EXCEEDED guidance ----------------

    #[test]
    fn postmortem_exceeded_guidance_mentions_split_task() {
        let guidance = foundation::budget_exceeded_playbook();
        assert!(
            guidance.contains("split") || guidance.contains("reduce scope"),
            "postmortem EXCEEDED guidance must mention split task or reduce scope"
        );
    }

    // -- foundation integration: hooks check FAIL guidance -------------------

    #[test]
    fn hooks_check_fail_output_includes_hook_broken_guidance() {
        let result = hooks::HooksCheckResult {
            templates_found: vec![],
            templates_missing: vec!["pre-tool-call.sh".to_string()],
            all_valid: false,
        };
        let output = hooks::format_hooks_check(&result);
        assert!(
            output.contains("PATH") || output.contains("restart") || output.contains("guidance"),
            "hooks check FAIL output must include hook broken guidance"
        );
    }
}
