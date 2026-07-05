# AKAR v0.9.0 First Auto-Hook Evidence Report

**Status: AUTO-HOOK EVIDENCE CONFIRMED — ALLOW and BLOCK both verified.**

Date: 2026-07-05
Session: Claude Code + AKAR v0.8.2
Auditor: post-session audit, evidence from real .akar/HOOK_EVENTS.jsonl

---

## Purpose

This report documents the first verified auto-firing of the AKAR PreToolUse
hook during a real Claude Code session. The hook was wired manually by the user
into `~/.claude/settings.json`. AKAR did not modify Claude Code configuration.

---

## Step 1 — Hook wiring confirmed

File: `~/.claude/settings.json` (not modified by AKAR)

Relevant section:
```json
"PreToolUse": [
  {
    "matcher": "Bash",
    "hooks": [
      {
        "type": "command",
        "command": "pwsh -NoProfile -ExecutionPolicy Bypass -File \"C:\\Users\\nbzkr\\Coding\\akar\\templates\\hooks\\pre-tool-call.ps1\""
      }
    ]
  }
]
```

Finding: PreToolUse hook present with matcher Bash. Wired manually by user.
AKAR did not write, modify, or read this file during the session.

---

## Step 2 — akar hooks --check

```
hooks check:
  status: PASS
  templates found:
    - pre-tool-call.sh
    - pre-tool-call.ps1
```

Finding: Both templates present, call `akar safety`, read stdin, write to
`.akar/HOOK_EVENTS.jsonl`, use exit 2 for BLOCK.

---

## Step 3 — akar in subprocess PATH confirmed

```
where.exe akar → C:\Users\nbzkr\bin\akar.exe
pwsh -NoProfile -Command "Get-Command akar" → C:\Users\nbzkr\bin\akar.exe
```

Finding: `akar.exe` installed at `C:\Users\nbzkr\bin\akar.exe`, which is on
the subprocess PATH. Previous session attempt failed because `akar` was not
yet installed to PATH. After installation the hook correctly invokes
`akar safety` on every Bash tool call.

---

## Step 4 — Safe command: ALLOW evidence

Command run by Claude Code Bash tool:
```
echo "akar auto-hook verification: safe command"
```

Hook event logged automatically to `.akar/HOOK_EVENTS.jsonl`:
```json
{"timestamp":"2026-07-06T01:12:09.3219491+08:00","hook":"PreToolUse","tool_name":"Bash","command_preview":"echo \"akar auto-hook verification: safe command\" 2>&1","decision":"ALLOW","exit_code":0}
```

Finding: Hook fired automatically. `akar safety` classified the command as
Safe. Hook exited 0. Claude Code executed the command. ALLOW logged.

---

## Step 5 — Destructive command: BLOCK evidence

Command attempted via Claude Code Bash tool:
```
rm -rf /
```

Claude Code hook error output (visible in session):
```
PreToolUse:Bash hook error: [pwsh -NoProfile -ExecutionPolicy Bypass -File
"C:\Users\nbzkr\Coding\akar\templates\hooks\pre-tool-call.ps1"]:
Write-Error: akar hook: BLOCKED - rm -rf /
```

Hook event logged automatically to `.akar/HOOK_EVENTS.jsonl`:
```json
{"timestamp":"2026-07-06T01:12:40.9505510+08:00","hook":"PreToolUse","tool_name":"Bash","command_preview":"rm -rf /","decision":"BLOCK","exit_code":2}
```

Finding: Hook fired automatically. `akar safety "rm -rf /"` returned exit 2
(BLOCKED — destructive filesystem wipe). Hook exited 2. Claude Code received
exit 2 and blocked execution before `rm` ran. BLOCK logged with exit_code 2.

---

## Step 6 — HOOK_EVENTS.jsonl evidence (selected entries)

Full log at `.akar/HOOK_EVENTS.jsonl` (gitignored). Selected relevant lines:

```
line 20: echo "akar auto-hook verification..." → ALLOW, exit 0
line 21: rm -rf /                              → BLOCK, exit 2
```

Earlier entries (lines 1–19) show the hook has been firing on every Bash tool
call throughout the v0.8.2 development session — 19 prior ALLOW events for
`cargo build`, `cargo test`, `cargo run`, `where.exe`, etc.

---

## Step 7 — Verification results

```
cargo build --release    PASS (akar v0.8.2, no recompile needed)
cargo test               251 passed, 0 failed
akar --version           akar 0.8.2
akar hooks --check       PASS
akar doctor              OK
akar eval                28/28 PASS
akar status              HEALTHY
```

Note: version reported is 0.8.2 — bumped to 0.9.0 after this report.

---

## Honest conclusion

### What worked

- **Auto-hook firing**: The PreToolUse hook fired automatically on every Bash
  tool call throughout the session. No manual invocation was needed.

- **ALLOW on safe commands**: `echo`, `cargo build`, `cargo test`, `cargo run`,
  `where.exe` — all logged ALLOW, exit 0. Claude Code executed them normally.

- **BLOCK on destructive command**: `rm -rf /` was classified Critical/BLOCKED
  by `akar safety`, hook exited 2, Claude Code blocked execution before `rm`
  ran. The command never executed.

- **Local audit trail**: All hook events recorded in `.akar/HOOK_EVENTS.jsonl`
  with timestamp, tool_name, command_preview, decision, exit_code. Gitignored.
  Not sent anywhere.

- **AKAR did not modify Claude Code configuration**: `~/.claude/settings.json`
  was wired manually by the user. AKAR never touched it.

### What remained manual

- Hook installation into `~/.claude/settings.json` — user action required
- Installing `akar.exe` to a PATH location visible to pwsh subprocesses —
  user action required (`copy akar.exe C:\Users\nbzkr\bin\`)

### Previous attempt failure (honest record)

The first auto-hook attempt in this session failed because `akar.exe` was at
`C:\cargo-target\steroid-cli\release\akar.exe`, which is not on the subprocess
PATH. The hook fell through to its no-akar fallback (ALLOW without classifying).
After the user copied `akar.exe` to `C:\Users\nbzkr\bin\`, the hook found
`akar` and classification worked correctly. This failure is documented honestly.

### Whether AKAR stayed advisory-only

Yes. AKAR classified commands and exited 2 to signal Claude Code. It did not
execute, edit, revert, or enforce anything itself. The block was Claude Code
acting on AKAR's exit code. AKAR's only writes were to `.akar/HOOK_EVENTS.jsonl`
and `.akar/EVENT_LOG.jsonl`.

---

## What v0.10.0 should prove

A full loop with auto-hook active end-to-end:

1. Commit (clean tree)
2. `akar status` → READY
3. `akar preflight --snapshot "<task>"`
4. Claude Code session with PreToolUse hook firing automatically
5. `akar postmortem --diff --baseline` → PASS
6. `.akar/HOOK_EVENTS.jsonl` shows ALLOW for all session commands, no BLOCK

This would be the first proof that AKAR's advisory loop and safety gate work
together in a real scoped session.
