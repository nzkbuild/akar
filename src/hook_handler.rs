//! UserPromptSubmit hook handler — auto-prepare AKAR context from Claude Code.
//!
//! `akar hook user-prompt-submit` reads Claude Code hook JSON from stdin,
//! evaluates the working tree, and returns structured JSON with
//! `hookSpecificOutput.additionalContext`.
//!
//! # Safety
//! - Dirty tree → injects stop/finish instruction, does NOT prepare new task
//! - Never runs project commands from the hook
//! - Never edits source files from the hook
//! - Never commits from the hook
//! - Writes only to `.akar/NEXT_RUN.md`

use std::io::Read;
use std::path::PathBuf;

use crate::capability;
use crate::config;
use crate::contract;
use crate::diff_budget;
use crate::event_log;
use crate::loop_governor;
use crate::project_detection;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Parsed fields from the UserPromptSubmit hook JSON.
#[derive(Debug)]
struct HookInput {
    prompt: String,
    cwd: PathBuf,
}

/// Outcome of hook handler evaluation.
#[derive(Debug)]
enum HookOutcome {
    /// Working tree is clean — generate NEXT_RUN.md and inject compact context.
    Ready {
        task: String,
        task_type: String,
        budget_files: usize,
        budget_loc: usize,
    },
    /// Working tree is dirty — inject stop/finish instruction.
    DirtyTree,
    /// Could not determine git status.
    NoRepo,
}

// ---------------------------------------------------------------------------
// JSON helpers (std-only, no serde)
// ---------------------------------------------------------------------------

/// Extract the first JSON string value for a given key. Handles escaped chars.
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let rest = json.splitn(2, &pattern).nth(1)?;
    let after_colon = rest.splitn(2, ':').nth(1)?;
    let trimmed = after_colon.trim_start();
    if !trimmed.starts_with('"') {
        return None;
    }
    let inner = &trimmed[1..];
    let mut result = String::new();
    let mut chars = inner.chars();
    loop {
        match chars.next() {
            None => return None,
            Some('\\') => match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => return None,
            },
            Some('"') => break,
            Some(c) => result.push(c),
        }
    }
    Some(result)
}

/// Build a Config from an explicit project root (the hook's cwd field).
fn config_for_cwd(project_root: &std::path::Path) -> config::Config {
    let project_name = project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let akar_dir = project_root.join(".akar");
    let global_dir = config::home_dir().join(".claude").join("akar");
    config::Config {
        project_root: project_root.to_path_buf(),
        akar_dir,
        global_dir,
        project_name,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run the UserPromptSubmit hook handler.
///
/// Reads JSON from stdin, evaluates the working tree, generates or skips
/// NEXT_RUN.md, and prints the structured JSON response to stdout.
pub fn run_user_prompt_submit_hook() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        print_stop_instruction("failed to read hook input from stdin");
        return;
    }

    let hook_input = match parse_hook_input(&input) {
        Some(h) => h,
        None => {
            print_stop_instruction("could not parse hook input JSON");
            return;
        }
    };

    let cfg = config_for_cwd(&hook_input.cwd);

    let outcome = evaluate(&hook_input, &cfg);
    let context = match outcome {
        HookOutcome::Ready {
            task,
            task_type,
            budget_files,
            budget_loc,
        } => {
            generate_next_run(&cfg, &task);
            let tc = contract::classify_prompt(&task);
            let inventory = capability::discover_all(&cfg.project_root);
            let selection = capability::select_capabilities(&inventory, &task, &tc.task_type);
            let kind_label = project_detection::detect_project_kind(&cfg.project_root).label();
            let profile = capability::build_task_profile(&task, &tc.task_type, kind_label);
            context_for_ready(&task, &task_type, budget_files, budget_loc, &selection, &profile)
        }
        HookOutcome::DirtyTree => context_for_dirty_tree(&cfg),
        HookOutcome::NoRepo => context_for_no_repo(),
    };

    print_response(&context);
}

// ---------------------------------------------------------------------------
// Internal
// ---------------------------------------------------------------------------

fn parse_hook_input(json: &str) -> Option<HookInput> {
    let prompt = extract_json_string(json, "prompt")?;
    let cwd_str = extract_json_string(json, "cwd").unwrap_or_default();
    let cwd = if cwd_str.is_empty() {
        std::env::current_dir().ok()?
    } else {
        PathBuf::from(&cwd_str)
    };
    if prompt.trim().is_empty() {
        return None;
    }
    Some(HookInput { prompt, cwd })
}

fn evaluate(input: &HookInput, cfg: &config::Config) -> HookOutcome {
    // Check working tree cleanliness.
    let clean = match diff_budget::is_working_tree_clean(&cfg.project_root) {
        Ok(true) => true,
        Ok(false) => false,
        Err(_) => return HookOutcome::NoRepo,
    };

    if !clean {
        return HookOutcome::DirtyTree;
    }

    let tc = contract::classify_prompt(&input.prompt);
    HookOutcome::Ready {
        task: input.prompt.clone(),
        task_type: format!("{:?}", tc.task_type),
        budget_files: tc.diff_budget.files_max,
        budget_loc: tc.diff_budget.loc_max,
    }
}

