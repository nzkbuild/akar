# Decisions

<!-- Record every significant architectural or design decision here.
     "Supersedes" references the Date of the decision this one replaces, or "-" if new. -->

| Date | Decision | Reason | Supersedes |
|------|----------|--------|------------|
| 2026-01-01 | Use append-only JSONL for event log | Simple, grep-friendly, no schema migrations | - |
| 2026-01-01 | No external crate dependencies | Keep binary small and build fast | - |
