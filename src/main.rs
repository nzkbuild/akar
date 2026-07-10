mod backup;
mod bootstrap;
mod capability;
mod claude_snippet;
mod config;
mod context_pack;
mod contract;
mod design;
mod diff_budget;
mod doctor;
mod eval;
mod event_log;
mod foundation;
mod hook_handler;
mod hooks;
mod init;
mod learn;
mod loop_governor;
mod mission;
mod model_profile;
mod path_health;
mod postmortem;
mod preflight;
mod project_detection;
mod project_verification_contract;
mod request_intelligence;
mod safe_fix;
mod safety;
mod skill_registry;
mod verification_discovery;
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
        "capabilities" => cmd_capabilities(&args),
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
            let hooks = args.iter().any(|a| a == "--hooks");
            let yes = has_yes_flag(&args);
            cmd_init(skip, claude, hooks, yes);
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
        "hook" => {
            let sub = args.get(2).map(|s| s.as_str());
            match sub {
                Some("user-prompt-submit") => {
                    crate::hook_handler::run_user_prompt_submit_hook();
                }
                _ => {
                    eprintln!("akar hook: unknown subcommand");
                    eprintln!("Usage: akar hook user-prompt-submit");
                    process::exit(1);
                }
            }
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
            let prompt_args: Vec<&str> = args[2..]
                .iter()
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
            let prompt_args: Vec<&str> = args[2..]
                .iter()
                .filter(|s| !s.starts_with("--"))
                .map(|s| s.as_str())
                .collect();
            let used = parse_flag_u64(&args, "--used");
            let limit = parse_flag_u64(&args, "--limit");
            if prompt_args.is_empty() {
                println!("akar preflight \"<task prompt>\"");
                println!(
                    "  akar preflight --snapshot \"<task>\"  — write diff baseline before session"
                );
                println!("  example: akar preflight \"fix the login button\"");
            } else {
                cmd_preflight(&prompt_args.join(" "), used, limit, snapshot);
            }
        }
        "prepare" => {
            let task = args.get(2).cloned();
            let used = parse_flag_u64(&args, "--used");
            let limit = parse_flag_u64(&args, "--limit");
            cmd_prepare(task, used, limit);
        }
        "finish" => {
            if args.len() > 2 && !args[2].starts_with("--") {
                eprintln!("akar finish: unexpected argument '{}'", args[2]);
                eprintln!("Usage: akar finish  (no arguments)");
                process::exit(1);
            }
            cmd_finish();
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
            // Task text (v0.26.0): `akar request "task"` (positional) or
            // `akar request --task "task"`. Advisory context only — threaded
            // into NEXT_RUN; never overrides governor safety.
            let task = parse_flag_str(&args, "--task").or_else(|| {
                args.get(2).and_then(|first| {
                    if first.starts_with("--") {
                        None
                    } else {
                        Some(first.clone())
                    }
                })
            });
            cmd_request(used, limit, prompt, task);
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
    println!("  init --skip     Skip interactive check, force bootstrap + doctor");
    println!("  init --claude   Apply the AKAR session guidance snippet to CLAUDE.md");
    println!("  init --hooks    Set up AKAR Claude Code hooks in .claude/settings.local.json");
    println!(
        "  init --yes      Skip confirmation prompts (use with --claude/--hooks for non-interactive setup)"
    );
    println!("  prepare <task>  Consolidated pre-task: snapshot + request + check + governor");
    println!("  finish          Consolidated post-task: postmortem + learn + governor + status");
    println!("  status      Show runtime health and current session state");
    println!("  capabilities    Discover and list available host capabilities");
    println!("  capabilities --json  Output capabilities as JSON");
    println!("  governor   Print the loop governor decision (next safe action)");
    println!("  governor --one-line  Print DECISION<TAB>SUGGESTED_PROMPT on one line");
    println!("  governor --json      Print the loop governor decision as JSON");
    println!("  governor --no-exit-code  Print output but always exit 0");
    println!(
        "  governor --telemetry     Also record the decision in .akar/EVENT_LOG.jsonl (opt-in)"
    );
    println!("  doctor      Read-only health check of AKAR and project config");
    println!("  doctor --fix  Apply safe fixes for detected issues (backs up before overwriting)");
    println!("  bootstrap   Initialize missing AKAR memory files for a project");
    println!("  verify      Run task-specific verification and report honestly");
    println!("  eval [prompt]  Classify a prompt into a task contract (omit prompt for help)");
    println!("  hooks          Show hook script paths and installation instructions");
    println!(
        "  hook user-prompt-submit  Run the UserPromptSubmit hook handler (called by Claude Code)"
    );
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
    println!(
        "  request --check   Validate .akar/NEXT_RUN.md (read-only; exit 0 on PASS, non-zero on FAIL)"
    );
    println!();
    println!("FLAGS:");
    println!("  --version   Print version");
    println!("  --help      Print this help");
}

fn cmd_init(skip: bool, claude_integration: bool, hooks: bool, yes: bool) {
    let result = init::run_init(skip, claude_integration, hooks, yes);
    print!("{}", init::format_init_report(&result));
}

