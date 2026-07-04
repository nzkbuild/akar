# Contributing to AKAR

AKAR is an early-stage project. Contributions are welcome, but please read this first.

## What's useful right now

- Bug reports with exact repro steps
- Corrections to docs that are wrong or misleading
- New eval scenarios (see `src/eval.rs` for examples)
- Windows compatibility fixes

## What to hold off on

- Large new features — the architecture is still settling
- New intelligence modules — these follow a defined roadmap
- Global `~/.claude` integration changes — these require careful review

## How to contribute

1. Fork the repo
2. Create a branch: `git checkout -b fix/your-fix`
3. Make changes, run `cargo test`
4. Open a pull request with a clear description

## Code style

- Follow existing patterns in `src/`
- No new external dependencies without discussion
- Tests required for new behavior
- Keep diffs small — AKAR practices what it preaches

## Reporting security issues

See [SECURITY.md](SECURITY.md).
