# AKAR Release Checklist

Run before tagging any release.

## Required

- [ ] `cargo test` — all tests pass, 0 failed
- [ ] `akar --version` — prints correct version matching Cargo.toml
- [ ] `akar bootstrap` — creates .akar/ and templates idempotently
- [ ] `akar doctor` — returns OK after bootstrap
- [ ] `akar eval` — all evals pass (currently 25/25)
- [ ] `git status` — working tree clean, no untracked artifacts
- [ ] `cargo build --release` — release binary compiles

## Security

- [ ] No secrets in staged files (`git diff --cached | grep -i "token\|password\|secret\|api_key"`)
- [ ] `.env` files not tracked
- [ ] EVENT_LOG.jsonl not tracked (runtime artifact)

## Docs

- [ ] README version line updated
- [ ] AKAR_ADOPTION_NOTES.md updated if behavior changed
- [ ] RELEASE_CHECKLIST.md reviewed

## Warnings

- [ ] `cargo build` produces no warnings (or all suppressed with documented reason)

## Commit

- [ ] All changed files staged
- [ ] Commit message follows: `feat: AKAR vX.Y.Z — short description`
- [ ] Co-Authored-By line included
