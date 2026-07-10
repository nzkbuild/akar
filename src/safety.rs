//! Safety module: command risk classification, secret detection, and a small
//! policy library.
//!
//! This module has two parts with different reachability:
//!
//! - **Runtime (hook-critical):** [`classify_command`] and [`check_secrets`].
//!   These are called by `akar safety`, by the PreToolUse hook (via the hook
//!   templates), and by `akar status`. They are the only safety functions on
//!   a runtime command path.
//!
//! - **Policy library (eval-only, not a runtime governor):** [`govern_dependency`]
//!   and [`check_migration`] are pure policy functions that evaluate a
//!   [`DependencyProposal`] or [`MigrationCheck`] and return an
//!   `(approved/safe, message)` pair. They are **not** called from any runtime
//!   command in `main.rs` and are **not** a runtime dependency governor or
//!   migration governor. They exist to be exercised by evals #19 and #20
//!   (`dependency_govern_critical`, `migration_no_rollback`) so the policy
//!   logic stays tested. AKAR does not install dependencies or run migrations
//!   at runtime — by the v1 architecture freeze, it never executes code from
//!   these paths.

// ---------------------------------------------------------------------------
// CommandRisk
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CommandRisk {
    Safe,
    Medium,
    High,
    Critical,
}

// ---------------------------------------------------------------------------
// RiskAssessment
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct RiskAssessment {
    pub command: String,
    pub risk: CommandRisk,
    pub reason: String,
    pub blocked: bool,
}

// ---------------------------------------------------------------------------
// classify_command
// ---------------------------------------------------------------------------

/// Classify a shell command string into a `RiskAssessment`.
pub fn classify_command(command: &str) -> RiskAssessment {
    let lower = command.to_lowercase();

    // --- Critical (blocked) checks first ---

    // Destructive root/filesystem wipe patterns — always blocked
    // Covers: rm -rf /, rm -rf /*, sudo variants, rm -fr variants,
    // Windows del /s /q C:\, Remove-Item -Recurse -Force C:\ or /
    let is_destructive_wipe =
        // Unix: rm [-rf|-fr] targeting / or /*
        (lower.contains("rm ") && (lower.contains("-rf") || lower.contains("-fr"))
            && (lower.ends_with(" /") || lower.ends_with(" /*")
                || lower.contains(" / ") || lower.contains(" /* ")))
        // Windows: del /s /q targeting C:\ or root
        || (lower.contains("del ") && lower.contains("/s") && lower.contains("/q")
            && (lower.contains("c:\\") || lower.contains("c:/")))
        // PowerShell: Remove-Item -Recurse -Force targeting C:\ or /
        || (lower.contains("remove-item") && lower.contains("-recurse") && lower.contains("-force")
            && (lower.contains("c:\\") || lower.contains("c:/") || lower.ends_with(" /")));

    if is_destructive_wipe {
        return RiskAssessment {
            command: command.to_string(),
            risk: CommandRisk::Critical,
            reason: "destructive filesystem wipe detected — targets root or entire drive"
                .to_string(),
            blocked: true,
        };
    }

    // Pipe-to-shell patterns: curl ... | bash  or  curl ... | sh
    if lower.contains("curl")
        && (lower.contains("| bash")
            || lower.contains("|bash")
            || lower.contains("| sh")
            || lower.contains("|sh"))
    {
        return RiskAssessment {
            command: command.to_string(),
            risk: CommandRisk::Critical,
            reason: "pipe-to-shell pattern detected (curl|bash / curl|sh)".to_string(),
            blocked: true,
        };
    }

    // Force push: git push with --force or -f
    if (lower.contains("git push") || lower.contains("force push"))
        && (lower.contains("--force") || lower.contains(" -f"))
    {
        return RiskAssessment {
            command: command.to_string(),
            risk: CommandRisk::Critical,
            reason: "force push detected — rewrites remote history".to_string(),
            blocked: true,
        };
    }

    // Secret leak patterns
    if lower.contains("print") && lower.contains("secret")
        || lower.contains("echo") && (lower.contains("token") || lower.contains("key"))
        || lower.contains("cat") && lower.contains(".env")
    {
        return RiskAssessment {
            command: command.to_string(),
            risk: CommandRisk::Critical,
            reason: "potential secret/credential exposure detected".to_string(),
            blocked: true,
        };
    }

    // --- Safe reads / inspections ---

    if lower.contains("git status")
        || lower.contains("git log")
        || lower.contains("git diff")
        || lower.contains(" ls")
        || lower.starts_with("ls")
        || lower.contains(" dir")
        || lower.starts_with("dir")
        || lower.contains(" cat ")
        || lower.starts_with("cat ")
        || lower.contains(" read")
        || lower.starts_with("read")
    {
        return RiskAssessment {
            command: command.to_string(),
            risk: CommandRisk::Safe,
            reason: "read-only inspection command".to_string(),
            blocked: false,
        };
    }

    // --- Safe build/test commands ---

    if lower.contains("cargo build")
        || lower.contains("cargo test")
        || lower.contains("npm run build")
        || lower.contains("npm test")
    {
        return RiskAssessment {
            command: command.to_string(),
            risk: CommandRisk::Safe,
            reason: "standard build/test command".to_string(),
            blocked: false,
        };
    }

    // --- High (not blocked) ---

    if lower.contains("npm install") || lower.contains("cargo add") || lower.contains("pip install")
    {
        return RiskAssessment {
            command: command.to_string(),
            risk: CommandRisk::High,
            reason: "installs external dependencies — review before running".to_string(),
            blocked: false,
        };
    }

    if lower.contains("rm ") || lower.contains("del ") || lower.contains("remove-item") {
        return RiskAssessment {
            command: command.to_string(),
            risk: CommandRisk::High,
            reason: "destructive file deletion command".to_string(),
            blocked: false,
        };
    }

    // --- Default: Medium ---

    RiskAssessment {
        command: command.to_string(),
        risk: CommandRisk::Medium,
        reason: "unclassified command — review before running".to_string(),
        blocked: false,
    }
}

