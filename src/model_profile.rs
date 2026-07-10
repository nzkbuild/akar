//! Model/Gateway profile — detects active model from environment variables
//! and returns capability heuristics for display purposes only.
//!
//! Does not call any model API. Does not persist state.

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

    let (strengths, weaknesses, task_size, autonomy, style, strictness) = if id.contains("opus") {
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
// Profile formatting
// ---------------------------------------------------------------------------

/// Format a `ModelProfile` as a concise multi-line string.
pub fn format_profile(profile: &ModelProfile) -> String {
    let mut out = String::new();

    out.push_str(&format!("model profile: {}\n", profile.model_id));
    out.push_str(&format!("  gateway:              {}\n", profile.gateway));
    out.push_str(&format!(
        "  best_task_size:       {}\n",
        profile.best_task_size
    ));
    out.push_str(&format!(
        "  autonomy_limit:       {}\n",
        profile.autonomy_limit
    ));
    out.push_str(&format!(
        "  output_style:         {}\n",
        profile.output_style
    ));
    out.push_str(&format!(
        "  verification:         {}\n",
        profile.verification_strictness
    ));
    out.push_str(&format!(
        "  last_calibrated:      {}\n",
        profile.last_calibrated
    ));

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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_format_profile_returns_non_empty() {
        let profile = default_profile("claude-haiku-3");
        let output = format_profile(&profile);
        assert!(!output.is_empty());
        assert!(output.contains("claude-haiku-3"));
        assert!(output.contains("autonomy_limit"));
    }

    #[test]
    fn detect_model_returns_unknown_when_no_env() {
        // Only asserts it doesn't panic and returns non-empty strings.
        let (model, gateway) = detect_model();
        assert!(!model.is_empty());
        assert!(!gateway.is_empty());
    }
}
