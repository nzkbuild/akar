//! Phase 7: Task Contract Engine
//!
//! Classifies a user prompt into a structured `TaskContract` that captures
//! intent, autonomy level, cost mode, risk, diff budget, and verification hooks.

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TaskType {
    Answer,
    Inspect,
    Bugfix,
    Frontend,
    Feature,
    Refactor,
    Research,
    Security,
    Greenfield,
    Repair,
    Migration,
    Dependency,
    Release,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Autonomy {
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum CostMode {
    Fast,
    Balanced,
    Deep,
    Autopilot,
    Emergency,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

// ---------------------------------------------------------------------------
// DiffBudget
// ---------------------------------------------------------------------------

/// The four diff-budget tiers' `(files_max, loc_max)` caps.
///
/// These are the **single source of truth** for budget cap numbers. Both the
/// contract classifier (`DiffBudget::micro/small/medium/large`) and the
/// user-facing task-name resolver (`diff_budget::budget_for_task_name`) read
/// from these constants so the two paths cannot silently diverge. The v0.21
/// audit (§7c.1) found a second hardcoded table in `diff_budget.rs` despite a
/// "no second budget table" comment; centralizing the caps here makes that
/// comment true.
pub const BUDGET_CAP_MICRO: (usize, usize) = (3, 60);
pub const BUDGET_CAP_SMALL: (usize, usize) = (5, 200);
pub const BUDGET_CAP_MEDIUM: (usize, usize) = (12, 600);
pub const BUDGET_CAP_LARGE: (usize, usize) = (30, 2000);

#[derive(Debug, Clone, PartialEq)]
pub struct DiffBudget {
    pub files_min: usize,
    pub files_max: usize,
    pub loc_min: usize,
    pub loc_max: usize,
    pub new_files_allowed: bool,
    pub dependencies_allowed: bool,
    pub migrations_allowed: bool,
}

impl DiffBudget {
    /// Micro budget — tiny targeted fixes (e.g. bugfix).
    fn micro() -> Self {
        let (files_max, loc_max) = BUDGET_CAP_MICRO;
        DiffBudget {
            files_min: 1,
            files_max,
            loc_min: 5,
            loc_max,
            new_files_allowed: false,
            dependencies_allowed: false,
            migrations_allowed: false,
        }
    }

    /// Small budget — focused changes touching a handful of files.
    fn small() -> Self {
        let (files_max, loc_max) = BUDGET_CAP_SMALL;
        DiffBudget {
            files_min: 1,
            files_max,
            loc_min: 10,
            loc_max,
            new_files_allowed: false,
            dependencies_allowed: false,
            migrations_allowed: false,
        }
    }

    /// Medium budget — moderate feature or refactor work.
    fn medium() -> Self {
        let (files_max, loc_max) = BUDGET_CAP_MEDIUM;
        DiffBudget {
            files_min: 2,
            files_max,
            loc_min: 30,
            loc_max,
            new_files_allowed: true,
            dependencies_allowed: false,
            migrations_allowed: false,
        }
    }

    /// Large budget — significant cross-cutting work.
    fn large() -> Self {
        let (files_max, loc_max) = BUDGET_CAP_LARGE;
        DiffBudget {
            files_min: 3,
            files_max,
            loc_min: 50,
            loc_max,
            new_files_allowed: true,
            dependencies_allowed: true,
            migrations_allowed: true,
        }
    }
}

// ---------------------------------------------------------------------------
// TaskContract
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct TaskContract {
    pub user_intent: String,
    pub inferred_goal: String,
    pub task_type: TaskType,
    pub autonomy: Autonomy,
    pub cost_mode: CostMode,
    pub risk_level: RiskLevel,
    pub confidence: Confidence,
    pub diff_budget: DiffBudget,
    pub stop_conditions: Vec<String>,
    pub verification_commands: Vec<String>,
    pub memory_update_allowed: bool,
}

// ---------------------------------------------------------------------------
// classify_prompt
// ---------------------------------------------------------------------------

/// Classify a free-form user prompt into a `TaskContract` using keyword rules.
pub fn classify_prompt(prompt: &str) -> TaskContract {
    let lower = prompt.to_lowercase();

    // Determine task type by priority order (more specific wins first).
    // Security keywords take precedence over generic "add"/"feature" words.
    let (task_type, risk_level, diff_budget, stop_conditions) = if lower.contains("security")
        || lower.contains("auth")
        || lower.contains("password")
        || lower.contains("token")
    {
        (
            TaskType::Security,
            RiskLevel::High,
            DiffBudget::small(),
            vec![
                "no secrets committed".to_string(),
                "auth flow still passes".to_string(),
                "no regressions in permission checks".to_string(),
            ],
        )
    } else if lower.contains("migrate") || lower.contains("migration") || lower.contains("schema") {
        (
            TaskType::Migration,
            RiskLevel::High,
            DiffBudget::large(),
            vec![
                "migration is reversible".to_string(),
                "data integrity verified".to_string(),
                "rollback plan documented".to_string(),
            ],
        )
    } else if lower.contains("dependency") || lower.contains("package") || lower.contains("install")
    {
        (
            TaskType::Dependency,
            RiskLevel::Medium,
            DiffBudget::small(),
            vec![
                "no breaking version conflicts".to_string(),
                "lockfile updated".to_string(),
            ],
        )
    } else if lower.contains("fix")
        || lower.contains("bug")
        || lower.contains("error")
        || lower.contains("broken")
    {
        (
            TaskType::Bugfix,
            RiskLevel::Low,
            DiffBudget::micro(),
            vec![
                "original symptom no longer reproducible".to_string(),
                "tests pass".to_string(),
            ],
        )
    } else if lower.contains("ui")
        || lower.contains("frontend")
        || lower.contains("design")
        || lower.contains("style")
        || lower.contains("css")
    {
        (
            TaskType::Frontend,
            RiskLevel::Low,
            DiffBudget::medium(),
            vec![
                "visual regression check done".to_string(),
                "responsive layout verified".to_string(),
            ],
        )
    } else if lower.contains("refactor") || lower.contains("clean") || lower.contains("restructure")
    {
        (
            TaskType::Refactor,
            RiskLevel::Medium,
            DiffBudget::medium(),
            vec![
                "behaviour unchanged".to_string(),
                "tests pass".to_string(),
                "no public API changes".to_string(),
            ],
        )
    } else if lower.contains("add")
        || lower.contains("feature")
        || lower.contains("implement")
        || lower.contains("create")
    {
        (
            TaskType::Feature,
            RiskLevel::Low,
            DiffBudget::medium(),
            vec![
                "feature works end-to-end".to_string(),
                "tests added".to_string(),
            ],
        )
    } else {
        // Default
        (
            TaskType::Feature,
            RiskLevel::Low,
            DiffBudget::medium(),
            vec![
                "feature works end-to-end".to_string(),
                "tests added".to_string(),
            ],
        )
    };

    let inferred_goal = infer_goal(&task_type, prompt);
    let verification_commands = default_verification(&task_type);

    TaskContract {
        user_intent: prompt.to_string(),
        inferred_goal,
        task_type,
        autonomy: Autonomy::A5,
        cost_mode: CostMode::Balanced,
        risk_level,
        confidence: Confidence::Medium,
        diff_budget,
        stop_conditions,
        verification_commands,
        memory_update_allowed: true,
    }
}

fn infer_goal(task_type: &TaskType, prompt: &str) -> String {
    let prefix = match task_type {
        TaskType::Bugfix => "Fix the defect: ",
        TaskType::Frontend => "Implement UI change: ",
        TaskType::Feature => "Deliver feature: ",
        TaskType::Refactor => "Refactor without behaviour change: ",
        TaskType::Security => "Harden security: ",
        TaskType::Migration => "Execute migration: ",
        TaskType::Dependency => "Update dependency: ",
        TaskType::Research => "Research and summarise: ",
        TaskType::Answer => "Answer: ",
        TaskType::Inspect => "Inspect and report: ",
        TaskType::Greenfield => "Build from scratch: ",
        TaskType::Repair => "Repair broken state: ",
        TaskType::Release => "Cut release: ",
    };
    format!("{}{}", prefix, prompt)
}

fn default_verification(task_type: &TaskType) -> Vec<String> {
    match task_type {
        TaskType::Bugfix => vec!["cargo test".to_string()],
        TaskType::Frontend => vec!["cargo build".to_string()],
        TaskType::Feature => vec!["cargo test".to_string(), "cargo build".to_string()],
        TaskType::Refactor => vec!["cargo test".to_string(), "cargo clippy".to_string()],
        TaskType::Security => vec!["cargo test".to_string(), "cargo audit".to_string()],
        TaskType::Migration => vec!["cargo test".to_string()],
        TaskType::Dependency => vec!["cargo build".to_string(), "cargo test".to_string()],
        _ => vec!["cargo build".to_string()],
    }
}

// ---------------------------------------------------------------------------
// format_contract
// ---------------------------------------------------------------------------

/// Produce a concise human-readable summary of a `TaskContract`.
pub fn format_contract(contract: &TaskContract) -> String {
    let task_type = format!("{:?}", contract.task_type);
    let autonomy = format!("{:?}", contract.autonomy);
    let cost_mode = format!("{:?}", contract.cost_mode);
    let risk = format!("{:?}", contract.risk_level);
    let confidence = format!("{:?}", contract.confidence);

    let budget = &contract.diff_budget;
    let budget_str = format!(
        "{}-{} files, {}-{} LOC{}{}{}",
        budget.files_min,
        budget.files_max,
        budget.loc_min,
        budget.loc_max,
        if budget.new_files_allowed {
            ", new-files:yes"
        } else {
            ""
        },
        if budget.dependencies_allowed {
            ", deps:yes"
        } else {
            ""
        },
        if budget.migrations_allowed {
            ", migrations:yes"
        } else {
            ""
        },
    );

    let stop = contract.stop_conditions.join("; ");
    let verify = contract.verification_commands.join(", ");

    format!(
        "contract:\n  intent:    {}\n  goal:      {}\n  type:      {}  autonomy: {}  cost: {}  risk: {}  confidence: {}\n  budget:    {}\n  stop:      {}\n  verify:    {}\n  memory:    {}",
        contract.user_intent,
        contract.inferred_goal,
        task_type,
        autonomy,
        cost_mode,
        risk,
        confidence,
        budget_str,
        stop,
        verify,
        if contract.memory_update_allowed {
            "allowed"
        } else {
            "locked"
        },
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_bugfix_prompt() {
        let c = classify_prompt("fix the login button");
        assert_eq!(c.task_type, TaskType::Bugfix);
    }

    #[test]
    fn classify_feature_prompt() {
        let c = classify_prompt("add dark mode");
        assert_eq!(c.task_type, TaskType::Feature);
    }

    #[test]
    fn classify_auth_prompt_is_security() {
        // "auth" triggers Security, overriding the generic "refactor" keyword
        let c = classify_prompt("refactor auth module");
        assert_eq!(c.task_type, TaskType::Security);
    }

    #[test]
    fn classify_migration_prompt() {
        let c = classify_prompt("migrate user schema");
        assert_eq!(c.task_type, TaskType::Migration);
    }

    #[test]
    fn classify_frontend_prompt() {
        let c = classify_prompt("update the css styles");
        assert_eq!(c.task_type, TaskType::Frontend);
    }

    #[test]
    fn classify_refactor_prompt() {
        let c = classify_prompt("clean up the parser module");
        assert_eq!(c.task_type, TaskType::Refactor);
    }

    #[test]
    fn classify_dependency_prompt() {
        let c = classify_prompt("install the serde package");
        assert_eq!(c.task_type, TaskType::Dependency);
    }

    // -- Diff budget assertions -----------------------------------------------

    #[test]
    fn bugfix_has_micro_budget() {
        let c = classify_prompt("fix the crash on startup");
        assert_eq!(c.diff_budget.files_max, 3);
        assert_eq!(c.diff_budget.loc_max, 60);
        assert!(!c.diff_budget.new_files_allowed);
    }

    #[test]
    fn feature_has_medium_budget() {
        let c = classify_prompt("add a new export feature");
        assert_eq!(c.diff_budget.files_max, 12);
        assert!(c.diff_budget.new_files_allowed);
    }

    #[test]
    fn migration_has_large_budget_with_migrations_allowed() {
        let c = classify_prompt("migrate the user schema to v2");
        assert!(c.diff_budget.migrations_allowed);
        assert!(c.diff_budget.dependencies_allowed);
    }

    #[test]
    fn security_has_small_budget() {
        let c = classify_prompt("fix the auth token validation");
        assert_eq!(c.diff_budget.files_max, 5);
        assert!(!c.diff_budget.new_files_allowed);
    }

    // -- Default fields -------------------------------------------------------

    #[test]
    fn default_autonomy_is_a5() {
        let c = classify_prompt("do something");
        assert_eq!(c.autonomy, Autonomy::A5);
    }

    #[test]
    fn default_cost_mode_is_balanced() {
        let c = classify_prompt("do something");
        assert_eq!(c.cost_mode, CostMode::Balanced);
    }

    #[test]
    fn stop_conditions_nonempty_for_all_types() {
        let prompts = [
            "fix the bug",
            "add dark mode",
            "refactor auth module",
            "migrate user schema",
            "install serde package",
            "update css style",
        ];
        for p in &prompts {
            let c = classify_prompt(p);
            assert!(
                !c.stop_conditions.is_empty(),
                "stop_conditions empty for prompt: {}",
                p
            );
        }
    }

    // -- format_contract ------------------------------------------------------

    #[test]
    fn format_contract_is_nonempty() {
        let c = classify_prompt("fix the login button");
        let s = format_contract(&c);
        assert!(!s.is_empty());
    }

    #[test]
    fn format_contract_contains_key_fields() {
        let c = classify_prompt("fix the login button");
        let s = format_contract(&c);
        assert!(s.contains("Bugfix"), "should mention task type");
        assert!(
            s.contains("fix the login button"),
            "should include user intent"
        );
        assert!(s.contains("A5"), "should mention autonomy level");
    }
}
