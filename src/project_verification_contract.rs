//! Project-aware verification contract (v0.30.0 → v0.31.0 → v0.38.0).
//!
//! Build, test, verification, and NEXT_RUN data keyed by ProjectKind.
//! Discovery of extra verification hints is delegated to
//! `verification_discovery` for Unknown and known project kinds.
//!
//! Detection is delegated to the shared `project_detection` module — this
//! module contains no marker-file logic of its own.

use std::path::Path;

// Re-export for callers that import through this module.
pub use crate::project_detection::ProjectKind;

/// Detect the project kind from marker files in `project_root`.
///
/// Delegates to the shared `project_detection` module.
pub fn detect_project_kind(project_root: &Path) -> ProjectKind {
    crate::project_detection::detect_project_kind(project_root)
}

// ---------------------------------------------------------------------------
// Discovery hints
// ---------------------------------------------------------------------------

pub use crate::verification_discovery::{VerificationConfidence, discover_verification_hints};

// ---------------------------------------------------------------------------
// Build commands
// ---------------------------------------------------------------------------

pub fn build_commands(kind: ProjectKind) -> Vec<&'static str> {
    match kind {
        ProjectKind::Rust => vec!["cargo build --release"],
        _ => vec![],
    }
}

// ---------------------------------------------------------------------------
// Test commands
// ---------------------------------------------------------------------------

pub fn test_commands(kind: ProjectKind) -> Vec<&'static str> {
    match kind {
        ProjectKind::Rust => vec!["cargo test"],
        ProjectKind::Node => vec!["npm test"],
        ProjectKind::Python => vec!["python -m pytest"],
        ProjectKind::Unknown => vec![],
    }
}

// ---------------------------------------------------------------------------
// AKAR CLI prefix
// ---------------------------------------------------------------------------

/// The prefix for invoking AKAR CLI commands. Rust projects use `cargo run --`
/// (the repo's own build); non-Rust projects use `akar` (the PATH binary).
pub fn akar_prefix(kind: ProjectKind) -> &'static str {
    match kind {
        ProjectKind::Rust => "cargo run --",
        _ => "akar",
    }
}

/// AKAR CLI commands that are always allowed, using the project-appropriate prefix.
pub fn akar_cli_commands(kind: ProjectKind) -> Vec<String> {
    let p = akar_prefix(kind);
    vec![
        format!("{} --version", p),
        format!("{} status", p),
        format!("{} governor --json --no-exit-code", p),
        format!("{} doctor", p),
        format!("{} eval", p),
        format!("{} hooks --check", p),
    ]
}

// ---------------------------------------------------------------------------
// Project-specific NEXT_RUN additions
// ---------------------------------------------------------------------------

/// Additional allowed commands specific to the project kind.
///
/// For known project kinds includes the built-in commands.  For Unknown
/// projects includes high-confidence discovered hints (requires_confirmation=false).
pub fn project_allowed_commands(kind: ProjectKind) -> Vec<String> {
    let mut cmds: Vec<String> = Vec::new();
    for c in build_commands(kind) {
        cmds.push(c.to_string());
    }
    for c in test_commands(kind) {
        cmds.push(c.to_string());
    }
    cmds
}

/// Discovered verification hints that can be added as additional allowed
/// commands (high-confidence only, from local project files).
///
/// This does NOT run any discovered command — it only returns hints that
/// were found by reading safe local files.
#[allow(dead_code)]
pub fn discovered_allowed_commands(root: &Path, kind: ProjectKind) -> Vec<String> {
    let discovery = crate::verification_discovery::discover_verification_hints(root);
    let mut cmds: Vec<String> = Vec::new();
    for hint in &discovery.hints {
        // Only high-confidence hints with no confirmation requirement
        // are added as allowed commands for known project kinds. For
        // Unknown projects, even high-confidence hints require the user
        // to confirm before running.
        if hint.confidence == crate::verification_discovery::VerificationConfidence::High
            && !hint.requires_confirmation
        {
            // For known project kinds, the built-in commands already cover
            // these — skip duplicates.
            let built_in = project_allowed_commands(kind);
            if !built_in.contains(&hint.command) {
                cmds.push(hint.command.clone());
            }
        }
    }
    cmds
}

/// Additional verification commands specific to the project kind.
pub fn project_verification_commands(kind: ProjectKind) -> Vec<String> {
    project_allowed_commands(kind)
}