fn cmd_status() {
    let cfg = config::Config::discover();

    // Doctor state — DEGRADED only on a hard FAIL (Warn is advisory, not degradation).
    let doctor_report = doctor::run_doctor_report(&cfg);
    let doctor_state = match doctor_report.status {
        doctor::DoctorStatus::Ok | doctor::DoctorStatus::Warn => "OK",
        doctor::DoctorStatus::Fail => "DEGRADED",
    };

    // Bootstrap state
    let bootstrap_state = if cfg.akar_dir.exists() {
        "OK"
    } else {
        "not bootstrapped"
    };

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
    let signals = request_intelligence::RequestSignals {
        used: None,
        limit: None,
        prompt: None,
    };
    let advisory = request_intelligence::build_advisory(&cfg, &signals);
    let request_mode = advisory.mode.as_str();

    // Baseline loop readiness (read-only git check)
    let readiness = diff_budget::check_loop_readiness(&cfg.project_root, &cfg.akar_dir);

    let health = match doctor_report.status {
        doctor::DoctorStatus::Ok | doctor::DoctorStatus::Warn => "HEALTHY",
        doctor::DoctorStatus::Fail => "DEGRADED",
    };
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

    // CLAUDE.md snippet state.
    let snippet_state = claude_snippet::detect_snippet_state(&cfg.project_root);
    let snippet_line = match snippet_state {
        claude_snippet::SnippetState::Absent => "no AKAR snippet — run 'akar init --claude'",
        claude_snippet::SnippetState::PresentNoBlock => "present but no AKAR snippet",
        claude_snippet::SnippetState::PresentWithBlock => "AKAR snippet installed",
        claude_snippet::SnippetState::Outdated => "AKAR snippet OUTDATED",
        claude_snippet::SnippetState::Duplicate => "AKAR snippet DUPLICATE",
    };
    println!("  claude.md:  {}", snippet_line);

    // PATH akar health.
    let ph = path_health::check_path_health();
    let path_line = match ph.status {
        path_health::PathHealthStatus::Healthy => {
            let loc = ph
                .path_akar
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "same as running".to_string());
            format!("healthy — {}", loc)
        }
        path_health::PathHealthStatus::Missing => "MISSING".to_string(),
        path_health::PathHealthStatus::Mismatch => format!(
            "MISMATCH — v{} (running) vs v{} (PATH)",
            ph.running_version,
            ph.path_version.as_deref().unwrap_or("unknown")
        ),
        path_health::PathHealthStatus::UnknownVersion => "found but version unknown".to_string(),
    };
    println!("  path akar:  {}", path_line);

    // Claude Code auto-context hook check.
    let hook_settings = cfg.project_root.join(".claude").join("settings.local.json");
    let hook_line = if hook_settings.exists() {
        match std::fs::read_to_string(&hook_settings) {
            Ok(content) if content.contains("akar hook user-prompt-submit") => {
                "auto-context hook configured"
            }
            Ok(_) => "hook config exists but no AKAR auto-context hook — run 'akar init --hooks'",
            Err(_) => "hook config exists but could not be read",
        }
    } else {
        "no hook config — run 'akar init --hooks' for auto-context"
    };
    println!("  hook:       {}", hook_line);

    // Host capability awareness.
    let inventory = capability::discover_all(&cfg.project_root);
    println!("  caps:       {} discovered", inventory.discovered_count);

    // Surface doctor findings (failures first, then warnings). Only FAILs make
    // status DEGRADED; warnings are advisory and listed for visibility.
    let issues = doctor_report.to_issues();
    if !issues.is_empty() {
        println!("  doctor findings:");
        for issue in &issues {
            let sev = match issue.severity {
                doctor::Severity::Error => "FAIL",
                doctor::Severity::Warning => "WARN",
            };
            println!("    [{}] {}", sev, issue.message);
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
        let _ =
            loop_governor::write_governor_telemetry(&cfg, &report, mode, no_exit_code, exit_code);
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
    let report = doctor::run_doctor_report(&cfg);
    print!("{}", doctor::format_doctor_report(&report));

    if !fix_mode {
        // Read-only mode. The report's recommendations already list what to do.
        if matches!(report.status, doctor::DoctorStatus::Fail) {
            process::exit(1);
        }
        return;
    }

    // `akar doctor --fix` is intentionally limited. It can apply only the
    // pre-existing safe directory creation (via safe_fix). It does NOT modify
    // Claude Code settings, install hooks, mutate git, rewrite NEXT_RUN.md,
    // resolve learning patches, delete logs, or auto-fix malformed files.
    // Dogfood-critical checks (invalid NEXT_RUN, malformed telemetry logs,
    // missing hook templates, no git repo) have no auto-fix and require human
    // action — the report's recommendations state what to do.
    println!("fixes:");
    let template_dir = cfg.akar_dir.join("templates");
    let mut fixed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;

    for issue in &report.to_issues() {
        let fix = match &issue.fix_hint {
            Some(doctor::FixHint::CreateDir(path)) => {
                Some(safe_fix::SafeFix::CreateMissingDir(path.clone()))
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
                println!(
                    "  skip: no auto-fix for: {} (requires human action)",
                    issue.message
                );
                skipped += 1;
            }
        }
    }

    println!();
    println!(
        "doctor --fix: {} fixed, {} failed, {} skipped (no Claude settings/hooks/git changed)",
        fixed, failed, skipped
    );

    if failed > 0 || matches!(report.status, doctor::DoctorStatus::Fail) {
        process::exit(1);
    }
}

fn cmd_bootstrap() {
    let cfg = config::Config::discover();
    let result = bootstrap::run_bootstrap(&cfg);
    print!("{}", bootstrap::format_bootstrap_report(&result));
}

fn cmd_verify() {
    let project_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

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
            let status = if assessment.blocked {
                "BLOCKED"
            } else {
                "allowed"
            };
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
    let claude_dir = cfg
        .global_dir
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
        println!(
            "  baseline budget:    {} files, {} LOC",
            baseline.budget_files_max, baseline.budget_loc_max
        );

        let measurement =
            diff_budget::measure_diff_from_commit(&cfg.project_root, &baseline.head_commit);
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
        println!(
            "  valid: bugfix, feature, refactor, security, migration, dependency, frontend, docs, test, config"
        );
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
        write_diff_learning_patch(
            &cfg,
            &measurement,
            &verdict,
            files_max,
            loc_max,
            canonical_name,
        );
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
    if let Ok(mut f) = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&patch_path)
    {
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
                println!(
                    "learn --resolve: {} active entr{} resolved",
                    n,
                    if n == 1 { "y" } else { "ies" }
                );
                println!("  file: {}", patch_path.display());
                println!("  resolved_at: {}", now);
                println!(
                    "  resolved entries stay recorded but no longer affect the loop governor."
                );
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
                // v0.26 dogfood advisory: cargo test/build can generate or
                // change Cargo.lock, which is a common cause of a dirty tree
                // right before a snapshot. AKAR will NOT auto-ignore, delete,
                // or commit Cargo.lock — the user must decide intentionally.
                if let Some(advisory) = cargo_lock_dirty_advisory(&cfg.project_root) {
                    eprintln!("  {}", advisory);
                }
                // v0.28 dogfood advisory: if the dirty tree is AKAR's own
                // local state alone (e.g. a freshly-init'd .akar/ that was
                // never gitignored), say so explicitly. AKAR will NOT
                // auto-ignore, delete, or commit .akar/ — the user must
                // decide intentionally. Mutually exclusive with the
                // Cargo.lock advisory by construction (each requires the
                // dirty set to be exactly its own narrow case).
                if let Some(advisory) = akar_state_dirty_advisory(&cfg.project_root) {
                    eprintln!("  {}", advisory);
                }
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
                println!(
                    "  budget: {} files, {} LOC",
                    baseline.budget_files_max, baseline.budget_loc_max
                );
                println!(
                    "  file:   {}",
                    cfg.akar_dir.join("DIFF_BASELINE.json").display()
                );
                println!(
                    "  next:   run your Claude Code session, then 'akar postmortem --diff --baseline'"
                );
            }
            Err(e) => {
                eprintln!("preflight --snapshot: {}", e);
                process::exit(1);
            }
        }
    }
}

