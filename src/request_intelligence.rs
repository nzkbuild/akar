//! Request Intelligence v0 — local strategy advisor for request pressure.
//!
//! Does not connect to provider APIs. Uses local signals only:
//! - recent telemetry event count
//! - postmortem outcome
//! - manually supplied used/limit counts (optional)

#[allow(unused_imports)]
use std::path::Path;
use crate::{config, event_log, postmortem};

// ---------------------------------------------------------------------------
// Pressure modes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum PressureMode {
    Normal,
    Saver,
    Compact,
    Boundary,
    Resume,
}

impl PressureMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PressureMode::Normal   => "NORMAL",
            PressureMode::Saver    => "SAVER",
            PressureMode::Compact  => "COMPACT",
            PressureMode::Boundary => "BOUNDARY",
            PressureMode::Resume   => "RESUME",
        }
    }
}

// ---------------------------------------------------------------------------
// RequestAdvisory
// ---------------------------------------------------------------------------

pub struct RequestAdvisory {
    pub mode: PressureMode,
    pub reason: String,
    pub strategy: Vec<String>,
    pub next_action: String,
    pub next_run_recommended: bool,
}

// ---------------------------------------------------------------------------
// Input signals
// ---------------------------------------------------------------------------

pub struct RequestSignals {
    /// Manually supplied requests used, if known.
    pub used: Option<u64>,
    /// Manually supplied request limit, if known.
    pub limit: Option<u64>,
    /// Prompt to analyse for length (redacted before storage).
    #[allow(dead_code)]
    pub prompt: Option<String>,
}

impl RequestSignals {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        RequestSignals { used: None, limit: None, prompt: None }
    }
}

// ---------------------------------------------------------------------------
// build_advisory
// ---------------------------------------------------------------------------

pub fn build_advisory(cfg: &config::Config, signals: &RequestSignals) -> RequestAdvisory {
    let log_path = cfg.akar_dir.join("EVENT_LOG.jsonl");
    let pm_report = postmortem::run_postmortem(&log_path);
    let event_count = pm_report.total_events;
    let outcome = &pm_report.latest_outcome;
    let has_failures = pm_report.warnings.len() > 0;
    let has_patches = cfg.akar_dir.join("LEARNING_PATCHES.md").exists();

    // Determine pressure level from local signals.
    // Priority: explicit used/limit > postmortem outcome > event count
    let mode = determine_mode(signals, event_count, outcome, has_failures);

    let reason = build_reason(signals, event_count, outcome, has_failures, has_patches);
    let strategy = strategy_for_mode(&mode);
    let next_action = next_action_for_mode(&mode, outcome);
    let next_run_recommended = matches!(mode, PressureMode::Resume);

    RequestAdvisory { mode, reason, strategy, next_action, next_run_recommended }
}

fn determine_mode(
    signals: &RequestSignals,
    event_count: usize,
    outcome: &postmortem::Outcome,
    has_failures: bool,
) -> PressureMode {
    // Explicit usage ratio takes priority when supplied.
    if let (Some(used), Some(limit)) = (signals.used, signals.limit) {
        if limit == 0 {
            return PressureMode::Normal;
        }
        let ratio = used as f64 / limit as f64;
        if ratio >= 0.95 { return PressureMode::Resume; }
        if ratio >= 0.85 { return PressureMode::Boundary; }
        if ratio >= 0.70 { return PressureMode::Compact; }
        if ratio >= 0.50 { return PressureMode::Saver; }
        return PressureMode::Normal;
    }

    // Local signals only.
    if matches!(outcome, postmortem::Outcome::Failed) && has_failures {
        return PressureMode::Boundary;
    }
    if matches!(outcome, postmortem::Outcome::Failed) {
        return PressureMode::Compact;
    }
    if matches!(outcome, postmortem::Outcome::Degraded) {
        return PressureMode::Saver;
    }
    if event_count > 50 {
        return PressureMode::Compact;
    }
    if event_count > 20 {
        return PressureMode::Saver;
    }
    PressureMode::Normal
}