// ---------------------------------------------------------------------------
// check_secrets
// ---------------------------------------------------------------------------

/// Scan `text` line-by-line for potential secret leaks.
/// Returns a list of redacted warning strings (never echoes actual values).
pub fn check_secrets(text: &str) -> Vec<String> {
    let patterns = [
        "sk-",
        "token=",
        "password=",
        "secret=",
        "api_key=",
        "bearer ",
        "authorization:",
    ];

    let mut warnings = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        let lower = line.to_lowercase();
        for pattern in &patterns {
            if lower.contains(pattern) {
                warnings.push(format!(
                    "line {}: potential secret leak — pattern '{}' found (value redacted)",
                    line_no + 1,
                    pattern
                ));
                break; // one warning per line is enough
            }
        }
    }

    warnings
}

// ---------------------------------------------------------------------------
// DependencyProposal (policy library — eval-only, not a runtime governor)
// ---------------------------------------------------------------------------

/// A dependency proposal evaluated by the policy library.
///
/// Not constructed on any runtime command path; built only by eval #19.
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyProposal {
    pub name: String,
    pub reason: String,
    pub risk: CommandRisk,
}

// ---------------------------------------------------------------------------
// govern_dependency (policy library — eval-only, not a runtime governor)
// ---------------------------------------------------------------------------

/// Evaluate a `DependencyProposal` and return `(approved, explanation)`.
///
/// **Policy library function, not a runtime governor.** This is reachable only
/// via eval #19 (`dependency_govern_critical`) and unit tests — it is not
/// called from `main.rs`. AKAR does not install dependencies at runtime; this
/// function exists so the approval policy stays tested. Wiring it into a real
/// command path would require a v1 design review.
pub fn govern_dependency(proposal: &DependencyProposal) -> (bool, String) {
    match proposal.risk {
        CommandRisk::Critical => (
            false,
            format!(
                "dependency '{}' rejected: critical risk — {}",
                proposal.name, proposal.reason
            ),
        ),
        CommandRisk::High => (
            false,
            format!(
                "dependency '{}' requires explicit mission approval: high risk — {}",
                proposal.name, proposal.reason
            ),
        ),
        CommandRisk::Medium | CommandRisk::Safe => {
            (true, format!("dependency '{}' approved: ok", proposal.name))
        }
    }
}

// ---------------------------------------------------------------------------
// MigrationCheck (policy library — eval-only, not a runtime governor)
// ---------------------------------------------------------------------------