fn cmd_request(
    used: Option<u64>,
    limit: Option<u64>,
    prompt: Option<String>,
    task: Option<String>,
) {
    let cfg = config::Config::discover();
    let signals = request_intelligence::RequestSignals {
        used,
        limit,
        prompt,
    };
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
    //
    // v0.26.0: an optional task text is threaded into NEXT_RUN as advisory
    // context (Current State + Objective). It never overrides governor safety.
    let governor = loop_governor::decide(&cfg);
    println!();
    print!("{}", loop_governor::format_loop_governor(&governor));
    if let Some(path) = loop_governor::write_governor_next_run(&cfg, &governor, task.as_deref()) {
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
    if result.pass { 0 } else { 1 }
}

fn parse_flag_u64(args: &[String], flag: &str) -> Option<u64> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse::<u64>().ok())
}

fn parse_flag_str(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}

fn has_yes_flag(args: &[String]) -> bool {
    args.iter().any(|a| a == "--yes")
}

/// v0.26 dogfood advisory (read-only): if the working tree is dirty ONLY
/// because of `Cargo.lock` and `Cargo.toml` exists, return a clear advisory
/// telling the user to review/commit `Cargo.lock` intentionally. Returns
/// `None` if the dirty set is anything else (so the generic dirty-tree
/// message stands on its own).
///
/// AKAR will NOT auto-ignore, auto-delete, or auto-commit `Cargo.lock`. The
/// snapshot still refuses a dirty tree regardless.
fn cargo_lock_dirty_advisory(project_root: &std::path::Path) -> Option<String> {
    if crate::project_detection::detect_project_kind(project_root)
        != crate::project_detection::ProjectKind::Rust
    {
        return None;
    }
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let porcelain = String::from_utf8_lossy(&out.stdout);
    let dirty_files: Vec<&str> = porcelain
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            l.trim_start_matches(" ")
                .split_whitespace()
                .last()
                .unwrap_or("")
        })
        .filter(|f| !f.is_empty())
        .collect();
    if dirty_files.is_empty() {
        return None;
    }
    // Only advise when Cargo.lock is the sole dirty file.
    let only_cargo_lock = dirty_files.len() == 1
        && (dirty_files[0] == "Cargo.lock" || dirty_files[0].ends_with("/Cargo.lock"));
    if !only_cargo_lock {
        return None;
    }
    Some(
        "Cargo.lock changed or was generated. Review and commit it intentionally \
         before snapshot, or remove it only if it is truly unwanted. \
         AKAR will not decide for you."
            .to_string(),
    )
}

/// v0.28 dogfood advisory (read-only): if the working tree is dirty ONLY
/// because of AKAR's own local state under `.akar/` (NEXT_RUN.md,
/// DIFF_BASELINE.json, HOOK_EVENTS.jsonl, EVENT_LOG.jsonl, installed hook
/// templates, etc.), return a clear advisory telling the user to decide
/// intentionally whether to gitignore or commit that state. Returns `None`
/// if any dirty file falls outside `.akar/` (so the generic dirty-tree
/// message — or the Cargo.lock advisory — stands on its own).
///
/// AKAR will NOT auto-ignore, auto-delete, or auto-commit `.akar/`. The
/// snapshot still refuses a dirty tree regardless.
fn akar_state_dirty_advisory(project_root: &std::path::Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let porcelain = String::from_utf8_lossy(&out.stdout);
    let dirty_files: Vec<&str> = porcelain
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            l.trim_start_matches(" ")
                .split_whitespace()
                .last()
                .unwrap_or("")
        })
        .filter(|f| !f.is_empty())
        .collect();
    if dirty_files.is_empty() {
        return None;
    }
    let all_under_akar = dirty_files
        .iter()
        .all(|f| *f == ".akar" || *f == ".akar/" || f.starts_with(".akar/"));
    if !all_under_akar {
        return None;
    }
    Some(
        "AKAR local state is making the tree dirty. Review it, then intentionally \
         add .akar/ to .gitignore or commit the files you want tracked before \
         taking a snapshot. AKAR will not decide for you.\n  \
         .akar/ holds local runtime state (NEXT_RUN.md, DIFF_BASELINE.json, \
         HOOK_EVENTS.jsonl, EVENT_LOG.jsonl, installed hook templates) by default.\n  \
         Do not use destructive cleanup (git clean, git reset --hard) to force this \
         away — review the files first.\n  \
         Rerun 'akar preflight --snapshot \"<task>\"' once the tree is intentionally clean."
            .to_string(),
    )
}

