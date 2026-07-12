<#
.SYNOPSIS
AKAR v0.56.2 External Benchmark — Full 4-Condition Runner
Runs all 4 conditions with the redesigned fixture oracle and model identity validation.

.EXAMPLE
.\run-bench.ps1
Follow the prompts. This script does NOT call Claude — you copy/paste.

.NOTES
- Does NOT modify global Claude Code config
- Does NOT use dangerous permission modes
- Each condition is isolated in its own directory
- Verifies fixture FAILS on baseline before each session
- Validates model identity before each session
- Verifies AKAR hook fired after AKAR-enabled sessions
- v0.56.2: redesigned fixture oracle, model identity validation
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
Write-Host " AKAR v0.56.2 External Benchmark Runner" -ForegroundColor Cyan
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Redesigned fixture: Stage 1 and Stage 2 MUST FAIL on baseline." -ForegroundColor Yellow
Write-Host "After correct fix, both stages PASS." -ForegroundColor Yellow
Write-Host ""
Write-Host "This script guides you through 4 Claude Code sessions."
Write-Host "For each condition: open a NEW Claude Code window, cd into the clone"
Write-Host "directory, and paste the prompt."
Write-Host ""

for ($i = 0; $i -lt $Conditions.Count; $i++) {
  $c = $Conditions[$i]
  $clonePath = Join-Path $BenchRoot $c.Dir
  $model = $c.Model
  $desc = $c.Desc
  $akarEnabled = $c.Akar -eq "enabled"

  # --- Reset clone to pristine baseline ---
  Write-Host "[$($i+1)/4] Resetting $desc..." -ForegroundColor Yellow
  Push-Location $clonePath
  git checkout -- . 2>$null
  git clean -fd 2>$null
  Remove-Item -Recurse -Force @(".akar", "node_modules") -ErrorAction SilentlyContinue
  Pop-Location

  # --- Re-init AKAR hooks for AKAR-enabled conditions ---
  if ($akarEnabled) {
    Write-Host "  Re-initializing AKAR hooks..." -ForegroundColor Yellow
    Push-Location $clonePath
    akar init --claude --hooks --yes 2>&1 | Out-Null
    $hookDiag = akar doctor 2>&1
    Pop-Location
    Write-Host "  Hook diagnostic: $hookDiag"
    if ($hookDiag -match "FAIL") {
      Write-Host "  WARNING: Hook diagnostic FAILED — session will be INVALID." -ForegroundColor Red
    }
  }

  # --- Pre-flight: verify fixture FAILS on baseline ---
  Push-Location $clonePath
  $stage1Pre = node src/tests.js 2>&1
  $s1Exit = $LASTEXITCODE
  $stage2Pre = node src/audit-tests.js 2>&1
  $s2Exit = $LASTEXITCODE
  Pop-Location
  Write-Host "  Baseline Stage 1 (must FAIL): $stage1Pre"
  Write-Host "  Baseline Stage 2 (must FAIL): $stage2Pre"
  if ($s1Exit -eq 0) {
    Write-Host "  ERROR: Stage 1 PASSES on baseline — fixture not propagated. Aborting." -ForegroundColor Red
    exit 1
  }
  if ($s2Exit -eq 0) {
    Write-Host "  ERROR: Stage 2 PASSES on baseline — fixture not propagated. Aborting." -ForegroundColor Red
    exit 1
  }

  # --- Model identity pre-flight ---
  Write-Host ""
  Write-Host "==============================================" -ForegroundColor Green
  Write-Host " CONDITION $($i+1)/4: $desc" -ForegroundColor Green
  Write-Host "==============================================" -ForegroundColor Green
  Write-Host ""
  Write-Host "1. Open a NEW Claude Code session (fresh window or terminal)"
  Write-Host "2. cd into: $clonePath"
  Write-Host "3. REQUIRED: /model $model"
  Write-Host "   VERIFY the response says 'Set model to' the CORRECT model for this condition."
  Write-Host "   If it says anything else, CLOSE the session and start over."
  Write-Host ""
  if ($akarEnabled) {
    Write-Host "   AKAR hook IS active via .claude/settings.local.json (v0.56.1 fix)" -ForegroundColor Yellow
    Write-Host "   If AKAR context does NOT appear after the prompt, the hook failed."
    Write-Host "   Mark the session INVALID and re-run."
  } else {
    Write-Host "   NO AKAR hook — .claude/ absent" -ForegroundColor DarkGray
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

  # --- Model identity post-flight ---
  $modelOk = Read-Host "  Did the session use model '$model'? (y/n)"
  if ($modelOk -ne "y") {
    Write-Host "  SESSION INVALID — wrong model. Results marked accordingly." -ForegroundColor Red
    $modelValid = $false
  } else {
    $modelValid = $true
  }

  # --- Verify hook fired (AKAR-enabled conditions only) ---
  Push-Location $clonePath
  $hookFired = $false
  if ($akarEnabled) {
    $nextRunExists = Test-Path ".akar/NEXT_RUN.md"
    if ($nextRunExists) {
      Write-Host "    [OK] .akar/NEXT_RUN.md exists"
    } else {
      Write-Host "    [MISSING] .akar/NEXT_RUN.md — hook did NOT generate context" -ForegroundColor Red
    }
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
  }

  # --- Capture results ---
  Write-Host "  Capturing results..." -ForegroundColor Yellow
  $diff = git diff --stat 2>&1
  $diffDetail = git diff src/validator.js 2>&1
  $s1 = node src/tests.js 2>&1
  $s1Exit = $LASTEXITCODE
  $s2 = node src/audit-tests.js 2>&1
  $s2Exit = $LASTEXITCODE
  $status = git status -s 2>&1

  $validity = "VALID"
  if ($akarEnabled -and -not $hookFired) { $validity = "INVALID (hook not fired)" }
  if (-not $modelValid) { $validity = "INVALID (wrong model)" }

  $resultFile = Join-Path $BenchRoot "$($c.Dir)-v0562-RESULT.txt"
  @"
=== $desc (v0.56.2) ===
Date: $(Get-Date -Format "yyyy-MM-ddTHH:mm:sszzz")
Model: $model
Model validated: $modelValid
AKAR: $($c.Akar)
Hook fired: $(if ($akarEnabled) { $hookFired } else { "n/a" })
Validity: $validity

--- DIFF STAT ---
$diff

--- VALIDATOR DIFF ---
$diffDetail

--- STAGE 1 (functional) ---
$s1
exit code: $s1Exit

--- STAGE 2 (audit) ---
$s2
exit code: $s2Exit

--- GIT STATUS ---
$status
"@ | Out-File -FilePath $resultFile -Encoding utf8

  Pop-Location
  if ($validity -eq "VALID") {
    Write-Host "  Results saved to: $resultFile [$validity]" -ForegroundColor Green
  } else {
    Write-Host "  Results saved to: $resultFile [$validity]" -ForegroundColor Red
  }
  Write-Host ""
}

# --- Summary ---
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host " ALL 4 CONDITIONS COMPLETE" -ForegroundColor Cyan
Write-Host "==============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "v0.56.2 results (new fixture oracle):"
foreach ($c in $Conditions) {
  $rf = Join-Path $BenchRoot "$($c.Dir)-v0562-RESULT.txt"
  Write-Host "  $rf"
}
Write-Host ""
Write-Host "To score: send these 4 result files and say:"
Write-Host '  "Read all 4 v0.56.2 RESULT.txt files and score them against SCORING.md"'
Write-Host ""
Write-Host "NOTE: Old v0.56.0 RESULT.txt files are from the obsolete oracle"
Write-Host "(tests passed on buggy baseline). They are preserved as diagnostic"
Write-Host "evidence, not comparable with v0.56.2 results."
Write-Host ""

Pop-Location
