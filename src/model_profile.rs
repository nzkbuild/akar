//! Model/Gateway drift detection and calibration.
//!
//! Tracks which model and gateway are in use, detects drift between sessions,
//! and maintains per-model capability profiles.

use std::process::Command;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Capability profile for a specific model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelProfile {
    pub model_id: String,
    pub gateway: String,
    pub observed_strengths: Vec<String>,
    pub observed_weaknesses: Vec<String>,
    /// Preferred task granularity: "micro" | "small" | "medium" | "large"
    pub best_task_size: String,
    /// Autonomy ceiling: "A0" through "A6"
    pub autonomy_limit: String,
    /// Preferred output verbosity: "concise" | "detailed"
    pub output_style: String,
    /// How aggressively to verify outputs: "normal" | "strict"
    pub verification_strictness: String,
    pub known_failure_patterns: Vec<String>,
    /// ISO 8601 date of last calibration, e.g. "2026-07-04"
    pub last_calibrated: String,
}

/// Snapshot of session identity used for drift comparison.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SessionFingerprint {
    pub started_at: String,
    pub project_id: String,
    pub git_root: String,
    pub branch: String,
    pub cwd: String,
    pub model: String,
    pub gateway: String,
    pub base_url: String,
}

// ---------------------------------------------------------------------------
// Model detection
// ---------------------------------------------------------------------------

/// Detect the active model and gateway from environment variables.
///
/// Checks (in order): `ANTHROPIC_MODEL`, `CLAUDE_MODEL`, `OPENAI_MODEL`.
/// Returns `("unknown", "unknown")` when none are set.
pub fn detect_model() -> (String, String) {
    if let Ok(m) = std::env::var("ANTHROPIC_MODEL") {
        if !m.is_empty() {
            return (m, "anthropic".to_string());
        }
    }
    if let Ok(m) = std::env::var("CLAUDE_MODEL") {
        if !m.is_empty() {
            return (m, "claude".to_string());
        }
    }
    if let Ok(m) = std::env::var("OPENAI_MODEL") {
        if !m.is_empty() {
            return (m, "openai".to_string());
        }
    }
    ("unknown".to_string(), "unknown".to_string())
}

// ---------------------------------------------------------------------------
// Profile construction
// ---------------------------------------------------------------------------