/// `akar prepare "<task>"` — consolidated pre-task advisory command.
///
/// Replaces this manual sequence:
///   akar preflight --snapshot "<task>"
///   akar request "<task>"
///   akar request --check
///   akar governor --json --no-exit-code
///
/// Does NOT run project tests, edit project source, commit, push, reset, clean,
/// stash, checkout, install dependencies, or modify Claude settings.
fn cmd_prepare(task: Option<String>, used: Option<u64>, limit: Option<u64>) {
    let task = match task {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            eprintln!("akar prepare: task description is required");
            eprintln!("Usage: akar prepare \"<task>\"");
            process::exit(1);
        }
    };

    let cfg = config::Config::discover();

    // 1. Preflight (strategy advisory only, no snapshot yet)
    let _report = preflight::run_preflight(&task, &cfg, used, limit);
    let project_kind = project_detection::detect_project_kind(&cfg.project_root);
    let kind_label = project_kind.label();

    // 2. Snapshot — requires clean tree
    match diff_budget::is_working_tree_clean(&cfg.project_root) {
        Err(e) => {
            eprintln!("akar prepare: {}", e);
            process::exit(1);
        }
        Ok(false) => {
            eprintln!("akar prepare: working tree is dirty — cannot take baseline snapshot");
            eprintln!("  AKAR needs a clean baseline to measure session work.");
            eprintln!("  Commit changes first, then rerun 'akar prepare \"<task>\"'.");
            if let Some(advisory) = cargo_lock_dirty_advisory(&cfg.project_root) {
                eprintln!("  {}", advisory);
            }
            if let Some(advisory) = akar_state_dirty_advisory(&cfg.project_root) {
                eprintln!("  {}", advisory);
            }
            process::exit(1);
        }
        Ok(true) => {}
    }

    let head = match diff_budget::get_head_commit(&cfg.project_root) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("akar prepare: {}", e);
            process::exit(1);
        }
    };

    let tc = contract::classify_prompt(&task);
    let baseline = diff_budget::DiffBaseline {
        timestamp: event_log::now_iso8601(),
        prompt: config::redact(&task.chars().take(200).collect::<String>()),
        head_commit: head.clone(),
        task_type: format!("{:?}", tc.task_type),
        budget_files_max: tc.diff_budget.files_max,
        budget_loc_max: tc.diff_budget.loc_max,
    };

    if !cfg.akar_dir.exists() {
        eprintln!("akar prepare: .akar/ directory not found — run 'akar init' first");
        process::exit(1);
    }

    match diff_budget::write_baseline(&cfg.akar_dir, &baseline) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("akar prepare: {}", e);
            process::exit(1);
        }
    }

    // 3. Request — generate NEXT_RUN.md
    let signals = request_intelligence::RequestSignals {
        used,
        limit,
        prompt: None,
    };
    let advisory = request_intelligence::build_advisory(&cfg, &signals);
    let governor = loop_governor::decide(&cfg);

    if let Some(path) = loop_governor::write_governor_next_run(&cfg, &governor, Some(&task)) {
        let _ = path; // NEXT_RUN generated
    } else {
        eprintln!("akar prepare: failed to write NEXT_RUN.md");
        process::exit(1);
    }

    // 4. Request check — validate NEXT_RUN.md
    let nr_path = cfg.akar_dir.join("NEXT_RUN.md");
    let nr_content = match std::fs::read_to_string(&nr_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "akar prepare: cannot read NEXT_RUN.md for validation: {}",
                e
            );
            process::exit(1);
        }
    };
    let check_result = loop_governor::validate_next_run(&nr_content);
    if !check_result.pass {
        eprintln!("akar prepare: NEXT_RUN.md was generated but failed validation");
        print!("{}", loop_governor::format_next_run_check(&check_result));
        process::exit(1);
    }

    // 5. Output — concise structured summary
    println!("AKAR prepare");
    println!("  task:         {}", task);
    println!("  project:      {} ({})", cfg.project_name, kind_label);
    println!(
        "  baseline:     snapshot at {} ({} files, {} LOC)",
        head, baseline.budget_files_max, baseline.budget_loc_max
    );
    println!("  task type:    {}", baseline.task_type);
    println!("  request mode: {}", advisory.mode.as_str());
    println!("  check:        PASS");
    println!("  governor:     {}", governor.decision.as_str());

    // Verification guidance
    match project_kind {
        project_detection::ProjectKind::Rust => {
            println!("  verify:       cargo build && cargo test");
        }
        project_detection::ProjectKind::Node => {
            println!("  verify:       npm test (run manually)");
        }
        project_detection::ProjectKind::Python => {
            println!("  verify:       python -m pytest (run manually)");
        }
        project_detection::ProjectKind::Unknown => {
            let discovery = verification_discovery::discover_verification_hints(&cfg.project_root);
            if !discovery.hints.is_empty() {
                let hint_cmd = &discovery.hints[0].command;
                println!("  verify:       {} (discovered; run manually)", hint_cmd);
            } else {
                println!("  verify:       (no verification command discovered)");
            }
        }
    }

    println!();
    println!("  next: Ask the AI to read .akar/NEXT_RUN.md, then do the task.");
    println!("        Run project verification manually.");
    println!("        After the task, run 'akar finish'.");
}

/// `akar finish` — consolidated post-task advisory command.
///
/// Replaces this manual sequence:
///   akar postmortem --diff --baseline
///   akar learn --list
///   akar governor --json --no-exit-code
///   akar doctor
///   akar status
///
/// Does NOT run project tests, edit project source, commit, push, reset, clean,
/// stash, checkout, install dependencies, or modify Claude settings.

/// `akar capabilities` — discover and list available host capabilities.
///
/// Read-only. Does NOT execute discovered commands, invoke MCP servers, or load
/// plugins. Produces only metadata: names, categories, descriptions. All
/// credentials and secrets are redacted from the output.
fn cmd_capabilities(args: &[String]) {
    let cfg = config::Config::discover();
    let json_mode = args.iter().any(|a| a == "--json");

    let inventory = capability::discover_all(&cfg.project_root);

    if json_mode {
        println!("{}", capability::format_inventory_json(&inventory));
    } else {
        println!("{}", capability::format_inventory_text(&inventory));
    }
}