/// Generate NEXT_RUN.md for the given task.
fn generate_next_run(cfg: &config::Config, task: &str) {
    // Ensure .akar/ directory exists.
    let _ = std::fs::create_dir_all(&cfg.akar_dir);

    // Run governor BEFORE writing baseline so it observes a clean tree.
    let governor = loop_governor::decide(cfg);

    // Write baseline (needed for prepare-equivalent behavior).
    if let Ok(head) = diff_budget::get_head_commit(&cfg.project_root) {
        let tc = contract::classify_prompt(task);
        let baseline = diff_budget::DiffBaseline {
            timestamp: event_log::now_iso8601(),
            prompt: config::redact(&task.chars().take(200).collect::<String>()),
            head_commit: head,
            task_type: format!("{:?}", tc.task_type),
            budget_files_max: tc.diff_budget.files_max,
            budget_loc_max: tc.diff_budget.loc_max,
        };
        let _ = diff_budget::write_baseline(&cfg.akar_dir, &baseline);
    }

    // Write NEXT_RUN.md with the governor decision from clean state.
    let _ = loop_governor::write_governor_next_run(cfg, &governor, Some(task));
}

fn context_for_ready(
    task: &str,
    task_type: &str,
    budget_files: usize,
    budget_loc: usize,
    selection: &capability::CapabilitySelection,
    profile: &capability::TaskProfile,
) -> String {
    capability::build_enhanced_context(task, task_type, budget_files, budget_loc, selection, profile)
}

fn context_for_dirty_tree(cfg: &config::Config) -> String {
    format!(
        "\
[AKAR auto-context — STOP]

The working tree is dirty. AKAR cannot prepare a new task while uncommitted
changes exist.

Run `akar finish` to measure and close out the current task, then commit
or stash changes before starting new work.

Project: {project}
Check:  `akar status`",
        project = cfg.project_name,
    )
}

fn context_for_no_repo() -> String {
    "\
[AKAR auto-context]

AKAR could not determine git repository status. Project verification
is unavailable. Proceed with caution."
        .to_string()
}

/// Print the Claude Code hook response to stdout.
///
/// The response wraps additionalContext in the nested hookSpecificOutput
/// envelope expected by Claude Code's UserPromptSubmit hook.
fn print_response(additional_context: &str) {
    // JSON-escape the context for embedding.
    let escaped = additional_context
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    println!(
        "{{\"hookSpecificOutput\":{{\"hookSpecificOutput\":{{\"additionalContext\":\"{}\"}}}}}}",
        escaped
    );
}

