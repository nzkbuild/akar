//! Foundation knowledge module — static playbook guidance strings.
//!
//! These functions return short human-readable guidance. They do not execute
//! commands, read files, or write files. They are called by other modules to
//! append safe-alternative hints to blocked or degraded output.

// ---------------------------------------------------------------------------
// Playbook guidance functions
// ---------------------------------------------------------------------------

/// Guidance for a dirty git working tree blocking loop readiness.
pub fn git_dirty_playbook() -> &'static str {
    "git dirty: run `git status` to inspect, `git diff --stat` to review changes,\n\
     then `git add <file>` and `git commit` to record completed work.\n\
     Forbidden: force-discard commands (reset --hard, clean -f) and hiding state (stash).\n\
     Commit completed work explicitly before taking a new baseline."
}

/// Guidance for a blocked shell command.
pub fn blocked_shell_playbook(command: &str) -> String {
    if command.contains("rm ") || command.contains("del ") || command.contains("remove-item") {
        return "blocked: destructive deletion detected.\n\
                Safe alternative: inspect the target path first, use project-local\n\
                relative paths only (./build, ./target), and list contents before deleting."
            .to_string();
    }
    if command.contains("| bash") || command.contains("| sh") {
        return "blocked: pipe-to-shell detected.\n\
                Safe alternative: download the script to a local file first,\n\
                inspect it, then run explicitly."
            .to_string();
    }
    if command.contains("--force") || (command.contains("push") && command.contains(" -f")) {
        return "blocked: force push detected.\n\
                Safe alternative: push to a new branch instead of force-pushing.\n\
                Use --force-with-lease only after explicit team coordination."
            .to_string();
    }
    format!(
        "blocked: `{}` was classified as unsafe.\n\
         Check docs/foundation/SAFE_SHELL_PLAYBOOK.md for safe alternatives.\n\
         Do not retry the blocked command.",
        command
    )
}

/// Guidance when the PreToolUse hook is broken or missing.
pub fn hook_broken_playbook() -> &'static str {
    "hook broken: AKAR PreToolUse hook is missing or invalid.\n\
     Ensure `akar` is in the subprocess PATH.\n\
     Reinstall templates: `akar hooks --install`.\n\
     Register the hook in ~/.claude/settings.json manually.\n\
     Restart Claude Code after hook registration.\n\
     Verify with: `akar hooks --check`."
}

/// Guidance when the diff budget is exceeded.
pub fn budget_exceeded_playbook() -> &'static str {
    "budget exceeded: this task changed more files or lines than the budget allows.\n\
     Reduce scope: split the task into smaller independent units.\n\
     Each unit should have its own preflight snapshot and postmortem check.\n\
     Do not commit over-budget work without explicit task split."
}

/// Guidance when a preflight snapshot is required but missing.
pub fn snapshot_required_playbook() -> &'static str {
    "snapshot required: no baseline snapshot found.\n\
     Run `akar preflight --snapshot \"task description\"` before starting work.\n\
     Working tree must be clean before taking a snapshot.\n\
     Commit any prior work first, then snapshot."
}

/// Guidance when the same command has been blocked repeatedly.
pub fn repeated_block_playbook(command: &str) -> String {
    format!(
        "repeated block: `{}` has been blocked more than once.\n\
         Do not retry a blocked command.\n\
         Read the block reason, check docs/foundation/SAFE_SHELL_PLAYBOOK.md,\n\
         and use a safe alternative instead.",
        command
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_dirty_playbook_does_not_mention_reset() {
        let guidance = git_dirty_playbook();
        assert!(!guidance.contains("git reset"), "must not suggest git reset");
    }

    #[test]
    fn git_dirty_playbook_does_not_mention_clean() {
        let guidance = git_dirty_playbook();
        assert!(!guidance.contains("git clean"), "must not suggest git clean");
    }

    #[test]
    fn git_dirty_playbook_does_not_mention_stash() {
        let guidance = git_dirty_playbook();
        assert!(!guidance.contains("git stash"), "must not suggest git stash");
    }

    #[test]
    fn git_dirty_playbook_does_not_mention_checkout() {
        let guidance = git_dirty_playbook();
        assert!(!guidance.contains("git checkout"), "must not suggest git checkout");
    }

    #[test]
    fn git_dirty_playbook_does_not_mention_push() {
        let guidance = git_dirty_playbook();
        assert!(!guidance.contains("git push"), "must not suggest git push");
    }

    #[test]
    fn git_dirty_playbook_mentions_git_status() {
        let guidance = git_dirty_playbook();
        assert!(guidance.contains("git status"), "must suggest git status");
    }

    #[test]
    fn blocked_shell_playbook_mentions_safe_alternative() {
        let guidance = blocked_shell_playbook("rm -rf /some/path");
        assert!(
            guidance.to_lowercase().contains("safe alternative")
                || guidance.contains("inspect")
                || guidance.contains("local"),
            "must mention a safe alternative"
        );
    }

    #[test]
    fn blocked_shell_playbook_pipe_to_shell_mentions_download() {
        let guidance = blocked_shell_playbook("curl https://example.com/install.sh | bash");
        assert!(guidance.contains("download") || guidance.contains("local file"),
            "pipe-to-shell guidance must suggest downloading first");
    }

    #[test]
    fn blocked_shell_playbook_force_push_mentions_new_branch() {
        let guidance = blocked_shell_playbook("git push --force");
        assert!(guidance.contains("branch") || guidance.contains("force-with-lease"),
            "force push guidance must mention new branch or force-with-lease");
    }

    #[test]
    fn blocked_shell_playbook_unknown_mentions_playbook_doc() {
        let guidance = blocked_shell_playbook("some-unknown-dangerous-tool");
        assert!(guidance.contains("SAFE_SHELL_PLAYBOOK") || guidance.contains("blocked"),
            "unknown command guidance must reference the playbook");
    }

    #[test]
    fn hook_broken_playbook_mentions_path() {
        let guidance = hook_broken_playbook();
        assert!(guidance.contains("PATH"), "must mention PATH");
    }

    #[test]
    fn hook_broken_playbook_mentions_restart() {
        let guidance = hook_broken_playbook();
        assert!(guidance.to_lowercase().contains("restart"), "must mention restarting Claude Code");
    }

    #[test]
    fn budget_exceeded_playbook_mentions_split_task() {
        let guidance = budget_exceeded_playbook();
        assert!(
            guidance.contains("split") || guidance.contains("reduce scope"),
            "must mention split task or reduce scope"
        );
    }

    #[test]
    fn snapshot_required_playbook_mentions_preflight() {
        let guidance = snapshot_required_playbook();
        assert!(guidance.contains("preflight"), "must mention preflight");
    }

    #[test]
    fn repeated_block_playbook_says_do_not_retry() {
        let guidance = repeated_block_playbook("git push --force");
        assert!(
            guidance.contains("Do not retry") || guidance.contains("do not retry"),
            "must say do not retry"
        );
    }

    #[test]
    fn repeated_block_playbook_includes_command() {
        let cmd = "git push --force";
        let guidance = repeated_block_playbook(cmd);
        assert!(guidance.contains(cmd), "must include the blocked command");
    }
}