/// Build a reasonable default `ModelProfile` for `model_id`.
///
/// Applies heuristics based on well-known model name fragments; unknown
/// models get conservative defaults.
pub fn default_profile(model_id: &str) -> ModelProfile {
    let id = model_id.to_lowercase();

    // Capability heuristics based on model family.
    let (strengths, weaknesses, task_size, autonomy, style, strictness) =
        if id.contains("opus") {
            (
                vec![
                    "complex reasoning".to_string(),
                    "long-context synthesis".to_string(),
                    "multi-step planning".to_string(),
                ],
                vec![
                    "speed on micro tasks".to_string(),
                    "cost efficiency".to_string(),
                ],
                "large".to_string(),
                "A4".to_string(),
                "detailed".to_string(),
                "normal".to_string(),
            )
        } else if id.contains("sonnet") {
            (
                vec![
                    "balanced speed and quality".to_string(),
                    "code generation".to_string(),
                    "instruction following".to_string(),
                ],
                vec!["very long document synthesis".to_string()],
                "medium".to_string(),
                "A3".to_string(),
                "concise".to_string(),
                "normal".to_string(),
            )
        } else if id.contains("haiku") {
            (
                vec![
                    "fast responses".to_string(),
                    "simple classification".to_string(),
                    "low-latency pipelines".to_string(),
                ],
                vec![
                    "deep reasoning".to_string(),
                    "large context tasks".to_string(),
                ],
                "micro".to_string(),
                "A2".to_string(),
                "concise".to_string(),
                "strict".to_string(),
            )
        } else if id.contains("gpt-4") {
            (
                vec![
                    "instruction following".to_string(),
                    "structured output".to_string(),
                ],
                vec!["very long reasoning chains".to_string()],
                "medium".to_string(),
                "A3".to_string(),
                "concise".to_string(),
                "normal".to_string(),
            )
        } else {
            // Conservative defaults for unknown models.
            (
                vec!["general text generation".to_string()],
                vec!["unknown — profile not calibrated".to_string()],
                "small".to_string(),
                "A2".to_string(),
                "concise".to_string(),
                "strict".to_string(),
            )
        };

    ModelProfile {
        model_id: model_id.to_string(),
        gateway: "unknown".to_string(),
        observed_strengths: strengths,
        observed_weaknesses: weaknesses,
        best_task_size: task_size,
        autonomy_limit: autonomy,
        output_style: style,
        verification_strictness: strictness,
        known_failure_patterns: Vec::new(),
        last_calibrated: "never".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Drift detection
// ---------------------------------------------------------------------------

/// Compare two `SessionFingerprint`s and return human-readable drift warnings.
///
/// Returns an empty `Vec` when the fingerprints represent the same session context.
#[allow(dead_code)]
pub fn detect_drift(old: &SessionFingerprint, new: &SessionFingerprint) -> Vec<String> {
    let mut warnings = Vec::new();

    if old.model != new.model {
        warnings.push(format!(
            "model changed: {} -> {}",
            old.model, new.model
        ));
    }

    if old.gateway != new.gateway {
        warnings.push(format!(
            "gateway changed: {} -> {}",
            old.gateway, new.gateway
        ));
    }

    if old.branch != new.branch {
        warnings.push(format!(
            "branch changed: {} -> {}",
            old.branch, new.branch
        ));
    }

    if old.project_id != new.project_id {
        warnings.push(format!(
            "project changed: {} -> {}",
            old.project_id, new.project_id
        ));
    }

    warnings
}

// ---------------------------------------------------------------------------
// Profile formatting
// ---------------------------------------------------------------------------

/// Format a `ModelProfile` as a concise multi-line string.
pub fn format_profile(profile: &ModelProfile) -> String {
    let mut out = String::new();

    out.push_str(&format!("model profile: {}\n", profile.model_id));
    out.push_str(&format!("  gateway:              {}\n", profile.gateway));
    out.push_str(&format!("  best_task_size:       {}\n", profile.best_task_size));
    out.push_str(&format!("  autonomy_limit:       {}\n", profile.autonomy_limit));
    out.push_str(&format!("  output_style:         {}\n", profile.output_style));
    out.push_str(&format!("  verification:         {}\n", profile.verification_strictness));
    out.push_str(&format!("  last_calibrated:      {}\n", profile.last_calibrated));

    if !profile.observed_strengths.is_empty() {
        out.push_str("  strengths:\n");
        for s in &profile.observed_strengths {
            out.push_str(&format!("    - {}\n", s));
        }
    }

    if !profile.observed_weaknesses.is_empty() {
        out.push_str("  weaknesses:\n");
        for w in &profile.observed_weaknesses {
            out.push_str(&format!("    - {}\n", w));
        }
    }

    if !profile.known_failure_patterns.is_empty() {
        out.push_str("  failure_patterns:\n");
        for fp in &profile.known_failure_patterns {
            out.push_str(&format!("    - {}\n", fp));
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Calibration
// ---------------------------------------------------------------------------

/// Update a profile given a prompt and an observed response quality.
///
/// When `response_quality` is `"poor"`, the prompt is recorded as a known
/// failure pattern. Otherwise the profile is returned unchanged (future
/// phases may implement positive calibration).
#[allow(dead_code)]
pub fn calibrate_from_prompt(prompt: &str, response_quality: &str) -> ModelProfile {
    let (model_id, gateway) = detect_model();
    let mut profile = default_profile(&model_id);
    profile.gateway = gateway;

    if response_quality == "poor" {
        profile
            .known_failure_patterns
            .push(format!("poor response on: {}", prompt));
    }

    profile
}

// ---------------------------------------------------------------------------
// Git helpers
// ---------------------------------------------------------------------------

/// Return the current git branch name, or `"unknown"` if git is unavailable
/// or the directory is not a repository.
#[allow(dead_code)]
pub fn detect_git_branch() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

/// Return the git repository root, or `"unknown"` if unavailable.
#[allow(dead_code)]
pub fn detect_git_root() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- default_profile ------------------------------------------------------

    #[test]
    fn test_default_profile_produces_valid_profile() {
        let profile = default_profile("claude-opus-4");
        assert_eq!(profile.model_id, "claude-opus-4");
        assert!(!profile.best_task_size.is_empty());
        assert!(!profile.autonomy_limit.is_empty());
        assert!(!profile.output_style.is_empty());
        assert!(!profile.verification_strictness.is_empty());
        assert!(!profile.observed_strengths.is_empty());
    }

    #[test]
    fn test_default_profile_unknown_model_is_conservative() {
        let profile = default_profile("some-mystery-model-v99");
        assert_eq!(profile.autonomy_limit, "A2");
        assert_eq!(profile.verification_strictness, "strict");
    }

    // -- detect_drift ---------------------------------------------------------

    fn make_fingerprint(model: &str, gateway: &str, branch: &str, project: &str) -> SessionFingerprint {
        SessionFingerprint {
            started_at: "2026-07-04T00:00:00Z".to_string(),
            project_id: project.to_string(),
            git_root: "/repo".to_string(),
            branch: branch.to_string(),
            cwd: "/repo".to_string(),
            model: model.to_string(),
            gateway: gateway.to_string(),
            base_url: "https://api.anthropic.com".to_string(),
        }
    }

    #[test]
    fn test_detect_drift_catches_model_change() {
        let old = make_fingerprint("claude-opus-4", "anthropic", "main", "akar");
        let new = make_fingerprint("claude-sonnet-4", "anthropic", "main", "akar");
        let warnings = detect_drift(&old, &new);
        assert!(
            warnings.iter().any(|w| w.contains("model changed")),
            "expected model drift warning, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_detect_drift_catches_branch_change() {
        let old = make_fingerprint("claude-opus-4", "anthropic", "main", "akar");
        let new = make_fingerprint("claude-opus-4", "anthropic", "feature/phase-14", "akar");
        let warnings = detect_drift(&old, &new);
        assert!(
            warnings.iter().any(|w| w.contains("branch changed")),
            "expected branch drift warning, got: {:?}",
            warnings
        );
    }

    #[test]
    fn test_detect_drift_returns_empty_when_same() {
        let fp = make_fingerprint("claude-opus-4", "anthropic", "main", "akar");
        let warnings = detect_drift(&fp, &fp);
        assert!(
            warnings.is_empty(),
            "expected no drift warnings, got: {:?}",
            warnings
        );
    }

    // -- format_profile -------------------------------------------------------

    #[test]
    fn test_format_profile_returns_non_empty() {
        let profile = default_profile("claude-haiku-3");
        let output = format_profile(&profile);
        assert!(!output.is_empty());
        assert!(output.contains("claude-haiku-3"));
        assert!(output.contains("autonomy_limit"));
    }

    // -- calibrate_from_prompt ------------------------------------------------

    #[test]
    fn test_calibrate_from_prompt_poor_quality_adds_failure_pattern() {
        // Temporarily unset model env vars so we get a deterministic result.
        // (We just verify the failure pattern is added regardless of model.)
        let profile = calibrate_from_prompt("summarise this 500-page doc in one sentence", "poor");
        assert!(
            !profile.known_failure_patterns.is_empty(),
            "expected at least one failure pattern"
        );
        assert!(
            profile.known_failure_patterns[0].contains("poor response on"),
            "failure pattern should contain the prompt context"
        );
    }

    #[test]
    fn test_calibrate_from_prompt_good_quality_no_failure_pattern() {
        let profile = calibrate_from_prompt("list files in /tmp", "good");
        assert!(
            profile.known_failure_patterns.is_empty(),
            "good quality should not add failure patterns"
        );
    }
}
