# Claude Code Hook Playbook

AKAR integrates with Claude Code via the PreToolUse hook mechanism. This playbook documents correct hook setup and behavior.

## How PreToolUse Works

Claude Code fires a PreToolUse hook before executing each tool call. The hook receives a JSON payload on stdin describing the tool and its inputs.

- Hook exit 0: tool proceeds normally
- Hook exit 2: tool is BLOCKED — Claude Code does not execute it
- Hook exit 1 or other: treated as error, behavior depends on Claude Code version

AKAR uses exit 2 exclusively for BLOCK decisions.

## Hook Matcher

The AKAR hook targets the Bash tool only:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "pwsh \"C:\\path\\to\\akar\\templates\\hooks\\pre-tool-call.ps1\""
          }
        ]
      }
    ]
  }
}
```

The matcher `"Bash"` ensures the hook only fires for Bash tool calls, not Read, Write, Edit, or other tools.

## JSON Stdin Payload

Claude Code sends a JSON object to stdin. The hook must read it from stdin, not from command arguments.

Example payload:
```
{"session_id":"abc123","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{"command":"cargo test"}}
```

The hook must extract `tool_input.command` only — that is the shell command to classify.

## Extracting the Command

The hook extracts only the command field from the JSON, not the full payload. The full JSON blob must never be written to logs. Only the command string and the block decision are logged.

Bash hook pattern:
1. Read stdin: JSON=$(cat)
2. Extract command field from tool_input
3. Pass command to: akar safety "$COMMAND"
4. If akar safety exits 2: write to HOOK_EVENTS.jsonl, exit 2
5. Otherwise: exit 0

PowerShell hook pattern:
1. Read stdin: $json = $input | Out-String
2. Extract command via string parsing
3. Pass command to: akar safety $Command
4. If akar safety exits 2: write to HOOK_EVENTS.jsonl, exit 2
5. Otherwise: exit 0

## HOOK_EVENTS.jsonl Evidence

Every block must be logged to `.akar/HOOK_EVENTS.jsonl`. Each log entry records:
- timestamp
- tool_name
- command (the blocked command, not the full JSON)
- decision: BLOCK or ALLOW
- reason from akar safety

This file is the audit trail. AKAR reads it during postmortem and eval.

## AKAR Must Be in Subprocess PATH

The hook script runs in a subprocess. The subprocess must be able to find the `akar` executable.

On Windows: ensure the directory containing `akar.exe` is in the system PATH or specify the full path in the hook command.

On Unix: ensure `akar` is in PATH or use an absolute path.

If the hook cannot find `akar`, it must fail closed (exit 2, block the tool call) rather than silently allowing.

## Fail-Closed Behavior

If the hook:
- Cannot find the akar binary
- Cannot read stdin
- Encounters a parse error
- Times out

The hook must exit 2 (block) rather than exit 0 (allow). Fail-closed is always safer than fail-open for a safety hook.

## AKAR Does Not Edit settings.json

AKAR never automatically edits `~/.claude/settings.json`. Hook registration is always a manual step:

1. Copy hook template from `templates/hooks/`
2. Edit the command path to point to your akar binary
3. Add the hook block to `~/.claude/settings.json` manually
4. Restart Claude Code to load the new hook

The `akar hooks --install` command only copies templates to `.akar/hooks/`. It does not touch settings.json.

## Verifying the Hook is Active

Run: `akar hooks --check`

This verifies:
- Hook templates exist
- Templates call `akar safety`
- Templates read from stdin
- Templates write to `HOOK_EVENTS.jsonl`
- Templates use exit 2 for BLOCK

Then verify actual hook firing by running a blocked command in Claude Code and checking `.akar/HOOK_EVENTS.jsonl` for an entry.

## Troubleshooting

### Hook not firing
- Confirm the hook is registered in `~/.claude/settings.json`
- Confirm Claude Code was restarted after adding the hook
- Check the matcher is exactly `"Bash"` (case-sensitive)

### HOOK_EVENTS.jsonl is empty
- Hook may not be in PATH — check akar binary location
- Hook may be exiting 0 silently — check hook script for error handling
- Confirm the hook script has execute permission (Unix)

### akar not found in hook
- Add akar binary directory to system PATH
- Or specify the full absolute path in the hook command
- Restart Claude Code after PATH changes

### False blocks
- Inspect HOOK_EVENTS.jsonl for the blocked command
- Run `akar safety "the command"` manually to see the classification
- If classification is wrong, it is a safety module issue, not a hook issue
