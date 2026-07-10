<#
.SYNOPSIS
AKAR v0.56 External Benchmark — One-Shot Runner
Runs all 4 conditions (Haiku/Sonnet × AKAR enabled/disabled) in the current
Claude Code session. User pastes one prompt per condition.

.EXAMPLE
.\run-bench.ps1
Follow the prompts. This script does NOT call Claude — you copy/paste.

.NOTES
- Does NOT modify global Claude Code config
- Does NOT use dangerous permission modes
- Each condition is isolated in its own directory
- Studio OAuth is used (no API keys needed)
#>

$ErrorActionPreference = "Stop"
$BenchRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $BenchRoot

$Prompt = @'
Fix the redirect validation bug in this project.
'@.Trim()

$Conditions = @(
  @{Dir="clone-a-haiku-noakar"; Model="haiku";  Akar="disabled"; Desc="Haiku 4.5, no AKAR"},
  @{Dir="clone-b-haiku-akar";   Model="haiku";  Akar="enabled";  Desc="Haiku 4.5, AKAR enabled"},
  @{Dir="clone-c-sonnet-noakar"; Model="sonnet"; Akar="disabled"; Desc="Sonnet 5, no AKAR"},
  @{Dir="clone-d-sonnet-akar";  Model="sonnet"; Akar="enabled";  Desc="Sonnet 5, AKAR enabled"}
)

Write-Host "==============================================" -ForegroundColor Cyan
Write-Host " AKAR v0.56 External Benchmark Runner" -ForegroundColor Cyan
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This script guides you through 4 Claude Code sessions."
Write-Host "For each condition: open a NEW Claude Code window (or use --print-mode"
Write-Host "if that works on your system), cd into the clone directory, and paste"
Write-Host "the prompt."
Write-Host ""
Write-Host "IMPORTANT: After each session finishes, press Enter in THIS window."
Write-Host "The script will capture results and move to the next condition."
Write-Host ""

for ($i = 0; $i -lt $Conditions.Count; $i++) {
  $c = $Conditions[$i]
  $clonePath = Join-Path $BenchRoot $c.Dir
  $model = $c.Model
  $desc = $c.Desc

  # --- Reset clone to pristine baseline ---
  Write-Host "[$($i+1)/4] Resetting $desc..." -ForegroundColor Yellow
  Push-Location $clonePath
  git checkout -- . 2>$null
  git clean -fd 2>$null
  rm -rf .akar node_modules 2>$null
  Pop-Location

  # --- Pre-flight: verify tests pass ---
  Push-Location $clonePath
  $stage1 = node src/tests.js 2>&1
  $stage2 = node src/audit-tests.js 2>&1
  Pop-Location
  Write-Host "  Baseline Stage 1: $stage1"
  Write-Host "  Baseline Stage 2: $stage2"

  # --- Launch instructions ---
  Write-Host ""
  Write-Host "==============================================" -ForegroundColor Green
  Write-Host " CONDITION $($i+1)/4: $desc" -ForegroundColor Green
  Write-Host "==============================================" -ForegroundColor Green
  Write-Host ""
  Write-Host "1. Open a NEW Claude Code session (file > new window, or 'claude' in terminal)"
  Write-Host "2. cd into: $clonePath"
  if ($c.Akar -eq "enabled") {
    Write-Host "3. Model: /model $model   (AKAR hook IS active via .claude/settings.local.json)"
  } else {
    Write-Host "3. Model: /model $model   (NO AKAR hook — .claude/ absent)"
  }
  Write-Host ""
  Write-Host "4. Copy/paste this EXACT prompt:" -ForegroundColor White
  Write-Host "----------------------------------------" -ForegroundColor White
  Write-Host $Prompt -ForegroundColor White
  Write-Host "----------------------------------------" -ForegroundColor White
  Write-Host ""
  Write-Host "5. Let Claude finish. Then close the session."
  Write-Host "6. Press ENTER in THIS window to capture results..."

  Read-Host

  # --- Capture results ---
  Write-Host "  Capturing results..." -ForegroundColor Yellow
  Push-Location $clonePath

  $diff = git diff --stat 2>&1
  $diffDetail = git diff src/validator.js 2>&1
  $s1 = node src/tests.js 2>&1
  $s2 = node src/audit-tests.js 2>&1
  $status = git status -s 2>&1

  $resultFile = Join-Path $BenchRoot "$($c.Dir)-RESULT.txt"
  @"
=== $desc ===
Date: $(Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz")
Model: $model
AKAR: $($c.Akar)

--- DIFF STAT ---
$diff

--- VALIDATOR DIFF ---
$diffDetail

--- STAGE 1 ---
$s1

--- STAGE 2 ---
$s2

--- GIT STATUS ---
$status
"@ | Out-File -FilePath $resultFile -Encoding utf8

  Pop-Location
  Write-Host "  Results saved to: $resultFile" -ForegroundColor Green
  Write-Host ""
}

# --- Summary ---
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host " ALL 4 CONDITIONS COMPLETE" -ForegroundColor Cyan
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Results files:"
foreach ($c in $Conditions) {
  $rf = Join-Path $BenchRoot "$($c.Dir)-RESULT.txt"
  Write-Host "  $rf"
}
Write-Host ""
Write-Host "Send these 4 result files back to Claude Code and say:"
Write-Host '  "Read all 4 RESULT.txt files and score them against docs/audits/benchmarks/SCORING.md"'
Write-Host ""

Pop-Location
