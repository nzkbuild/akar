# AKAR Release Checklist

Run before tagging any release.

## Install verification

- [ ] `cargo build --release` — release binary compiles
- [ ] `akar --version` — prints correct version matching Cargo.toml
- [ ] `akar bootstrap` — creates .akar/ and templates idempotently
- [ ] `akar doctor` — returns OK after bootstrap
- [ ] `akar status` — shows HEALTHY with correct version

## Core workflow verification

- [ ] `akar preflight "fix the login button"` — returns nonempty strategy
- [ ] `akar run "fix the login button"` — completes scaffold workflow
- [ ] `akar telemetry` — shows event count after run
- [ ] `akar postmortem` — shows outcome after run
- [ ] `akar eval` — all evals pass (currently 28/28)

## Test suite

- [ ] `cargo test` — all tests pass, 0 failed
- [ ] `cargo build` — 0 warnings

## Benchmark smoke test

- [ ] Run `akar eval` and confirm pass/fail counts match expected
- [ ] Check `.akar/EVENT_LOG.jsonl` has entries after workflow run
- [ ] Check `akar postmortem` correctly classifies clean vs degraded

## Security

- [ ] No secrets in staged files (`git diff --cached | grep -i "token\|password\|secret\|api_key"`)
- [ ] `.env` files not tracked
- [ ] `EVENT_LOG.jsonl` not tracked (gitignored runtime artifact)
- [ ] `SKILL_INVENTORY.md`, `NEXT_RUN.md`, `LEARNING_PATCHES.md` not tracked

## Git hygiene

- [ ] `git status` — working tree clean, no untracked artifacts
- [ ] No `test_output*.txt` or `reports/` files staged

## Docs

- [ ] README version line updated
- [ ] AKAR_ADOPTION_NOTES.md updated if behavior changed
- [ ] RELEASE_CHECKLIST.md reviewed and updated

## Commit

- [ ] All changed files staged
- [ ] Commit message follows: `feat: AKAR vX.Y.Z — short description`
- [ ] Co-Authored-By line included