fn build_reason(
    signals: &RequestSignals,
    event_count: usize,
    outcome: &postmortem::Outcome,
    has_failures: bool,
    has_patches: bool,
) -> String {
    if let (Some(used), Some(limit)) = (signals.used, signals.limit) {
        return format!("{}/{} requests used ({:.0}%)", used, limit,
            used as f64 / limit.max(1) as f64 * 100.0);
    }
    let mut parts = Vec::new();
    parts.push(format!("{} telemetry events recorded", event_count));
    match outcome {
        postmortem::Outcome::Clean   => parts.push("last mission clean".to_string()),
        postmortem::Outcome::Degraded => parts.push("last mission degraded".to_string()),
        postmortem::Outcome::Failed   => parts.push("last mission failed".to_string()),
        postmortem::Outcome::Unknown  => parts.push("outcome unknown".to_string()),
    }
    if has_failures { parts.push("warnings detected".to_string()); }
    if has_patches  { parts.push("learning patches exist".to_string()); }
    parts.join(", ")
}

fn strategy_for_mode(mode: &PressureMode) -> Vec<String> {
    match mode {
        PressureMode::Normal => vec![
            "proceed normally".to_string(),
            "use standard context pack".to_string(),
        ],
        PressureMode::Saver => vec![
            "batch file reads".to_string(),
            "avoid repeated status checks".to_string(),
            "skip full roadmap reload".to_string(),
        ],
        PressureMode::Compact => vec![
            "drop cold context (old lessons, archived plans)".to_string(),
            "summarize before continuing".to_string(),
            "use compact context pack only".to_string(),
            "skip non-essential verification".to_string(),
        ],
        PressureMode::Boundary => vec![
            "finish current atomic step only".to_string(),
            "verify minimally".to_string(),
            "do not start new large task".to_string(),
            "write checkpoint before stopping".to_string(),
        ],
        PressureMode::Resume => vec![
            "write NEXT_RUN.md with continuation prompt".to_string(),
            "stop at safe checkpoint".to_string(),
            "do not start any new work".to_string(),
        ],
    }
}

fn next_action_for_mode(mode: &PressureMode, outcome: &postmortem::Outcome) -> String {
    match mode {
        PressureMode::Normal   => "continue with current task".to_string(),
        PressureMode::Saver    => "continue but reduce context reads".to_string(),
        PressureMode::Compact  => {
            if matches!(outcome, postmortem::Outcome::Failed) {
                "run 'akar doctor' then continue with compact context".to_string()
            } else {
                "summarize current state then continue".to_string()
            }
        }
        PressureMode::Boundary => "complete current step, run 'akar verify', then stop".to_string(),
        PressureMode::Resume   => "write .akar/NEXT_RUN.md and stop cleanly".to_string(),
    }
}

// ---------------------------------------------------------------------------
// NEXT_RUN.md proposal
// ---------------------------------------------------------------------------

/// Write a NEXT_RUN.md continuation file if mode is Resume and file doesn't exist.
/// Never overwrites existing file. Returns path written or None.
pub fn write_next_run(cfg: &config::Config, continuation_prompt: &str) -> Option<std::path::PathBuf> {
    if !cfg.akar_dir.exists() {
        return None;
    }
    let path = cfg.akar_dir.join("NEXT_RUN.md");
    if path.exists() {
        return None; // preserve existing
    }
    let ts = event_log::now_iso8601();
    let prompt_preview = config::redact(
        &continuation_prompt.chars().take(200).collect::<String>()
    );
    let content = format!(
        "# NEXT_RUN — Continuation State\n\
        generated: {ts}\n\
        project: {project}\n\n\
        ## Last Known State\n\
        - See `.akar/EVENT_LOG.jsonl` for recent events\n\
        - See `.akar/LEARNING_PATCHES.md` for open patches\n\n\
        ## Recommended Next Command\n\
        ```\n\
        akar mission \"{prompt}\"\n\
        ```\n\n\
        ## Notes\n\
        - Run `akar doctor` first if any issues are suspected\n\
        - Run `akar eval` to verify baseline before continuing\n",
        ts = ts,
        project = cfg.project_name,
        prompt = prompt_preview,
    );
    std::fs::write(&path, content).ok()?;
    Some(path)
}

