# Safe Shell Playbook

AKAR blocks destructive shell commands at the hook layer. This playbook documents blocked patterns and safe alternatives.

## Blocked Patterns

### Root Filesystem Wipe

The following command patterns are always blocked:

- Unix root wipe: rm with -rf or -fr flag targeting / or /*
- With sudo prefix: same patterns prefixed with sudo
- Windows drive wipe: del /s /q targeting C:\ root
- PowerShell drive wipe: Remove-Item with -Recurse -Force targeting C:\ or /

Why: These commands destroy the entire filesystem. There is no safe version of these commands.

Safe alternative:
1. Inspect the target path first with ls or Get-ChildItem
2. Use project-local relative paths only: ./build, ./target, ./dist
3. For cleanup, list candidates first then delete explicitly by name

### Pipe-to-Shell

The following patterns are always blocked:

- curl ... | bash
- curl ... | sh
- wget ... | bash

Why: Executes arbitrary remote code without inspection.

Safe alternative:
1. Download to a local file first
2. Inspect the script contents before running
3. Run explicitly only after review

### Secret Exposure

Blocked patterns include commands containing: token=, password=, secret=, api_key=, echo + key/token, cat .env

Why: Prevents credential leakage to logs or terminal history.

Safe alternative:
1. Reference credentials by key name only, never by value
2. Use secret management tools or environment variable references
3. Never echo or print credential values

### Force Push

Blocked:
- git push --force
- git push -f

Why: Rewrites remote history and destroys team members' work.

Safe alternative:
1. Use --force-with-lease only after explicit team coordination
2. Create a new branch and push that instead
3. Use git log to understand divergence before any push

## High-Risk Commands (Not Blocked)

These commands are classified High risk but not automatically blocked. Review before running.

### File Deletion in Subdirectory

Pattern: rm -rf ./subdir

Safe practice:
1. List first to confirm target contents
2. Confirm the relative path is correct
3. Run deletion only after confirmation

### Dependency Installation

Pattern: npm install, pip install, cargo add

Safe practice:
1. Review the package name carefully for typosquatting
2. Check package reputation and maintenance status
3. Pin exact versions rather than open ranges

## Dry-Run and Inspection First

Before running destructive commands, use listing or dry-run modes:

- To delete files: list matching files first, then delete by name
- To clean a directory: inspect contents before removing
- To clean untracked git files: run git clean -n (dry-run) before git clean -f
- To install packages: use dry-run flags where available

## Project-Local Paths Only

AKAR playbooks assume project-local operations:
- Prefer relative paths: ./build, ./target, ./node_modules
- Avoid absolute system paths: /usr, /var, /etc, C:\Windows, C:\Program Files
- Operations targeting root or system paths require explicit human justification

## When the Hook Blocks Your Command

If the AKAR PreToolUse hook blocks a command:
1. Read the block reason in the hook output
2. Check this playbook for a safe alternative
3. Do NOT retry the same blocked command
4. Inspect .akar/HOOK_EVENTS.jsonl for block evidence
5. Escalate to human review if the block seems incorrect
