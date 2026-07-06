# AKAR pre-tool-call hook (PowerShell)
#
# Called by Claude Code before each tool execution via PreToolUse hook.
# Claude Code sends a JSON object via stdin with tool_name, tool_input,
# and a top-level "cwd" field (the working directory when the hook fired).
#
# Exit codes:
#   0 — allow (safe or non-Bash tool)
#   2 — block (AKAR classified command as BLOCKED)
#   exit 1 does NOT block in Claude Code — always use exit 2 to block.
#
# Hook events are written to .akar/HOOK_EVENTS.jsonl for local audit.
# AKAR does not send hook telemetry anywhere.
#
# v0.29.0: log root is chosen from the Claude Code "cwd" field (the target
# project root) when present, falling back to the hook process cwd.  Each
# event line includes a "log_root" field so the target project is explicit.
#
# Install: register in ~/.claude/settings.json under hooks.PreToolUse.
# AKAR does not install this automatically.
#
# Example settings.json entry:
#   "PreToolUse": [
#     {
#       "matcher": "Bash",
#       "hooks": [
#         {
#           "type": "command",
#           "command": "pwsh \"C:\\path\\to\\akar\\templates\\hooks\\pre-tool-call.ps1\""
#         }
#       ]
#     }
#   ]

# Read JSON from stdin
$json = $input | Out-String
if (-not $json.Trim()) {
    exit 0
}

# ---------- v0.29.0: choose log root from Claude Code "cwd" field ----------
# Priority: explicit cwd from hook JSON > process cwd as fallback.
$logRoot = (Get-Location).Path
$cwdFromJson = ""
if ($json -match '"cwd"\s*:\s*"([^"]+)"') {
    $cwdFromJson = $Matches[1] -replace '\\\\', '\'
}
if ($cwdFromJson -and (Test-Path $cwdFromJson -PathType Container)) {
    $logRoot = $cwdFromJson
}
$logRootEsc = $logRoot -replace '\\', '\\' -replace '"', '\"'
# --------------------------------------------------------------------------

# Helper: write a JSONL event line to .akar/HOOK_EVENTS.jsonl under $logRoot
function Write-HookEvent {
    param($ToolName, $CommandPreview, $Decision, $ExitCode)
    $timestamp = (Get-Date -Format 'o')
    $esc = $CommandPreview -replace '\\', '\\' -replace '"', '\"' -replace "`n", '\n' -replace "`t", '\t'
    $line = "{`"timestamp`":`"$timestamp`",`"hook`":`"PreToolUse`",`"tool_name`":`"$ToolName`",`"command_preview`":`"$esc`",`"decision`":`"$Decision`",`"exit_code`":$ExitCode,`"log_root`":`"$logRootEsc`"}"
    $akarDir = Join-Path $logRoot ".akar"
    if (-not (Test-Path $akarDir)) { New-Item -ItemType Directory -Force -Path $akarDir | Out-Null }
    Add-Content -Path (Join-Path $akarDir "HOOK_EVENTS.jsonl") -Value $line
}

# Extract tool_name
$toolName = ""
if ($json -match '"tool_name"\s*:\s*"([^"]+)"') {
    $toolName = $Matches[1]
}

# Only check Bash tool calls
if ($toolName -ne "Bash") {
    Write-Output "akar hook: tool=$toolName — skipping (not Bash)"
    Write-HookEvent -ToolName $toolName -CommandPreview "" -Decision "SKIP" -ExitCode 0
    exit 0
}

# Extract tool_input.command
$command = ""
if ($json -match '"command"\s*:\s*"((?:[^"\\]|\\.)*)"') {
    $command = $Matches[1] -replace '\\n', "`n" -replace '\\t', "`t" -replace '\\"', '"' -replace '\\\\', '\'
}

if (-not $command.Trim()) {
    Write-Output "akar hook: Bash tool with no command — allowing"
    Write-HookEvent -ToolName "Bash" -CommandPreview "" -Decision "ALLOW" -ExitCode 0
    exit 0
}

# Redact obvious secrets and truncate preview to 300 chars
$preview = $command -replace '(?i)(password|token|secret|key|api_key|apikey|bearer)\s*[=:]\s*\S+', '$1=[REDACTED]'
if ($preview.Length -gt 300) { $preview = $preview.Substring(0, 300) }

# Check akar is available
if (-not (Get-Command akar -ErrorAction SilentlyContinue)) {
    Write-Warning "akar: not found in PATH — skipping safety check"
    Write-HookEvent -ToolName "Bash" -CommandPreview $preview -Decision "ALLOW" -ExitCode 0
    exit 0
}

# Run akar safety check
$output = akar safety $command 2>&1
$exitCode = $LASTEXITCODE

Write-Output $output

# Determine decision label
$decision = if ($exitCode -eq 2) { "BLOCK" } elseif ($output -match "WARN|High|Critical") { "WARN" } else { "ALLOW" }
$hookExitCode = if ($exitCode -eq 2) { 2 } else { 0 }

Write-HookEvent -ToolName "Bash" -CommandPreview $preview -Decision $decision -ExitCode $hookExitCode

# Exit 2 blocks in Claude Code. Exit 1 does NOT block.
if ($exitCode -eq 2) {
    Write-Error "akar hook: BLOCKED — $command"
    exit 2
}

exit 0
