# Verify Recipe

<!-- Exact commands to run before declaring a task done.
     Keep this file up to date as the project evolves. -->

## Build Command

```sh
cargo build --workspace
```

Expected: exits 0, no warnings promoted to errors.

## Test Command

```sh
cargo test --workspace
```

Expected: all tests pass, output ends with `test result: ok`.

## Typecheck Command

```sh
cargo check --workspace
```

Expected: exits 0. Run this as a fast pre-check before the full build.

## Manual Checks

<!-- Steps that cannot be automated. Complete each before marking done. -->
- [ ] `akar status` prints correct project name and path
- [ ] `akar doctor` exits 0 and reports no panics
- [ ] `akar --version` prints the version from Cargo.toml
- [ ] Event log file is created on first append and appended-to (not overwritten) on subsequent calls
- [ ] Rotation renames to `.bak` and the original path is writable again after rotate

## Not Verified

<!-- Things this recipe does not cover. Be honest. -->
- Cross-platform file-permission edge cases (tested on Windows only)
- Concurrent writes to the event log from multiple processes
- Behaviour when disk is full during append
