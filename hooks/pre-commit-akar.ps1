# AKAR pre-commit hook: verify before committing
Write-Host "[AKAR] Running pre-commit verification..."
akar verify
exit $LASTEXITCODE