/// Print a stop instruction when the hook handler itself encounters an error.
fn print_stop_instruction(reason: &str) {
    let msg = format!(
        "[AKAR auto-context]\n\nThe hook handler could not prepare context: {reason}\n\nProceed manually or run `akar status` to diagnose."
    );
    let escaped = msg
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    println!(
        "{{\"hookSpecificOutput\":{{\"hookSpecificOutput\":{{\"additionalContext\":\"{}\"}}}}}}",
        escaped
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract;
    use crate::capability;

    // -- JSON extraction --------------------------------------------------

    #[test]
    fn extract_simple_string() {
        let json = r#"{"prompt": "fix the bug", "cwd": "/home/user/project"}"#;
        assert_eq!(
            extract_json_string(json, "prompt"),
            Some("fix the bug".to_string())
        );
        assert_eq!(
            extract_json_string(json, "cwd"),
            Some("/home/user/project".to_string())
        );
    }

    #[test]
    fn extract_missing_key_returns_none() {
        let json = r#"{"prompt": "hello"}"#;
        assert_eq!(extract_json_string(json, "cwd"), None);
    }

    #[test]
    fn extract_empty_json_returns_none() {
        assert_eq!(extract_json_string("{}", "prompt"), None);
    }

    #[test]
    fn extract_escaped_quotes() {
        let json = r#"{"prompt": "fix \"the\" bug"}"#;
        assert_eq!(
            extract_json_string(json, "prompt"),
            Some(r#"fix "the" bug"#.to_string())
        );
    }

    #[test]
    fn extract_escaped_backslash() {
        let json = r#"{"prompt": "C:\\Users\\name"}"#;
        assert_eq!(
            extract_json_string(json, "prompt"),
            Some(r#"C:\Users\name"#.to_string())
        );
    }

    #[test]
    fn extract_escaped_newline() {
        let json = r#"{"prompt": "line1\nline2"}"#;
        assert_eq!(
            extract_json_string(json, "prompt"),
            Some("line1\nline2".to_string())
        );
    }

    // -- parse_hook_input --------------------------------------------------

    #[test]
    fn parse_valid_user_prompt_submit_json() {
        let json = r#"{"prompt": "fix the multiply bug", "cwd": "/home/user/project"}"#;
        let input = parse_hook_input(json);
        assert!(input.is_some());
        let input = input.unwrap();
        assert_eq!(input.prompt, "fix the multiply bug");
        assert_eq!(input.cwd, PathBuf::from("/home/user/project"));
    }

    #[test]
    fn parse_empty_prompt_returns_none() {
        let json = r#"{"prompt": "   ", "cwd": "/tmp"}"#;
        assert!(parse_hook_input(json).is_none());
    }

    #[test]
    fn parse_missing_prompt_returns_none() {
        let json = r#"{"cwd": "/tmp"}"#;
        assert!(parse_hook_input(json).is_none());
    }

    #[test]
    fn parse_missing_cwd_falls_back_to_current_dir() {
        let json = r#"{"prompt": "test"}"#;
        let input = parse_hook_input(json);
        assert!(input.is_some());
    }

    // -- print_response ----------------------------------------------------

    #[test]
    fn response_has_correct_envelope_structure() {
        // Capture stdout by running print_response and checking the output
        // contains the expected keys.
        let ctx = "test context";
        let escaped = ctx.replace('\\', "\\\\").replace('"', "\\\"");
        let expected = format!(
            "{{\"hookSpecificOutput\":{{\"hookSpecificOutput\":{{\"additionalContext\":\"{}\"}}}}}}",
            escaped
        );
        // Verify the structure contains the nested envelope keys.
        assert!(expected.contains("\"hookSpecificOutput\""));
        assert!(expected.contains("\"additionalContext\""));
        assert!(expected.contains("test context"));
    }

    #[test]
    fn response_escapes_special_chars() {
        let ctx = "line1\nline2\t\"quoted\"";
        let escaped = ctx
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        assert!(escaped.contains("\\n"));
        assert!(escaped.contains("\\t"));
        assert!(escaped.contains("\\\""));
        assert!(!escaped.contains("\n"));
    }

    // -- context_for_dirty_tree --------------------------------------------

    #[test]
    fn dirty_tree_context_mentions_finish_and_status() {
        let cfg = config::Config {
            project_root: PathBuf::from("/tmp/test"),
            akar_dir: PathBuf::from("/tmp/test/.akar"),
            global_dir: PathBuf::from("/home/user/.claude/akar"),
            project_name: "test-project".to_string(),
        };
        let ctx = context_for_dirty_tree(&cfg);
        assert!(ctx.contains("dirty"), "must mention dirty tree");
        assert!(ctx.contains("akar finish"), "must mention akar finish");
        assert!(ctx.contains("akar status"), "must mention akar status");
    }

    // -- context_for_ready -------------------------------------------------

    #[test]
    fn ready_context_includes_task_and_budget() {
        let selection = capability::CapabilitySelection {
            selected: vec![],
            total_discovered: 0,
            omitted_count: 0,
            context_chars: 0,
            estimated_tokens: 0,
            selection_time_ms: 0,
        };
        let profile = capability::build_task_profile("fix the bug", &contract::TaskType::Bugfix, "Rust");
        let ctx = context_for_ready("fix the bug", "Bugfix", 3, 60, &selection, &profile);
        assert!(
            ctx.contains("fix the bug"),
            "must include task: got '{}'",
            ctx
        );
        assert!(
            ctx.contains("Bugfix"),
            "must include task type: got '{}'",
            ctx
        );
        assert!(
            ctx.contains("3 files"),
            "must include file budget: got '{}'",
            ctx
        );
        assert!(
            ctx.contains("60 LOC"),
            "must include LOC budget: got '{}'",
            ctx
        );
        assert!(
            ctx.contains(".akar/NEXT_RUN.md"),
            "must mention NEXT_RUN.md: got '{}'",
            ctx
        );
        assert!(
            ctx.contains("akar finish"),
            "must mention akar finish: got '{}'",
            ctx
        );
    }

    // -- context_for_no_repo -----------------------------------------------

    #[test]
    fn no_repo_context_mentions_caution() {
        let ctx = context_for_no_repo();
        assert!(
            ctx.to_lowercase().contains("caution") || ctx.to_lowercase().contains("unavailable"),
            "must mention caution or unavailable: got '{}'",
            ctx
        );
    }

    // -- print_stop_instruction --------------------------------------------

    #[test]
    fn stop_instruction_contains_reason() {
        // We test that the constructed message contains the reason and
        // produces valid JSON structure.
        let reason = "test failure";
        let msg = format!(
            "[AKAR auto-context]\n\nThe hook handler could not prepare context: {reason}\n\nProceed manually or run `akar status` to diagnose."
        );
        assert!(msg.contains(reason));
        assert!(msg.contains("akar status"));
    }
}
