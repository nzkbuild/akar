<#
.SYNOPSIS
AKAR v0.56.1 External Benchmark — One-Shot Runner (B+D Rerun)
Runs only the 2 AKAR-enabled conditions (Haiku + Sonnet) after the hook fix.
Preserves valid A/C controls from v0.56.0.

.EXAMPLE
.\run-bench-bd.ps1
Follow the prompts. This script does NOT call Claude — you copy/paste.

.NOTES
- Does NOT modify global Claude Code config
- Does NOT use dangerous permission modes
- Each condition is isolated in its own directory
- Verifies AKAR hook fired after each session
#>

$ErrorActionPreference = "Stop"
$BenchRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $BenchRoot

$Prompt = @'
Fix the redirect validation bug in this project.
'@.Trim()

$Conditions = @(
  @{Dir="clone-b-haiku-akar";   Model="haiku";  Desc="Haiku 4.5, AKAR enabled"},
  @{Dir="clone-d-sonnet-akar";  Model="sonnet"; Desc="Sonnet 5, AKAR enabled"}
)

Write-Host "==============================================" -ForegroundColor Cyan
Write-Host " AKAR v0.56.1 B+D Rerun Runner" -ForegroundColor Cyan
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This script guides you through 2 Claude Code sessions."
Write-Host "For each condition: open a NEW Claude Code window, cd into the clone"
Write-Host "directory, and paste the prompt."
Write-Host ""
Write-Host "v0.56.1 fix: hookSpecificOutput envelope corrected (was double-nested)."
Write-Host ""

for ($i = 0; $i -lt $Conditions.Count; $i++) {
  $c = $Conditions[$i]
  $clonePath = Join-Path $BenchRoot $c.Dir
  $model = $c.Model
  $desc = $c.Desc

  # --- Reset clone to pristine baseline ---
  Write-Host "[$($i+1)/2] Resetting $desc..." -ForegroundColor Yellow
  Push-Location $clonePath
  git checkout -- . 2>$null
  git clean -fd 2>$null
  Remove-Item -Recurse -Force @(".akar", "node_modules") -ErrorAction SilentlyContinue
  Pop-Location

  # --- Re-init AKAR hooks with fixed v0.56.1 binary ---
  Write-Host "  Re-initializing AKAR hooks with v0.56.1..." -ForegroundColor Yellow
  Push-Location $clonePath
  akar init --claude --hooks --yes 2>&1 | Out-Null
  $hookDiag = akar doctor 2>&1
  Pop-Location

  # --- Verify hook handler format BEFORE the session ---
  Write-Host "  Hook diagnostic:" -ForegroundColor Yellow
  Write-Host "  $hookDiag"
  if ($hookDiag -match "FAIL") {
    Write-Host "  WARNING: Hook diagnostic FAILED — session will be INVALID." -ForegroundColor Red
  }

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
  Write-Host " CONDITION $($i+1)/2: $desc" -ForegroundColor Green
  Write-Host "==============================================" -ForegroundColor Green
  Write-Host ""
  Write-Host "1. Open a NEW Claude Code session (fresh window or terminal)"
  Write-Host "2. cd into: $clonePath"
  Write-Host "3. Model: /model $model"
  Write-Host "   (AKAR hook IS active via .claude/settings.local.json - v0.56.1 fix)"
  Write-Host ""
  Write-Host "4. Copy/paste this EXACT prompt:" -ForegroundColor White
  Write-Host "----------------------------------------" -ForegroundColor White
  Write-Host $Prompt -ForegroundColor White
  Write-Host "----------------------------------------" -ForegroundColor White
  Write-Host ""
  Write-Host "5. Let Claude finish. Then close the session."
  Write-Host "6. Press ENTER in THIS window to capture results..."

  Read-Host

  # --- Verify hook fired ---
  Write-Host "  Verifying hook fired..." -ForegroundColor Yellow
  Push-Location $clonePath
  $hookFired = $false
  $nextRunExists = Test-Path ".akar/NEXT_RUN.md"
  if ($nextRunExists) {
    Write-Host "    [OK] .akar/NEXT_RUN.md exists"
  } else {
    Write-Host "    [MISSING] .akar/NEXT_RUN.md — hook did NOT generate context" -ForegroundColor Red
  }

  # Check HOOK_EVENTS.jsonl for UserPromptSubmit event
  if (Test-Path ".akar/HOOK_EVENTS.jsonl") {
    $upsEvents = Select-String -Path ".akar/HOOK_EVENTS.jsonl" -Pattern "UserPromptSubmit" -SimpleMatch
    if ($upsEvents) {
      Write-Host "    [OK] UserPromptSubmit event recorded in HOOK_EVENTS.jsonl"
      $hookFired = $true
    } else {
      Write-Host "    [MISSING] No UserPromptSubmit event in HOOK_EVENTS.jsonl" -ForegroundColor Red
    }
  } else {
    Write-Host "    [MISSING] .akar/HOOK_EVENTS.jsonl" -ForegroundColor Red
  }

  # Capture results
  Write-Host "  Capturing results..." -ForegroundColor Yellow
  $diff = git diff --stat 2>&1
  $diffDetail = git diff src/validator.js 2>&1
  $s1 = node src/tests.js 2>&1
  $s2 = node src/audit-tests.js 2>&1
  $status = git status -s 2>&1

  $hookStatus = if ($hookFired) { "OBSERVED-FIRED" } else { "NOT-FIRED-INVALID" }

  $resultFile = Join-Path $BenchRoot "$($c.Dir)-v0561-RESULT.txt"
  @"
=== $desc (v0.56.1 fix) ===
Date: $(Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz")
Model: $model
AKAR: enabled (v0.56.1 — hookSpecificOutput fix)
Hook status: $hookStatus
NEXT_RUN.md: $(if ($nextRunExists) { "present" } else { "MISSING" })

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
  if ($hookFired) {
    Write-Host "  Results saved to: $resultFile [VALID]" -ForegroundColor Green
  } else {
    Write-Host "  Results saved to: $resultFile [INVALID — hook did not fire]" -ForegroundColor Red
  }
  Write-Host ""
}

# --- Summary ---
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host " B+D RERUN COMPLETE" -ForegroundColor Cyan
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Results files:"
foreach ($c in $Conditions) {
  $rf = Join-Path $BenchRoot "$($c.Dir)-v0561-RESULT.txt"
  Write-Host "  $rf"
}
Write-Host ""
Write-Host "Send these result files back to Claude Code with A/C results and say:"
Write-Host '  "Score A, B, C, D against SCORING.md using the v0.56.1 B/D reruns"'
Write-Host ""

Pop-Location
