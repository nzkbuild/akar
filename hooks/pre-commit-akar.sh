#!/bin/bash
# AKAR pre-commit hook: verify before committing
echo "[AKAR] Running pre-commit verification..."
akar verify
exit $?
