# Security Policy

## Scope

AKAR is a local CLI tool. It does not send data to external services.

## Reporting a vulnerability

If you find a security issue (secret exposure, path traversal, command injection, etc.):

1. Do not open a public issue.
2. Email the maintainer directly or open a private GitHub security advisory.
3. Include a clear description and repro steps.

## What AKAR does with your data

- All telemetry is local to `.akar/EVENT_LOG.jsonl` — never transmitted.
- AKAR reads your project files but does not upload them.
- AKAR does not store API keys, tokens, or credentials.
- The `redact()` function strips common secret patterns before logging.

## Known limitations

- AKAR is in early development. The secret redaction is best-effort, not exhaustive.
- Review `.akar/` contents before committing to a public repo.
