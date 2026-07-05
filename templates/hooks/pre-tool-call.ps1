# AKAR pre-tool-call hook (PowerShell)
#
# Usage: called by Claude Code before each tool execution.
# Claude Code passes the command string as the first argument.
# This script calls `akar safety` and exits non-zero only for BLOCKED commands.
#
# Install: copy to your project's hook location and register in
# ~/.claude/settings.json under hooks.preToolCall. AKAR does not install
# this automatically.

param(
    [Parameter(Position=0)]
    [string]$Command = ""
)

if (-not $Command) {
    # Nothing to check — allow
    exit 0
}

# Check akar is available
if (-not (Get-Command akar -ErrorAction SilentlyContinue)) {
    Write-Warning "akar: not found in PATH — skipping safety check"
    exit 0
}

# Run akar safety check
$output = akar safety $Command 2>&1
$exitCode = $LASTEXITCODE

Write-Output $output

# akar safety exits 2 for BLOCKED commands
if ($exitCode -eq 2) {
    Write-Error "akar: command blocked — $Command"
    exit 1
}

exit 0
