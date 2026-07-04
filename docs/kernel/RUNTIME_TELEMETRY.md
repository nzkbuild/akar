# Kernel Policy: Runtime Telemetry

## Core Rule

Local only. No cloud. No secrets. Append-only.

## Record Schema

Each telemetry entry is one JSON line appended to `EVENT_LOG.jsonl`.

```jsonc
{
  "mission_id": "<uuid>",
  "timestamp": "<ISO 8601>",
  "model": "<model-id>",
  "gateway": "<gateway-name>",
  "task_type": "<build|fix|refactor|research|audit|architect>",
  "autonomy": "<supervised|autopilot>",
  "cost_mode": "<min|balanced|max>",
  "skills_selected": ["<skill-id>"],
  "requests_used": 0,
  "tokens": { "input": 0, "output": 0 },
  "files_changed": ["<relative-path>"],
  "verification_result": "<passed|partial|skipped|failed>",
  "failure_class": "<type from learning taxonomy | null>",
  "lesson_created": "<LP-id | null>"
}
```

## Rules

- Append only. Never rewrite or delete existing entries.
- Redact all secrets before writing. No API keys, tokens, or credentials.
- File paths are relative to project root; no absolute paths that leak user info.
- Rotate when EVENT_LOG.jsonl exceeds 10 MB: rename to EVENT_LOG.<date>.jsonl.
- Each mission writes exactly one entry on completion or Resume stop.

## Must

- Record every mission that reaches Mission Compiler.
- Record verification_result honestly; never write "passed" without evidence.
- Record failure_class when any failure is detected.
- Record lesson_created when an LP is written.

## Should

- Record skills_selected at activation time, not planning time.
- Record requests_used as the actual count, not the budget.

## Never

- Upload to any external endpoint.
- Write unredacted secrets or credentials.
- Run as an always-on background watcher.
- Write partial entries; wait for mission completion or Resume to flush.

## Storage

- Path: `<project-root>/EVENT_LOG.jsonl`
- Format: newline-delimited JSON (NDJSON)
- Rotation: rename on size threshold; no deletion

## Effective

v0.1.0 (basic). v0.1.1 (schema extended). v0.1.2 (all fields required).
