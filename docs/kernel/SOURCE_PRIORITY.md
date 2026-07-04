# Source Priority

## For Code & Architecture Decisions

1. Actual code (what the repo does)
2. Tests (what behavior is expected)
3. Config files (what is configured)
4. Project docs (what was intended)
5. Project memory (what was learned)
6. Global memory (cross-project lessons)
7. Model knowledge (fallback only)

## For External / Current Facts

1. Current official source (live docs, spec, registry)
2. Memory (only if source is unavailable and recency is low-risk)

## Rules

Must:
- Resolve conflicts by ranking — higher source wins
- Treat model knowledge as lowest trust for anything version-sensitive

Should:
- Fetch current docs when implementing against a versioned API or package
- Note source when making a claim that depends on external facts

Never:
- Implement based on model knowledge alone for current external facts (pricing, API shape, package versions, laws)
- Treat memory as authoritative when a live source is reachable
- Proceed without a source when the task requires current external facts