/// Project-specific stop conditions.
pub fn project_stop_conditions(kind: ProjectKind) -> Vec<String> {
    match kind {
        ProjectKind::Rust => vec!["Stop if `cargo test` fails.".to_string()],
        ProjectKind::Node => vec!["Stop if `npm test` fails.".to_string()],
        ProjectKind::Python => vec!["Stop if `python -m pytest` fails.".to_string()],
        ProjectKind::Unknown => vec![
            "Stop if verification fails (run the project's documented verification command)."
                .to_string(),
        ],
    }
}

// ---------------------------------------------------------------------------
// Unknown-project guidance (for Verification Required section)
// ---------------------------------------------------------------------------

pub fn unknown_verification_guidance() -> Vec<&'static str> {
    vec![
        "Run the project's documented verification command.",
        "Inspect README or project scripts before choosing a test command.",
    ]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- build commands ----------------------------------------------------

    #[test]
    fn rust_build_commands_include_cargo_build() {
        let cmds = build_commands(ProjectKind::Rust);
        assert!(cmds.contains(&"cargo build --release"));
    }

    #[test]
    fn node_build_commands_are_empty() {
        assert!(build_commands(ProjectKind::Node).is_empty());
    }

    // ---- test commands -----------------------------------------------------

    #[test]
    fn rust_test_commands_include_cargo_test() {
        let cmds = test_commands(ProjectKind::Rust);
        assert!(cmds.contains(&"cargo test"));
    }

    #[test]
    fn node_test_commands_include_npm_test() {
        let cmds = test_commands(ProjectKind::Node);
        assert!(cmds.contains(&"npm test"));
    }

    #[test]
    fn python_test_commands_include_pytest() {
        let cmds = test_commands(ProjectKind::Python);
        assert!(cmds.contains(&"python -m pytest"));
    }

    #[test]
    fn unknown_test_commands_are_empty() {
        assert!(test_commands(ProjectKind::Unknown).is_empty());
    }

    // ---- akar prefix -------------------------------------------------------

    #[test]
    fn rust_akar_prefix_is_cargo_run() {
        assert_eq!(akar_prefix(ProjectKind::Rust), "cargo run --");
    }

    #[test]
    fn non_rust_akar_prefix_is_akar() {
        assert_eq!(akar_prefix(ProjectKind::Node), "akar");
        assert_eq!(akar_prefix(ProjectKind::Python), "akar");
        assert_eq!(akar_prefix(ProjectKind::Unknown), "akar");
    }

    // ---- project allowed commands ------------------------------------------

    #[test]
    fn rust_allowed_includes_cargo_build_and_test() {
        let cmds = project_allowed_commands(ProjectKind::Rust);
        assert!(cmds.contains(&"cargo build --release".to_string()));
        assert!(cmds.contains(&"cargo test".to_string()));
    }

    #[test]
    fn node_allowed_includes_npm_test_only() {
        let cmds = project_allowed_commands(ProjectKind::Node);
        assert!(cmds.contains(&"npm test".to_string()));
        assert!(!cmds.contains(&"cargo build --release".to_string()));
        assert!(!cmds.contains(&"cargo test".to_string()));
    }

    #[test]
    fn python_allowed_includes_pytest_only() {
        let cmds = project_allowed_commands(ProjectKind::Python);
        assert!(cmds.contains(&"python -m pytest".to_string()));
        assert!(!cmds.contains(&"cargo build --release".to_string()));
        assert!(!cmds.contains(&"cargo test".to_string()));
    }

    #[test]
    fn unknown_allowed_is_empty() {
        assert!(project_allowed_commands(ProjectKind::Unknown).is_empty());
    }

    // ---- stop conditions ---------------------------------------------------

    #[test]
    fn rust_stop_condition_mentions_cargo_test() {
        let conds = project_stop_conditions(ProjectKind::Rust);
        assert!(conds.iter().any(|c| c.contains("cargo test")));
    }

    #[test]
    fn node_stop_condition_mentions_npm_test() {
        let conds = project_stop_conditions(ProjectKind::Node);
        assert!(conds.iter().any(|c| c.contains("npm test")));
    }

    #[test]
    fn python_stop_condition_mentions_pytest() {
        let conds = project_stop_conditions(ProjectKind::Python);
        assert!(conds.iter().any(|c| c.contains("python -m pytest")));
    }

    #[test]
    fn unknown_stop_condition_mentions_documented_verification() {
        let conds = project_stop_conditions(ProjectKind::Unknown);
        assert!(conds.iter().any(|c| c.contains("documented verification")));
    }

    // ---- unknown guidance --------------------------------------------------

    #[test]
    fn unknown_guidance_mentions_documented_command() {
        let g = unknown_verification_guidance();
        assert!(g.iter().any(|s| s.contains("documented verification")));
    }
}
