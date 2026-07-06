#!/usr/bin/env bash
# AKAR pre-tool-call hook (bash)
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

set -uo pipefail

# Read JSON from stdin
JSON="$(cat)"

if [ -z "$JSON" ]; then
    exit 0
fi

# ---------- v0.29.0: choose log root from Claude Code "cwd" field ----------
# Priority: explicit cwd from hook JSON > process cwd as fallback.
LOG_ROOT="$(pwd)"
CWD_FROM_JSON=""
if echo "$JSON" | grep -q '"cwd"'; then
    CWD_FROM_JSON="$(echo "$JSON" | grep -o '"cwd"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | grep -o '"[^"]*"$' | tr -d '"')"
fi
if [ -n "$CWD_FROM_JSON" ] && [ -d "$CWD_FROM_JSON" ]; then
    LOG_ROOT="$CWD_FROM_JSON"
fi
# --------------------------------------------------------------------------

# Ensure .akar/ exists under the chosen log root
AKAR_DIR="$LOG_ROOT/.akar"
mkdir -p "$AKAR_DIR"
HOOK_LOG="$AKAR_DIR/HOOK_EVENTS.jsonl"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo 'unknown')"
# JSON-escape log_root for embedding (a path may contain backslashes on Windows)
LOG_ROOT_ESC="$(echo "$LOG_ROOT" | sed 's/\\/\\\\/g; s/"/\\"/g')"

write_event() {
    local tool="$1" preview="$2" decision="$3" code="$4"
    # Escape preview for JSON: backslash, quote, newline, tab
    preview="$(echo "$preview" | sed 's/\\/\\\\/g; s/"/\\"/g; s/$/\\n/g' | tr -d '\n' | sed 's/\\n$//')"
    echo "{\"timestamp\":\"$TIMESTAMP\",\"hook\":\"PreToolUse\",\"tool_name\":\"$tool\",\"command_preview\":\"$preview\",\"decision\":\"$decision\",\"exit_code\":$code,\"log_root\":\"$LOG_ROOT_ESC\"}" >> "$HOOK_LOG"
}

# Extract tool_name
TOOL_NAME=""
if echo "$JSON" | grep -q '"tool_name"'; then
    TOOL_NAME="$(echo "$JSON" | grep -o '"tool_name"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | grep -o '"[^"]*"$' | tr -d '"')"
fi

# Only check Bash tool calls
if [ "$TOOL_NAME" != "Bash" ]; then
    echo "akar hook: tool=$TOOL_NAME — skipping (not Bash)"
    write_event "$TOOL_NAME" "" "SKIP" 0
    exit 0
fi

# Extract tool_input.command
COMMAND=""
if echo "$JSON" | grep -q '"command"'; then
    COMMAND="$(echo "$JSON" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | grep -o '"[^"]*"$' | tr -d '"')"
fi

if [ -z "$COMMAND" ]; then
    echo "akar hook: Bash tool with no command — allowing"
    write_event "Bash" "" "ALLOW" 0
    exit 0
fi

# Redact obvious secrets and truncate preview to 300 chars
PREVIEW="$(echo "$COMMAND" | sed 's/\(password\|token\|secret\|key\|api_key\|apikey\|bearer\)[[:space:]]*[=:][[:space:]]*[^[:space:]]*/\1=[REDACTED]/gi')"
PREVIEW="$(echo "$PREVIEW" | cut -c1-300)"

# Check akar is available
if ! command -v akar &>/dev/null; then
    echo "akar: not found in PATH — skipping safety check" >&2
    write_event "Bash" "$PREVIEW" "ALLOW" 0
    exit 0
fi

# Run akar safety check
OUTPUT="$(akar safety "$COMMAND" 2>&1)"
EXIT_CODE=$?

echo "$OUTPUT"

# Determine decision label
if [ "$EXIT_CODE" -eq 2 ]; then
    DECISION="BLOCK"
    HOOK_EXIT=2
elif echo "$OUTPUT" | grep -qiE "WARN|High|Critical"; then
    DECISION="WARN"
    HOOK_EXIT=0
else
    DECISION="ALLOW"
    HOOK_EXIT=0
fi

write_event "Bash" "$PREVIEW" "$DECISION" "$HOOK_EXIT"

# Exit 2 blocks in Claude Code. Exit 1 does NOT block.
if [ "$EXIT_CODE" -eq 2 ]; then
    echo "akar hook: BLOCKED — $COMMAND" >&2
    exit 2
fi

exit 0
