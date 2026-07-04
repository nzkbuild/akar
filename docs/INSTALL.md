# AKAR Install Guide

## Requirements

- Rust 1.70+ and Cargo
- Windows (primary target); macOS and Linux supported

## Install from source

```powershell
git clone https://github.com/nzkbuild/akar.git
cd akar
cargo build --release
# optional: add to PATH
copy target\release\akar.exe C:\Users\<you>\bin\akar.exe
```

## Verify install

```
akar --version
```

Expected output: `akar 0.2.1`

## Initialize a project

Navigate to any project directory, then:

```
akar bootstrap
akar doctor
```

`bootstrap` creates `.akar/` with 9 memory template files.
`doctor` checks that `.akar/` and `~/.claude/akar/` exist.
If doctor reports issues, run `akar doctor --fix`.

## Project-local state

All AKAR runtime state lives in `.akar/` inside your project.
`.akar/EVENT_LOG.jsonl` is gitignored by default.
`.akar/*.md` templates are committed as project starter state.

## Global Claude Code config

AKAR does NOT edit `~/.claude/` by default.
`akar bootstrap` creates `~/.claude/akar/` only (AKAR-specific dir).
Slash commands in `.claude/commands/` are project-local and optional.
To use them, copy `.claude/commands/akar-*.md` to your project's `.claude/commands/`.

## Troubleshooting

| Command | Purpose |
|---|---|
| `akar doctor` | Read-only health check |
| `akar doctor --fix` | Safe reversible fixes |
| `akar status` | Full runtime health at a glance |
