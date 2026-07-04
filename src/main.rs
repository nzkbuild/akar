mod backup;
mod bootstrap;
mod circuit_breaker;
mod config;
mod context_pack;
mod contract;
mod design;
mod doctor;
mod eval;
mod event_log;
mod mission;
mod model_profile;
mod safe_fix;
mod safety;
mod skill_registry;
mod verify;

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
        "doctor" => {
            let fix_mode = args.get(2).map(|s| s.as_str()) == Some("--fix");
            cmd_doctor(fix_mode);
        }
        "bootstrap" => cmd_bootstrap(),
        "verify" => cmd_verify(),
        "eval" => {
            let prompt = args.get(2).map(|s| s.as_str());
            cmd_eval(prompt);
        }
        "hooks" => cmd_hooks(),
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
        "postmortem" => cmd_postmortem(),
        "telemetry" => cmd_telemetry(),
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
    println!("  status      Show runtime health and current session state");
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
    println!();
    println!("FLAGS:");
    println!("  --version   Print version");
    println!("  --help      Print this help");
}

fn cmd_status() {
    let cfg = config::Config::discover();
    let issues = cfg.validate();

    let health = if issues.is_empty() { "HEALTHY" } else { "DEGRADED" };
    println!("status: {}", health);
    println!("  runtime:      akar {}", VERSION);
    println!("  project:      {}", cfg.project_name);
    println!("  project_root: {}", cfg.project_root.display());
    println!(
        "  akar_dir:     {} [{}]",
        cfg.akar_dir.display(),
        if cfg.akar_dir.exists() { "exists" } else { "missing" }
    );
    println!(
        "  global_dir:   {} [{}]",
        cfg.global_dir.display(),
        if cfg.global_dir.exists() { "exists" } else { "missing" }
    );

    let pack = context_pack::build_pack(&cfg);
    let hot_count = pack.files.iter().filter(|f| f.tier == context_pack::ContextTier::Hot).count();
    println!("  hot_context: {} file(s)", hot_count);

    let design_report = design::check_project(&cfg.project_root);
    let design_line = if design_report.issues.is_empty() {
        "OK".to_string()
    } else {
        format!("{} issue(s)", design_report.issues.len())
    };
    println!("  design:      {}", design_line);
    println!("  ram_budget:  <{} MB target (no daemon, no local LLM)", RAM_BUDGET_MB);

    if !issues.is_empty() {
        println!("  issues:");
        for issue in &issues {
            println!("    - {}", issue);
        }
    }
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

fn cmd_hooks() {
    println!("akar hooks:");
    println!("  bash hook:        hooks/pre-commit-akar.sh");
    println!("  powershell hook:  hooks/pre-commit-akar.ps1");
    println!("  settings example: .claude/settings.akar.json.example");
    println!("  install: merge hooks into your .git/hooks/ and settings into ~/.claude/settings.json");
}

fn cmd_skills() {
    let cfg = config::Config::discover();
    // Scan ~/.claude/ (the global Claude dir, parent of akar/)
    let claude_dir = cfg.global_dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| config::home_dir().join(".claude"));

    let skills = skill_registry::scan_skills(&claude_dir);
    print!("{}", skill_registry::format_registry(&skills));

    let warnings = skill_registry::check_kernel_priority(&skills);
    for w in &warnings {
        println!("{}", w);
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

fn cmd_postmortem() {
    println!("postmortem: not yet implemented");
    println!("  hint: postmortem analyzes mission failures and generates learning patches");
    println!("  status: stub (v0.1.1 architecture refinement)");
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
}