// ---------------------------------------------------------------------------
// format_advisory
// ---------------------------------------------------------------------------

pub fn format_advisory(advisory: &RequestAdvisory) -> String {
    let mut out = String::new();
    out.push_str(&format!("request: mode={}\n", advisory.mode.as_str()));
    out.push_str(&format!("  reason:  {}\n", advisory.reason));
    out.push_str("  strategy:\n");
    for s in &advisory.strategy {
        out.push_str(&format!("    - {}\n", s));
    }
    out.push_str(&format!("  next:    {}\n", advisory.next_action));
    if advisory.next_run_recommended {
        out.push_str("  hint: run 'akar request' after writing .akar/NEXT_RUN.md\n");
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_cfg() -> config::Config {
        config::Config {
            project_root: std::env::current_dir().unwrap(),
            akar_dir: std::path::PathBuf::from("/nonexistent/__akar_ri_test__"),
            global_dir: std::path::PathBuf::from("/nonexistent/__akar_ri_global__"),
            project_name: "test".to_string(),
        }
    }

    #[test]
    fn normal_mode_under_low_pressure() {
        let cfg = clean_cfg();
        let signals = RequestSignals::empty();
        let advisory = build_advisory(&cfg, &signals);
        assert_eq!(advisory.mode, PressureMode::Normal);
        assert!(!advisory.strategy.is_empty());
    }

    #[test]
    fn saver_mode_at_50_percent() {
        let cfg = clean_cfg();
        let signals = RequestSignals { used: Some(500), limit: Some(1000), prompt: None };
        let advisory = build_advisory(&cfg, &signals);
        assert_eq!(advisory.mode, PressureMode::Saver);
    }

    #[test]
    fn compact_mode_at_70_percent() {
        let cfg = clean_cfg();
        let signals = RequestSignals { used: Some(700), limit: Some(1000), prompt: None };
        let advisory = build_advisory(&cfg, &signals);
        assert_eq!(advisory.mode, PressureMode::Compact);
    }

    #[test]
    fn boundary_mode_at_85_percent() {
        let cfg = clean_cfg();
        let signals = RequestSignals { used: Some(850), limit: Some(1000), prompt: None };
        let advisory = build_advisory(&cfg, &signals);
        assert_eq!(advisory.mode, PressureMode::Boundary);
    }

    #[test]
    fn resume_mode_at_95_percent() {
        let cfg = clean_cfg();
        let signals = RequestSignals { used: Some(950), limit: Some(1000), prompt: None };
        let advisory = build_advisory(&cfg, &signals);
        assert_eq!(advisory.mode, PressureMode::Resume);
        assert!(advisory.next_run_recommended);
    }

    #[test]
    fn recommendations_are_nonempty() {
        let cfg = clean_cfg();
        let advisory = build_advisory(&cfg, &RequestSignals::empty());
        assert!(!advisory.strategy.is_empty());
        assert!(!advisory.next_action.is_empty());
        assert!(!advisory.reason.is_empty());
    }

    #[test]
    fn format_advisory_contains_mode() {
        let cfg = clean_cfg();
        let advisory = build_advisory(&cfg, &RequestSignals::empty());
        let out = format_advisory(&advisory);
        assert!(out.contains("request: mode="));
        assert!(out.contains("strategy:"));
        assert!(out.contains("next:"));
    }

    #[test]
    fn prompt_preview_redacted() {
        let signals = RequestSignals {
            used: None,
            limit: None,
            prompt: Some("token=sk-abc123secret fix the bug".to_string()),
        };
        // Redaction is applied in write_next_run — test the redact fn directly
        let preview = config::redact("token=sk-abc123secret fix the bug");
        assert!(!preview.contains("sk-abc123"));
        assert!(preview.contains("[REDACTED]"));
        let _ = signals;
    }

    #[test]
    fn no_secrets_printed_in_advisory() {
        let cfg = clean_cfg();
        let advisory = build_advisory(&cfg, &RequestSignals::empty());
        let out = format_advisory(&advisory);
        // Ensure no raw secret patterns appear
        assert!(!out.contains("sk-"));
        assert!(!out.contains("password="));
    }
}