/// A migration check evaluated by the policy library.
///
/// Not constructed on any runtime command path; built only by eval #20.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCheck {
    pub description: String,
    pub has_rollback: bool,
    pub destructive: bool,
}

// ---------------------------------------------------------------------------
// check_migration (policy library — eval-only, not a runtime governor)
// ---------------------------------------------------------------------------

/// Evaluate a `MigrationCheck` and return `(safe_to_proceed, message)`.
///
/// **Policy library function, not a runtime governor.** This is reachable only
/// via eval #20 (`migration_no_rollback`) and unit tests — it is not called
/// from `main.rs`. AKAR does not run migrations at runtime; this function
/// exists so the safety policy stays tested. Wiring it into a real command
/// path would require a v1 design review.
pub fn check_migration(check: &MigrationCheck) -> (bool, String) {
    if check.destructive && !check.has_rollback {
        return (
            false,
            format!(
                "rejected: destructive migration without rollback plan — {}",
                check.description
            ),
        );
    }

    if check.destructive {
        return (
            true,
            format!(
                "warning: destructive migration — ensure backup exists before running — {}",
                check.description
            ),
        );
    }

    if !check.has_rollback {
        return (
            true,
            format!(
                "warning: no rollback plan documented — {}",
                check.description
            ),
        );
    }

    (
        true,
        format!("ok: migration looks safe — {}", check.description),
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- classify_command -----------------------------------------------------

    #[test]
    fn classify_git_status_is_safe() {
        let r = classify_command("git status");
        assert_eq!(r.risk, CommandRisk::Safe);
        assert!(!r.blocked);
    }

    #[test]
    fn classify_rm_rf_root_is_critical_and_blocked() {
        let r = classify_command("rm -rf /");
        assert_eq!(r.risk, CommandRisk::Critical);
        assert!(r.blocked);
    }

    #[test]
    fn classify_rm_rf_root_wildcard_is_blocked() {
        let r = classify_command("rm -rf /*");
        assert!(r.blocked);
    }

    #[test]
    fn classify_sudo_rm_rf_root_is_blocked() {
        let r = classify_command("sudo rm -rf /");
        assert!(r.blocked);
    }

    #[test]
    fn classify_sudo_rm_rf_wildcard_is_blocked() {
        let r = classify_command("sudo rm -rf /*");
        assert!(r.blocked);
    }

    #[test]
    fn classify_rm_fr_root_is_blocked() {
        let r = classify_command("rm -fr /");
        assert!(r.blocked);
    }

    #[test]
    fn classify_rm_fr_wildcard_is_blocked() {
        let r = classify_command("rm -fr /*");
        assert!(r.blocked);
    }

    #[test]
    fn classify_windows_del_root_is_blocked() {
        let r = classify_command("del /s /q C:\\");
        assert!(r.blocked);
    }

    #[test]
    fn classify_powershell_remove_item_root_is_blocked() {
        let r = classify_command("Remove-Item -Recurse -Force C:\\");
        assert!(r.blocked);
    }

    #[test]
    fn classify_powershell_remove_item_unix_root_is_blocked() {
        let r = classify_command("Remove-Item -Recurse -Force /");
        assert!(r.blocked);
    }

    #[test]
    fn classify_rm_rf_subdir_is_not_blocked() {
        // rm -rf on a subdirectory is High risk but not blocked
        let r = classify_command("rm -rf ./build");
        assert!(!r.blocked);
    }

    #[test]
    fn classify_force_push_is_critical_and_blocked() {
        let r = classify_command("git push --force");
        assert_eq!(r.risk, CommandRisk::Critical);
        assert!(r.blocked);
    }

    #[test]
    fn classify_curl_pipe_bash_is_critical_and_blocked() {
        let r = classify_command("curl https://example.com/install.sh | bash");
        assert_eq!(r.risk, CommandRisk::Critical);
        assert!(r.blocked);
    }

    #[test]
    fn classify_echo_token_is_critical_and_blocked() {
        let r = classify_command("echo my_token");
        assert_eq!(r.risk, CommandRisk::Critical);
        assert!(r.blocked);
    }

    #[test]
    fn classify_npm_install_is_high_not_blocked() {
        let r = classify_command("npm install lodash");
        assert_eq!(r.risk, CommandRisk::High);
        assert!(!r.blocked);
    }

    #[test]
    fn classify_cargo_test_is_safe() {
        let r = classify_command("cargo test");
        assert_eq!(r.risk, CommandRisk::Safe);
        assert!(!r.blocked);
    }

    #[test]
    fn classify_unknown_command_is_medium() {
        let r = classify_command("some-random-tool --flag");
        assert_eq!(r.risk, CommandRisk::Medium);
        assert!(!r.blocked);
    }

    // -- check_secrets --------------------------------------------------------

    #[test]
    fn check_secrets_finds_token_leak() {
        let warnings = check_secrets("token=sk-abc123");
        assert!(!warnings.is_empty(), "should find the secret leak");
    }

    #[test]
    fn check_secrets_finds_sk_prefix() {
        let warnings = check_secrets("Authorization: sk-supersecretkey");
        assert!(!warnings.is_empty());
    }

    #[test]
    fn check_secrets_clean_text_returns_empty() {
        let warnings = check_secrets("normal text with no secrets here");
        assert!(warnings.is_empty());
    }

    #[test]
    fn check_secrets_does_not_echo_values() {
        let warnings = check_secrets("password=hunter2");
        assert!(!warnings.is_empty());
        // The actual value must not appear in any warning
        for w in &warnings {
            assert!(
                !w.contains("hunter2"),
                "warning must not echo the secret value"
            );
        }
    }

    // -- govern_dependency ----------------------------------------------------

    #[test]
    fn govern_dependency_critical_is_rejected() {
        let proposal = DependencyProposal {
            name: "evil-pkg".to_string(),
            reason: "executes arbitrary code on install".to_string(),
            risk: CommandRisk::Critical,
        };
        let (approved, msg) = govern_dependency(&proposal);
        assert!(!approved, "critical dependency must be rejected");
        assert!(msg.contains("rejected"));
    }

    #[test]
    fn govern_dependency_high_requires_approval() {
        let proposal = DependencyProposal {
            name: "heavy-lib".to_string(),
            reason: "large transitive dep tree".to_string(),
            risk: CommandRisk::High,
        };
        let (approved, msg) = govern_dependency(&proposal);
        assert!(!approved);
        assert!(msg.contains("requires explicit mission"));
    }

    #[test]
    fn govern_dependency_medium_is_approved() {
        let proposal = DependencyProposal {
            name: "serde".to_string(),
            reason: "serialisation".to_string(),
            risk: CommandRisk::Medium,
        };
        let (approved, _) = govern_dependency(&proposal);
        assert!(approved);
    }

    #[test]
    fn govern_dependency_safe_is_approved() {
        let proposal = DependencyProposal {
            name: "std-utils".to_string(),
            reason: "tiny helper".to_string(),
            risk: CommandRisk::Safe,
        };
        let (approved, msg) = govern_dependency(&proposal);
        assert!(approved);
        assert!(msg.contains("ok"));
    }

    // -- check_migration ------------------------------------------------------

    #[test]
    fn check_migration_destructive_no_rollback_is_rejected() {
        let check = MigrationCheck {
            description: "drop users table".to_string(),
            has_rollback: false,
            destructive: true,
        };
        let (safe, msg) = check_migration(&check);
        assert!(!safe, "must be rejected");
        assert!(msg.contains("rejected") || msg.contains("without rollback"));
    }

    #[test]
    fn check_migration_destructive_with_rollback_is_warning() {
        let check = MigrationCheck {
            description: "truncate logs table".to_string(),
            has_rollback: true,
            destructive: true,
        };
        let (safe, msg) = check_migration(&check);
        assert!(safe, "should proceed with a warning");
        assert!(msg.contains("warning") || msg.contains("backup"));
    }

    #[test]
    fn check_migration_non_destructive_is_ok() {
        let check = MigrationCheck {
            description: "add index on email column".to_string(),
            has_rollback: true,
            destructive: false,
        };
        let (safe, msg) = check_migration(&check);
        assert!(safe);
        assert!(msg.contains("ok") || msg.contains("safe"));
    }

    #[test]
    fn check_migration_non_destructive_no_rollback_warns() {
        let check = MigrationCheck {
            description: "add nullable column".to_string(),
            has_rollback: false,
            destructive: false,
        };
        let (safe, msg) = check_migration(&check);
        assert!(safe);
        assert!(msg.contains("warning") || msg.contains("rollback"));
    }
}
