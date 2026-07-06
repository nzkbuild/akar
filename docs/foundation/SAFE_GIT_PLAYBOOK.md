# Safe Git Playbook

AKAR enforces a conservative git posture. These patterns protect the working tree and history.

## Allowed Commands

These commands are read-only or explicitly scoped:

| Command | Purpose |
|---------|---------|
| `git status` | Inspect working tree state |
| `git diff` | Show unstaged changes |
| `git diff --stat` | Summary of changed files and lines |
| `git diff --cached` | Show staged changes |
| `git rev-parse HEAD` | Get current commit hash |
| `git log --oneline -N` | Inspect recent commits |
| `git add <explicit file>` | Stage specific file only |
| `git commit -m "..."` | Commit staged changes after verification |

## Forbidden Commands

These commands are blocked by AKAR default policy:

| Command | Reason |
|---------|--------|
| `git reset` | Discards commits or unstages - data loss risk |
| `git reset --hard` | Destroys uncommitted changes - irreversible |
| `git clean` | Deletes untracked files - irreversible |
| `git stash` | Hides state silently - masking, not resolving |
| `git checkout` | Switches branches or discards changes |
| `git push` | Publishes to remote - shared state mutation |
| `git push --force` | Rewrites remote history - always blocked |
| `git add .` | Broad staging - risks committing secrets or unreviewed changes |
| `git add -A` | Same as above |

## Dirty Tree Situations

When `akar status` shows `readiness: BLOCKED` due to dirty working tree:

### Step 1 - Inspect status
```
git status
```
Identify what files are modified, untracked, or staged.

### Step 2 - Inspect diff
```
git diff
git diff --stat
```
Understand what changed and whether it is expected.

### Step 3 - Verify completed work
- Run `cargo build --release` or equivalent
- Run `cargo test` or equivalent
- Confirm the change is correct and complete

### Step 4 - Commit explicit completed work
```
git add src/specific_file.rs
git add docs/specific_doc.md
git commit -m "feat: description of completed work"
```
Stage only files relevant to the current task.

### Step 5 - Start new baseline only after clean tree
```
git status   # must show clean
akar preflight --snapshot "next task description"
```
Never take a baseline snapshot on a dirty tree.

## Anti-Patterns

- Do not run `git reset` to clean the tree - commit the work or investigate the state
- Do not run `git stash` to temporarily hide state - it creates hidden context
- Do not run `git clean` to remove untracked files - inspect them first
- Do not broad-stage with `git add .` - always name explicit files
- Do not push before local verification passes