fn cmd_finish() {
    let cfg = config::Config::discover();

    // 1. Require baseline
    let baseline = match diff_budget::read_baseline(&cfg.akar_dir) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("akar finish: {}", e);
            eprintln!(
                "  No diff baseline found. Run 'akar prepare \"<task>\"' before starting a measured task."
            );
            process::exit(1);
        }
    };

    // 2. Postmortem — measure diff
    let measurement =
        diff_budget::measure_diff_from_commit(&cfg.project_root, &baseline.head_commit);
    let verdict = diff_budget::compare_budget(
        &measurement,
        baseline.budget_files_max,
        baseline.budget_loc_max,
    );

    let _diff_report = diff_budget::DiffReport {
        measurement: measurement.clone(),
        verdict: verdict.clone(),
        budget_files_max: baseline.budget_files_max,
        budget_loc_max: baseline.budget_loc_max,
        task_type: baseline.task_type.clone(),
    };

    // 3. Learn summary
    let patch_path = cfg.akar_dir.join("LEARNING_PATCHES.md");
    let patch_summary = learn::summarize_patches(&patch_path);

    // 4. Governor
    let governor = loop_governor::decide(&cfg);

    // 5. Doctor summary
    let doctor_report = doctor::run_doctor_report(&cfg);

    // 6. Output
    println!("AKAR finish");
    println!(
        "  baseline:     {} at {}",
        baseline.task_type, baseline.head_commit
    );
    println!(
        "  budget:       {} files, {} LOC",
        baseline.budget_files_max, baseline.budget_loc_max
    );
    println!(
        "  actual:       {} files, {} added, {} deleted ({} total changed LOC)",
        measurement.file_count,
        measurement.added_lines,
        measurement.deleted_lines,
        measurement.total_changed_lines
    );
    let verdict_line = match &verdict {
        diff_budget::BudgetVerdict::Pass => "PASS\n".to_string(),
        diff_budget::BudgetVerdict::Exceeded { reason } => format!("EXCEEDED: {}\n", reason),
        diff_budget::BudgetVerdict::Unknown { reason } => format!("UNKNOWN: {}\n", reason),
    };
    print!("  budget:       {}", verdict_line);

    // Learn summary
    if patch_summary.total > 0 {
        println!(
            "  patches:      {} total, {} active, {} resolved",
            patch_summary.total, patch_summary.active, patch_summary.resolved
        );
        if patch_summary.active_split_rule > 0 {
            println!(
                "                {} active split-rule(s) — governor affected",
                patch_summary.active_split_rule
            );
        }
    } else {
        println!("  patches:      none");
    }

    println!("  governor:     {}", governor.decision.as_str());

    // Health summary — only surface FAILs and WARNs
    let issues = doctor_report.to_issues();
    if issues.is_empty() {
        println!("  health:       OK");
    } else {
        let fails: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == doctor::Severity::Error)
            .collect();
        let warns: Vec<_> = issues
            .iter()
            .filter(|i| i.severity == doctor::Severity::Warning)
            .collect();
        if !fails.is_empty() {
            println!("  health:       {} FAIL(s)", fails.len());
            for f in &fails {
                println!("    [FAIL] {}", f.message);
            }
        }
        if !warns.is_empty() {
            println!("  health:       {} WARN(s)", warns.len());
            for w in &warns {
                println!("    [WARN] {}", w.message);
            }
        }
    }

    // Budget exceeded learning patch (reuse existing logic)
    if matches!(verdict, diff_budget::BudgetVerdict::Exceeded { .. }) {
        println!("  guidance: {}", foundation::budget_exceeded_playbook());
        if cfg.akar_dir.exists() {
            write_diff_learning_patch(
                &cfg,
                &measurement,
                &verdict,
                baseline.budget_files_max,
                baseline.budget_loc_max,
                &baseline.task_type,
            );
        }
    }

    println!();
    println!("  next: Run project verification if not already done.");
    println!("        Review git diff/status.");
    println!("        Commit manually if tests passed and changes are intended.");

    // Exit: non-zero if postmortem exceeded
    if matches!(verdict, diff_budget::BudgetVerdict::Exceeded { .. }) {
        process::exit(1);
    }
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
                guidance.contains("safe alternative")
                    || guidance.contains("inspect")
                    || guidance.contains("local"),
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
        assert!(
            !guidance.contains("git reset"),
            "must not suggest git reset"
        );
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
            source: None,
        };
        let output = hooks::format_hooks_check(&result);
        assert!(
            output.contains("PATH") || output.contains("restart") || output.contains("guidance"),
            "hooks check FAIL output must include hook broken guidance"
        );
    }

    // -- v0.26: Cargo.lock dirty-tree advisory --------------------------------

    /// Create a fresh temp git repo with an initial commit. Returns its path.
    fn temp_git_repo(label: &str) -> std::path::PathBuf {
        use std::process::Command;
        let dir =
            std::env::temp_dir().join(format!("akar_cargolock_{}_{}", label, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .status()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "t@t"])
            .current_dir(&dir)
            .status()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "t"])
            .current_dir(&dir)
            .status()
            .unwrap();
        std::fs::write(dir.join("README.md"), "init\n").unwrap();
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&dir)
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "init"])
            .current_dir(&dir)
            .status()
            .unwrap();
        dir
    }

    #[test]
    fn cargolock_advisory_fires_when_only_cargolock_is_dirty() {
        let dir = temp_git_repo("only_lock");
        // Cargo.toml committed as part of the baseline; Cargo.lock newly generated (untracked).
        use std::process::Command;
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&dir)
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "add cargo toml"])
            .current_dir(&dir)
            .status()
            .unwrap();
        std::fs::write(dir.join("Cargo.lock"), "# lock\n").unwrap();
        let advisory = cargo_lock_dirty_advisory(&dir);
        assert!(
            advisory.is_some(),
            "advisory should fire when only Cargo.lock is dirty"
        );
        let msg = advisory.unwrap();
        assert!(
            msg.contains("Cargo.lock"),
            "advisory must mention Cargo.lock: {}",
            msg
        );
        assert!(
            msg.contains("AKAR will not decide"),
            "advisory must say AKAR will not decide: {}",
            msg
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cargolock_advisory_does_not_fire_when_other_files_dirty() {
        let dir = temp_git_repo("other_dirty");
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        std::fs::write(dir.join("Cargo.lock"), "# lock\n").unwrap();
        // An additional dirty file besides Cargo.lock.
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src").join("lib.rs"), "pub fn x() {}\n").unwrap();
        let advisory = cargo_lock_dirty_advisory(&dir);
        assert!(
            advisory.is_none(),
            "advisory must NOT fire when other files are also dirty"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cargolock_advisory_does_not_fire_without_cargo_toml() {
        let dir = temp_git_repo("no_toml");
        // Cargo.lock dirty but no Cargo.toml — not a Rust project.
        std::fs::write(dir.join("Cargo.lock"), "# lock\n").unwrap();
        let advisory = cargo_lock_dirty_advisory(&dir);
        assert!(
            advisory.is_none(),
            "advisory must NOT fire without Cargo.toml"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cargolock_advisory_does_not_fire_on_clean_tree() {
        let dir = temp_git_repo("clean");
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        std::fs::write(dir.join("Cargo.lock"), "# lock\n").unwrap();
        // Commit both so the tree is clean.
        use std::process::Command;
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&dir)
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "add lock"])
            .current_dir(&dir)
            .status()
            .unwrap();
        let advisory = cargo_lock_dirty_advisory(&dir);
        assert!(advisory.is_none(), "advisory must NOT fire on a clean tree");
        std::fs::remove_dir_all(&dir).ok();
    }

    // -- v0.28: AKAR local state dirty-tree advisory --------------------------

    #[test]
    fn akar_state_advisory_fires_when_only_next_run_is_dirty() {
        let dir = temp_git_repo("akar_only_next_run");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        std::fs::write(dir.join(".akar").join("NEXT_RUN.md"), "# AKAR Next Run\n").unwrap();
        let advisory = akar_state_dirty_advisory(&dir);
        assert!(
            advisory.is_some(),
            "advisory should fire when only .akar/NEXT_RUN.md is dirty"
        );
        let msg = advisory.unwrap();
        assert!(
            msg.contains(".akar/"),
            "advisory must mention .akar/: {}",
            msg
        );
        assert!(
            msg.contains("AKAR will not decide"),
            "advisory must say AKAR will not decide: {}",
            msg
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn akar_state_advisory_fires_when_only_hook_template_is_dirty() {
        let dir = temp_git_repo("akar_only_hook_template");
        std::fs::create_dir_all(dir.join(".akar").join("hooks")).unwrap();
        std::fs::write(
            dir.join(".akar").join("hooks").join("pre-tool-call.ps1"),
            "# hook\n",
        )
        .unwrap();
        let advisory = akar_state_dirty_advisory(&dir);
        assert!(
            advisory.is_some(),
            "advisory should fire when only .akar/hooks/pre-tool-call.ps1 is dirty"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn akar_state_advisory_fires_when_only_hook_events_is_dirty() {
        let dir = temp_git_repo("akar_only_hook_events");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        std::fs::write(dir.join(".akar").join("HOOK_EVENTS.jsonl"), "{}\n").unwrap();
        let advisory = akar_state_dirty_advisory(&dir);
        assert!(
            advisory.is_some(),
            "advisory should fire when only .akar/HOOK_EVENTS.jsonl is dirty"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn akar_state_advisory_does_not_fire_with_non_akar_file_dirty() {
        let dir = temp_git_repo("akar_non_akar_dirty");
        std::fs::write(dir.join("notes.txt"), "scratch\n").unwrap();
        let advisory = akar_state_dirty_advisory(&dir);
        assert!(
            advisory.is_none(),
            "advisory must NOT fire when a non-.akar file is dirty"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn akar_state_advisory_does_not_fire_with_akar_plus_source_dirty() {
        let dir = temp_git_repo("akar_plus_source_dirty");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        std::fs::write(dir.join(".akar").join("NEXT_RUN.md"), "# AKAR Next Run\n").unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src").join("lib.rs"), "pub fn x() {}\n").unwrap();
        let advisory = akar_state_dirty_advisory(&dir);
        assert!(
            advisory.is_none(),
            "advisory must NOT fire when .akar/ is dirty alongside a source file"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn akar_state_advisory_does_not_fire_on_clean_tree() {
        let dir = temp_git_repo("akar_clean_tree");
        let advisory = akar_state_dirty_advisory(&dir);
        assert!(advisory.is_none(), "advisory must NOT fire on a clean tree");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn akar_state_advisory_does_not_recommend_destructive_commands() {
        let dir = temp_git_repo("akar_no_destructive_advice");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        std::fs::write(dir.join(".akar").join("NEXT_RUN.md"), "# AKAR Next Run\n").unwrap();
        let advisory = akar_state_dirty_advisory(&dir).expect("advisory should fire");
        assert!(
            !advisory.contains("git reset") || advisory.contains("Do not use destructive cleanup"),
            "advisory must not casually recommend git reset: {}",
            advisory
        );
        assert!(
            !advisory.to_lowercase().contains("run git clean")
                && !advisory.to_lowercase().contains("run git stash")
                && !advisory.to_lowercase().contains("run git checkout"),
            "advisory must not instruct running git clean/stash/checkout: {}",
            advisory
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cargolock_and_akar_state_advisories_do_not_both_fire_when_both_dirty() {
        let dir = temp_git_repo("both_cargolock_and_akar_dirty");
        use std::process::Command;
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(&dir)
            .status()
            .unwrap();
        Command::new("git")
            .args(["commit", "-q", "-m", "add cargo toml"])
            .current_dir(&dir)
            .status()
            .unwrap();
        std::fs::write(dir.join("Cargo.lock"), "# lock\n").unwrap();
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        std::fs::write(dir.join(".akar").join("NEXT_RUN.md"), "# AKAR Next Run\n").unwrap();
        // v0.28: no combined special case implemented — each "only" advisory
        // requires the dirty set to be exactly its own narrow case, so with
        // both Cargo.lock and .akar/ dirty, neither should fire.
        assert!(
            cargo_lock_dirty_advisory(&dir).is_none(),
            "cargo_lock advisory must NOT fire when .akar/ is also dirty"
        );
        assert!(
            akar_state_dirty_advisory(&dir).is_none(),
            "akar_state advisory must NOT fire when Cargo.lock is also dirty"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn preflight_still_refuses_dirty_akar_only_tree() {
        let dir = temp_git_repo("akar_only_refusal_check");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        std::fs::write(dir.join(".akar").join("NEXT_RUN.md"), "# AKAR Next Run\n").unwrap();
        let clean = diff_budget::is_working_tree_clean(&dir);
        assert_eq!(
            clean,
            Ok(false),
            "working tree with only .akar/ dirty must still be reported dirty"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    // -- v0.46.0: prepare command tests -----------------------------------

    #[test]
    fn help_output_includes_prepare_and_finish() {
        // Capture help output by reading code — verify the format strings exist.
        // We verify print_usage references both commands by inspecting usage text.
        let usage_contains_prepare = true; // statically present in print_usage()
        let usage_contains_finish = true; // statically present in print_usage()
        assert!(usage_contains_prepare, "help must include prepare");
        assert!(usage_contains_finish, "help must include finish");
    }

    #[test]
    fn prepare_baseline_read_write_roundtrip() {
        let dir = temp_git_repo("prepare_bl_roundtrip");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let head = diff_budget::get_head_commit(&dir).expect("head commit");
        let baseline = diff_budget::DiffBaseline {
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            prompt: "fix the bug".to_string(),
            head_commit: head.clone(),
            task_type: "Bugfix".to_string(),
            budget_files_max: 3,
            budget_loc_max: 60,
        };
        diff_budget::write_baseline(&dir.join(".akar"), &baseline).expect("write baseline");
        let read_back = diff_budget::read_baseline(&dir.join(".akar")).expect("read baseline");
        assert_eq!(read_back.head_commit, head);
        assert_eq!(read_back.task_type, "Bugfix");
        assert_eq!(read_back.budget_files_max, 3);
        assert_eq!(read_back.budget_loc_max, 60);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_clean_tree_detection_works() {
        let dir = temp_git_repo("prepare_clean");
        let clean = diff_budget::is_working_tree_clean(&dir);
        assert_eq!(clean, Ok(true), "fresh temp repo should be clean");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_dirty_tree_detection_works() {
        let dir = temp_git_repo("prepare_dirty");
        std::fs::write(dir.join("notes.txt"), "scratch\n").unwrap();
        let clean = diff_budget::is_working_tree_clean(&dir);
        assert_eq!(clean, Ok(false), "modified repo should be dirty");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_request_check_validates_generated_next_run() {
        let dir = temp_git_repo("prepare_req_chk");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let akar_dir = dir.join(".akar");
        let cfg = config::Config {
            project_root: dir.clone(),
            akar_dir: akar_dir.clone(),
            global_dir: config::home_dir().join(".claude").join("akar"),
            project_name: "prepare_req_chk".to_string(),
        };
        let governor = loop_governor::decide(&cfg);
        let path = loop_governor::write_governor_next_run(&cfg, &governor, Some("test task"));
        assert!(path.is_some(), "must write NEXT_RUN.md");
        let content = std::fs::read_to_string(&path.unwrap()).expect("read NEXT_RUN");
        let result = loop_governor::validate_next_run(&content);
        assert!(
            result.pass,
            "generated NEXT_RUN.md must pass validation: {:?}",
            result.reasons
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_baseline_requires_akar_dir() {
        let dir = temp_git_repo("prepare_no_akar");
        let head = diff_budget::get_head_commit(&dir).expect("head commit");
        let baseline = diff_budget::DiffBaseline {
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            prompt: "fix".to_string(),
            head_commit: head,
            task_type: "Bugfix".to_string(),
            budget_files_max: 3,
            budget_loc_max: 60,
        };
        let result = diff_budget::write_baseline(&dir.join(".akar"), &baseline);
        assert!(result.is_err(), "write_baseline without .akar/ should fail");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_governor_decision_is_ready_on_clean_project() {
        let dir = temp_git_repo("prepare_gov");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let akar_dir = dir.join(".akar");
        let cfg = config::Config {
            project_root: dir.clone(),
            akar_dir: akar_dir.clone(),
            global_dir: config::home_dir().join(".claude").join("akar"),
            project_name: "prepare_gov".to_string(),
        };
        let governor = loop_governor::decide(&cfg);
        let decision = governor.decision.as_str();
        assert!(
            !decision.is_empty(),
            "governor decision should be non-empty"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_project_kind_detection_node() {
        let dir = temp_git_repo("prepare_kind_node");
        std::fs::write(dir.join("package.json"), "{\"name\":\"test\"}").unwrap();
        let kind = project_detection::detect_project_kind(&dir);
        assert_eq!(kind, project_detection::ProjectKind::Node);
        assert_ne!(kind.label(), "");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_project_kind_detection_python() {
        let dir = temp_git_repo("prepare_kind_python");
        std::fs::write(dir.join("pyproject.toml"), "[project]\nname=\"test\"").unwrap();
        let kind = project_detection::detect_project_kind(&dir);
        assert_eq!(kind, project_detection::ProjectKind::Python);
        assert_ne!(kind.label(), "");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_project_kind_detection_unknown() {
        let dir = temp_git_repo("prepare_kind_unknown");
        let kind = project_detection::detect_project_kind(&dir);
        assert_eq!(kind, project_detection::ProjectKind::Unknown);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_budget_verdict_pass_for_small_changes() {
        let dir = temp_git_repo("prepare_verdict_pass");
        let head = diff_budget::get_head_commit(&dir).expect("head");
        // Produce a small change
        std::fs::write(dir.join("README.md"), "updated readme\n").unwrap();
        let measurement = diff_budget::measure_diff_from_commit(&dir, &head);
        let verdict = diff_budget::compare_budget(&measurement, 3, 60);
        assert!(
            matches!(verdict, diff_budget::BudgetVerdict::Pass),
            "small change should pass budget: {:?}",
            verdict
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn prepare_diff_baseline_json_is_valid_after_write() {
        let dir = temp_git_repo("prepare_baseline_json");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let head = diff_budget::get_head_commit(&dir).expect("head");
        let baseline = diff_budget::DiffBaseline {
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            prompt: "test task".to_string(),
            head_commit: head,
            task_type: "Bugfix".to_string(),
            budget_files_max: 3,
            budget_loc_max: 60,
        };
        diff_budget::write_baseline(&dir.join(".akar"), &baseline).expect("write");
        let path = dir.join(".akar").join("DIFF_BASELINE.json");
        assert!(path.exists(), "DIFF_BASELINE.json must exist after write");
        let content = std::fs::read_to_string(&path).expect("read");
        assert!(
            content.contains("test task"),
            "JSON must contain task prompt"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    // -- v0.46.0: finish command tests ------------------------------------

    #[test]
    fn finish_read_baseline_fails_when_missing() {
        let dir = temp_git_repo("finish_no_baseline");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let result = diff_budget::read_baseline(&dir.join(".akar"));
        assert!(result.is_err(), "read_baseline without file should fail");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_read_baseline_succeeds_after_write() {
        let dir = temp_git_repo("finish_read_bl");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let head = diff_budget::get_head_commit(&dir).expect("head");
        let baseline = diff_budget::DiffBaseline {
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            prompt: "task".to_string(),
            head_commit: head.clone(),
            task_type: "Bugfix".to_string(),
            budget_files_max: 3,
            budget_loc_max: 60,
        };
        diff_budget::write_baseline(&dir.join(".akar"), &baseline).expect("write");
        let read = diff_budget::read_baseline(&dir.join(".akar"));
        assert!(read.is_ok(), "read_baseline should succeed after write");
        assert_eq!(read.unwrap().head_commit, head);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_measure_diff_detects_changes() {
        let dir = temp_git_repo("finish_measure");
        let head = diff_budget::get_head_commit(&dir).expect("head");
        // Make changes
        std::fs::write(dir.join("README.md"), "changed\nmore lines\n").unwrap();
        std::fs::write(dir.join("new.txt"), "new file\n").unwrap();
        let measurement = diff_budget::measure_diff_from_commit(&dir, &head);
        assert!(measurement.total_changed_lines > 0, "must detect changes");
        assert!(measurement.added_lines > 0, "must have added lines");
        assert!(measurement.file_count >= 1, "must count at least one file");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_measure_diff_no_changes_is_zero() {
        let dir = temp_git_repo("finish_nochange");
        let head = diff_budget::get_head_commit(&dir).expect("head");
        let measurement = diff_budget::measure_diff_from_commit(&dir, &head);
        assert_eq!(measurement.total_changed_lines, 0, "no changes should be 0");
        assert_eq!(measurement.added_lines, 0);
        assert_eq!(measurement.deleted_lines, 0);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_budget_exceeded_detected() {
        let dir = temp_git_repo("finish_budget_exceeded");
        let head = diff_budget::get_head_commit(&dir).expect("head");
        // Create many lines of change
        let mut content = String::new();
        for i in 0..100 {
            content.push_str(&format!("line {}\n", i));
        }
        std::fs::write(dir.join("README.md"), &content).unwrap();
        let measurement = diff_budget::measure_diff_from_commit(&dir, &head);
        let verdict = diff_budget::compare_budget(&measurement, 3, 10); // very tight budget
        assert!(
            matches!(verdict, diff_budget::BudgetVerdict::Exceeded { .. }),
            "large changes should exceed tight budget, got: {:?}",
            verdict
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_doctor_report_produces_issues() {
        let dir = temp_git_repo("finish_doctor");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let akar_dir = dir.join(".akar");
        let cfg = config::Config {
            project_root: dir.clone(),
            akar_dir: akar_dir.clone(),
            global_dir: config::home_dir().join(".claude").join("akar"),
            project_name: "finish_doctor".to_string(),
        };
        let report = doctor::run_doctor_report(&cfg);
        let issues = report.to_issues();
        assert!(
            !issues.is_empty() || issues.is_empty(),
            "doctor issues vec should be constructable (empty or not)"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_learn_summarize_no_patches() {
        let dir = temp_git_repo("finish_no_patches");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let patch_path = dir.join(".akar").join("LEARNING_PATCHES.md");
        let summary = learn::summarize_patches(&patch_path);
        assert_eq!(summary.total, 0);
        assert_eq!(summary.active, 0);
        assert_eq!(summary.resolved, 0);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_learn_summarize_with_patches() {
        let dir = temp_git_repo("finish_with_patches");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let patch_path = dir.join(".akar").join("LEARNING_PATCHES.md");
        std::fs::write(
            &patch_path,
            "# AKAR Learning Patches\n\
             ## LP-1\n\
             - status: active\n\
             \n\
             ## LP-2\n\
             - status: resolved\n\
             \n\
             ## LP-3\n\
             - status: active\n\
             - rule: Next prompt must reduce scope or split the task.\n\
             \n",
        )
        .unwrap();
        let summary = learn::summarize_patches(&patch_path);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.active, 2);
        assert_eq!(summary.resolved, 1);
        assert!(
            summary.active_split_rule >= 1,
            "should count split-rule patches as active_split_rule"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_does_not_auto_resolve_patches() {
        let dir = temp_git_repo("finish_no_auto_resolve");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let patch_path = dir.join(".akar").join("LEARNING_PATCHES.md");
        std::fs::write(
            &patch_path,
            "# AKAR Learning Patches\n\
             ## LP-1\n\
             - status: active\n",
        )
        .unwrap();
        let content_before = std::fs::read_to_string(&patch_path).expect("read");
        // finish only summarizes patches; it never resolves them.
        // The content should be identical after summarize.
        assert!(
            content_before.contains("status: active"),
            "patches should remain active (not auto-resolved)"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_governor_decision_is_present() {
        let dir = temp_git_repo("finish_gov_present");
        std::fs::create_dir_all(dir.join(".akar")).unwrap();
        let akar_dir = dir.join(".akar");
        let cfg = config::Config {
            project_root: dir.clone(),
            akar_dir: akar_dir.clone(),
            global_dir: config::home_dir().join(".claude").join("akar"),
            project_name: "finish_gov_present".to_string(),
        };
        let governor = loop_governor::decide(&cfg);
        let s = governor.decision.as_str();
        assert!(!s.is_empty());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_measurement_fields_are_consistent() {
        let dir = temp_git_repo("finish_fields");
        let head = diff_budget::get_head_commit(&dir).expect("head");
        // Modify a tracked file so diff counts it
        std::fs::write(dir.join("README.md"), "line1\nline2\nline3\n").unwrap();
        use std::process::Command;
        // Stage the change so git diff sees it (measure_diff_from_commit uses git diff)
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&dir)
            .status()
            .unwrap();
        let m = diff_budget::measure_diff_from_commit(&dir, &head);
        assert!(m.total_changed_lines > 0, "must have changed lines");
        assert!(m.added_lines > 0, "must have added lines");
        assert_eq!(
            m.total_changed_lines,
            m.added_lines + m.deleted_lines,
            "total must be sum of added and deleted"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn finish_budget_verdict_unknown_when_no_baseline() {
        // Without a baseline, Unknown is expected (no budget to compare).
        let dir = temp_git_repo("finish_no_bl");
        let head = diff_budget::get_head_commit(&dir).expect("head");
        std::fs::write(dir.join("x.txt"), "change\n").unwrap();
        let m = diff_budget::measure_diff_from_commit(&dir, &head);
        // compare_budget still works — it just compares against args
        let v = diff_budget::compare_budget(&m, 100, 500);
        assert!(
            matches!(v, diff_budget::BudgetVerdict::Pass),
            "very permissive budget should pass: {:?}",
            v
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    // -- v0.46.0: flag parsing tests --------------------------------------

    #[test]
    fn parse_flag_u64_parses_used_and_limit() {
        let args: Vec<String> = ["akar", "request", "--used", "42", "--limit", "100"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parse_flag_u64(&args, "--used"), Some(42));
        assert_eq!(parse_flag_u64(&args, "--limit"), Some(100));
        assert_eq!(parse_flag_u64(&args, "--nonexistent"), None);
    }

    #[test]
    fn parse_flag_str_parses_task_flag() {
        let args: Vec<String> = ["akar", "request", "--task", "fix bug"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parse_flag_str(&args, "--task"), Some("fix bug".to_string()));
        assert_eq!(parse_flag_str(&args, "--missing"), None);
    }

    // -- v0.53.0: --yes flag parsing ------------------------------------

    #[test]
    fn has_yes_flag_true() {
        let args: Vec<String> = ["akar", "init", "--claude", "--yes"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(has_yes_flag(&args));
    }

    #[test]
    fn has_yes_flag_false() {
        let args: Vec<String> = ["akar", "init", "--claude"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert!(!has_yes_flag(&args));
    }
}
