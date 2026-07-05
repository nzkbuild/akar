#!/usr/bin/env bash
# AKAR pre-tool-call hook (bash)
#
# Usage: called by Claude Code before each tool execution.
# Claude Code passes the command string as the first argument or via stdin
# depending on hook configuration. This script calls `akar safety` and
# exits non-zero only for BLOCKED commands.
#
# Install: copy to your project's hook location and register in
# ~/.claude/settings.json under hooks.preToolCall. AKAR does not install
# this automatically.

set -euo pipefail

COMMAND="${1:-}"

# Fall back to stdin if no argument given
if [ -z "$COMMAND" ]; then
    COMMAND="$(cat)"
fi

if [ -z "$COMMAND" ]; then
    # Nothing to check — allow
    exit 0
fi

# Run akar safety check
if ! command -v akar &>/dev/null; then
    echo "akar: not found in PATH — skipping safety check" >&2
    exit 0
fi

OUTPUT="$(akar safety "$COMMAND" 2>&1)"
EXIT_CODE=$?

echo "$OUTPUT"

# akar safety exits 2 for BLOCKED commands
if [ $EXIT_CODE -eq 2 ]; then
    echo "akar: command blocked — $COMMAND" >&2
    exit 1
fi

exit 0
