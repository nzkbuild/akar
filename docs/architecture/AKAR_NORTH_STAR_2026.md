# AKAR North Star 2026 — Bounded Context Runtime

Date: 2026-07-14  
Status: PROPOSED — synthesized project direction  
Current source version: `0.56.1`

## 1. Executive direction

AKAR is a local, bounded context and governance runtime for AI-assisted software engineering.

The user installs AKAR once and enables it in a supported coding-agent host. The AI model remains the planner, builder, and executor. AKAR works underneath the host to compile the smallest safe task context, preserve critical state across context pressure, surface grounded project capabilities, prevent known-dangerous actions, and measure whether the model completed the task correctly with less waste.

AKAR must improve useful work per token without weakening correctness, safety, verification, or user intent. It must work at a safe baseline when model or proxy metadata is unavailable, then improve when trustworthy usage signals exist.

AKAR is not a universal compressor. It cannot promise zero information loss or identical gains across every model. Its promise is narrower and testable:

> AKAR compiles the smallest safe task context from local evidence, protects exact operational state, adapts only to trustworthy host/provider signals, degrades conservatively when signals are unavailable, and proves benefit through fixed-budget quality tests.

This direction preserves the original North Star. It adds the missing resource-governance mechanism needed to reduce context loss, overflow, repeated prompting, and token waste without turning AKAR into a model router or proxy.

## 2. Original North Star retained

The original doctrine remains authoritative in intent:

- AKAR solves root causes of poor AI work, not symptoms alone.
- AKAR helps both user and AI model.
- User installs once and works normally inside preferred coding-agent host.
- AKAR enhancement can be enabled or disabled.
- AI model remains worker, planner, builder, and executor.
- AKAR reduces wasted tokens, requests, input/output, bad code, excessive diffs, hallucination, context loss, dangerous commands, incomplete work, and repeated prompting.
- AKAR increases task understanding, safe execution, verification, clean diffs, environment awareness, cheap-model usefulness, and expensive-model efficiency.
- Improvement claims require controlled evidence.

The new direction does not replace these goals. It makes context and token efficiency an explicit kernel resource-governance responsibility.

## 3. Why direction must change now

The v0.43 assessment correctly identified AI-facing delivery as prerequisite to token optimization. Releases v0.48–v0.56 then built most of that delivery path:

- managed Claude guidance
- `UserPromptSubmit` auto-context design
- task and capability context compilation
- deterministic capability selection
- bounded capability/profile sections
- two-stage verification guidance
- benchmark fixture and scoring rubric

That progress changes the sequencing decision. Token and context work was premature at v0.43 because AKAR had no delivery path. It is now the correct next architectural concern because delivery exists in code, although live reliability and measurement remain unproven.

Current evidence also blocks overclaiming:

- Current benchmark did not measure AKAR impact because `UserPromptSubmit` failed to fire in enabled interactive runs.
- Input/output token counts were unavailable.
- Current `context_pack` inventories paths but does not compile file content.
- Current request-pressure modes do not use real context usage.
- Current model detection is display-oriented and incomplete.
- Current subsection character caps do not bound total injected context.

Therefore next phase must be measurement and hard bounds, not broad compression features.

## 4. Current North Star vs current build vs merged direction

| Dimension | Original North Star | Current build (`0.56.1`) | Merged North Star |
|---|---|---|---|
| Product role | Root/runtime enhancement layer | Local Rust advisory CLI plus Claude Code hooks | Local bounded context and governance runtime |
| Worker | AI model | Claude Code/model performs work | Host model remains sole project-work executor |
| User burden | Install once; normal prompts | Near-zero relay designed; `akar finish` remains manual; delivery unreliable in benchmark | One-time enablement; normal prompts; lifecycle automation only where host proves reliable |
| AI-facing delivery | Automatic | `UserPromptSubmit` adapter and `CLAUDE.md` path exist | Host adapter with end-to-end delivery evidence and immediate disable path |
| Context handling | Reduce loss and overload | Metadata/instruction compaction; no transcript management | Bounded context compiler with protected state ledger and operational HOT/WARM/COLD tiers |
| Token efficiency | Reduce input/output waste | Rough character estimate; no trustworthy usage signal | Provider-native or host-reported counts when available; conservative fixed budget otherwise |
| Model support | Cheap models improve; expensive models waste less | Claude-family and limited `gpt-4` display heuristics | Opaque model identity; no hard-coded safety assumptions; fixed-budget model matrix |
| Proxy support | Work across preferred tools | Passive compatibility when Claude Code hooks still work | Capability levels; safe local fallback; no assumption that API-shape compatibility means feature parity |
| Capability awareness | Better tool/environment use | Deterministic discovery and top-five ranking | Retained as grounded input to bounded compiler |
| Safety | Prevent dangerous actions | `PreToolUse` templates and classifier | Retained as protected kernel policy, never compressed away |
| Verification | Better, honest completion | Project-aware two-stage guidance | Retained as protected context plus measured completion gate |
| Diff discipline | Smaller, cleaner diffs | Baseline and postmortem measurement | Retained; automate measurement only through proven lifecycle events |
| Learning | Improve from failures | Local learning patches and reports | Evidence-driven policy changes; no autonomous self-modification |
| Evidence | Benchmark-like uplift | Fixture/rubric exist; enabled comparison invalid | Fixed-budget, repeated, model/provider/proxy quality and efficiency evaluation |

## 5. Product contract

### AKAR must

1. Compile one bounded, self-contained task context.
2. Preserve latest user intent, constraints, safety rules, unresolved failures, exact technical spans, changed-file state, and verification commands.
3. Prefer deterministic deletion, deduplication, and extraction before generated summaries.
4. Use trustworthy token/context metadata when available and label its source.
5. Fall back to conservative local budgets when metadata is absent or suspect.
6. Keep local-only core operation functional without model API or proxy access.
7. expose what was retained, omitted, estimated, and measured.
8. Prove delivery before claiming model improvement.
9. Prove quality retention before shipping a reduction policy.
10. Remain immediately disableable.

### AKAR must not

- execute project work
- route model calls
- become an HTTP proxy
- claim universal tokenizer accuracy
- infer provider guarantees from model aliases
- assume “OpenAI-compatible” means tokenizer, tool, cache, limit, error, or billing compatibility
- hard-code volatile model limits as safety truth
- add vector DB, embeddings, daemon, or cloud telemetry without measured need
- summarize exact commands, paths, symbols, errors, IDs, hashes, limits, or security constraints
- silently drop unknown state
- claim token savings or quality uplift without valid comparison data

## 6. Architecture projection

### 6.1 Kernel authority

```text
Safety and user intent (hard floor)
  └─ Bounded Context Kernel
       ├─ Context Compiler
       ├─ Capability Intelligence
       ├─ Request/Pressure Intelligence
       ├─ Verification and Diff Evidence
       ├─ Context Ledger
       └─ Host Adapter
            └─ Optional Provider Metadata Adapter
```

Skills and capabilities remain drivers or libraries, never policy authorities.

### 6.2 Canonical compiler input

```text
ContextInput
- latest request
- objective and acceptance criteria
- user and project constraints
- safety and stop conditions
- changed files and active state
- unresolved failures
- exact technical spans
- verification commands
- selected capabilities
- relevant decisions
- usage signal
- opaque provider/model metadata
```

### 6.3 Compiler output

```text
CompiledContext
- protected content
- relevant content
- optional content
- omitted-content manifest
- exact byte and character size
- token count or estimate
- count source and confidence
- pressure mode
- version/checksum
```

Output is one globally capped context block. AKAR must not inject compact guidance and then require loading a second overlapping full contract.

### 6.4 Separation of concerns

**Core compiler**

- host-independent data model
- normalization and deduplication
- relevance ranking
- budget allocation
- Unicode-safe rendering
- retention checks
- local audit metrics

**Claude Code adapter**

- parse hook input
- emit supported hook output
- record delivery evidence
- checkpoint protected state at `PreCompact`
- restore bounded state at `SessionStart`
- remain synchronous and fail safely

**Optional provider metadata adapter**

- discover model metadata when authorized and available
- count full model-visible input through native endpoint where supported
- report context, cache, and tool capability as optional facts
- never route requests or become required for safe operation

Do not add another host adapter until Claude Code delivery and benefit are proven.

## 7. Protected context policy

Protected content receives budget first and must never be rewritten into an approximate summary:

1. safety and security constraints
2. latest user request
3. objective and acceptance criteria
4. explicit allowed/forbidden boundaries
5. unresolved failures and blockers
6. tool-call IDs and protocol state when visible
7. commands, paths, URLs, code symbols, error strings, hashes, identifiers, and numeric limits
8. changed-file state
9. verification commands
10. decisions whose rationale affects correctness

When protected content alone exceeds budget, AKAR must report boundary state and preserve a pointer/checkpoint. It must not silently truncate required content.

## 8. Reduction ladder

Apply cheapest, safest operation first:

1. Remove exact duplicates.
2. Remove stale and completed tool output.
3. Replace repeated content with one canonical reference.
4. Keep current state; drop transition narration.
5. Remove unrelated COLD context.
6. Select relevant WARM context by deterministic task relevance.
7. Extract source facts verbatim.
8. Use generated summarization only as optional future fallback.

Generated summarization is not part of initial implementation. If later added, protected spans stay outside rewriting and summary output must pass retention checks.

## 9. Operational context tiers

### HOT — always eligible, protected first

- latest request
- active objective
- constraints and stop conditions
- changed files
- unresolved errors
- current decision
- verification commands
- active tool protocol state

### WARM — load by deterministic relevance

- architecture decisions
- related failure lessons
- subsystem conventions
- grounded capability metadata
- known relevant bugs

### COLD — excluded by default

- completed exploration
- old successful tool output
- unrelated decisions
- repeated documentation
- historical narration

### EXTERNAL — pointer, verify fresh

- large reports and logs
- current provider docs
- APIs, pricing, model limits, and package versions
- generated artifacts

## 10. Model and proxy compatibility contract

AKAR support has levels, not one binary claim.

| Level | Available signal | AKAR behavior |
|---|---|---|
| 0 — Local safe baseline | Claude Code hooks and filesystem only | Fixed conservative global budget; deterministic compiler; no cache or exact-token claims |
| 1 — Usage visible | Host/proxy reports usage | Pressure modes, overflow warning, efficiency metrics |
| 2 — Metadata visible | Actual model limits/count endpoint/capabilities available | Model-specific budget and trustworthy counting |
| 3 — Native features | Provider supports caching/context editing/compaction | Optional adapter uses proven native features without changing core contract |

Rules:

- Treat model ID, provider, gateway, and route as opaque metadata.
- Record whether counts are exact, provider estimates, local estimates, or unknown.
- Query current metadata where possible; do not freeze model tables.
- Never use OpenAI tokenizers as authority for non-OpenAI models.
- Never assume proxy aliases reveal upstream limits or tokenizer.
- Never assume cache usage includes cache reads/writes unless documented.
- Preserve safe Level 0 behavior whenever probing fails.

## 11. Quality and efficiency definition

AKAR succeeds only when quality stays equal or improves under lower resource use.

### Mandatory retention gates

- latest request retained
- objective and negation unchanged
- security and trust boundaries retained
- exact technical spans retained byte-for-byte
- unresolved failures retained
- verification remains runnable
- current state cannot be replaced by stale state
- tool-call correlation remains valid when applicable

### Outcome metrics

- task correctness
- hidden and visible test results
- exact-span retention
- tool-call validity
- relevant evidence recall
- files and LOC changed
- unrelated edits
- verification completion
- input, output, and cached tokens by count source
- requests and corrective turns
- latency and cost where available
- context overflow and compaction recovery
- safety-policy retention

### Comparison conditions

1. Model without AKAR
2. Current AKAR
3. Bounded deterministic compiler
4. Compiler plus query-aware extraction
5. Optional summarization only after prior conditions prove insufficient

Runs must use fixed budgets and repeated trials. Invalid delivery means condition is not measured, not failed or passed.

## 12. Revised progression

Progression is evidence-gated, not version-number driven.

### Stage 0 — Honest current baseline

Current state:

- advisory/governance foundation works locally
- deterministic capabilities and task profiles exist
- Claude Code delivery code exists
- live delivery benchmark failed
- token impact remains unmeasured

Exit gate:

- current docs agree with actual hook/setup behavior
- baseline tests pass or known failures are explicitly isolated

### Stage 1 — Reliable bounded delivery

Build only:

- one global `additionalContext` budget
- bounded task text
- Unicode-safe truncation
- no overlapping inline/`NEXT_RUN` payload
- exact output-size metrics
- delivery event/checksum evidence
- real Claude Code hook smoke matrix

Exit gate:

- context delivered in at least 95% of supported test sessions
- zero mandatory-retention failures
- total output never exceeds configured bound
- disable path works immediately

### Stage 2 — Context continuity

Build only:

- protected context ledger
- `PreCompact` checkpoint
- bounded `SessionStart` restore
- operational HOT/WARM/COLD selection
- deterministic stale-output removal and query-aware extraction

Exit gate:

- protected state survives compaction in all fixtures
- stale context never overrides latest request
- recovery uses fewer tokens than full-history replay

### Stage 3 — Measured model uplift and waste reduction

Build only measurement needed for controlled trials. Run cheap and strong models with and without AKAR.

Exit gate:

- no statistically meaningful task-success regression
- 100% mandatory exact-span, security, and tool-validity retention
- lower median total tokens or fewer corrective turns
- fewer context-overflow failures
- methodology and raw results published, including negative results

### Stage 4 — Optional provider-aware optimization

Add provider metadata/count adapters only where Stage 3 shows fixed local budgets are insufficient.

Exit gate:

- adapter failures always return to Level 0
- native counts validated against actual usage where available
- cache and native context features produce measured benefit

### Stage 5 — Cross-host expansion

Explore one additional host only after Claude Code stages pass and user demand exists.

Exit gate:

- canonical compiler unchanged
- host adapter carries transport differences
- equivalent safety and retention evidence exists

### Stage 6 — North Star beta

AKAR becomes runtime enhancement beta when:

- normal use requires no repeated AKAR commands
- supported hosts have reliable delivery
- context continuity survives compaction
- quality-per-token gains are published across multiple model tiers
- unknown models/proxies retain safe baseline behavior
- cross-platform and external-user evidence exists
- no safety boundary was weakened to obtain efficiency

## 13. Immediate project projection

### Next release: bounded-delivery foundation

Priority order:

1. Prove `UserPromptSubmit` delivery end to end.
2. Add one global injected-context bound.
3. Make every truncation Unicode-safe.
4. Bound and protect latest task text.
5. Remove duplicate inline and `NEXT_RUN` guidance.
6. Report output bytes/chars and token-count source.
7. Add retention tests for security, exact spans, failures, and verification.
8. Re-run enabled benchmark conditions only after delivery evidence passes.

### Following release: protected context ledger

Only after bounded delivery passes:

1. Define compact ledger schema.
2. Capture protected state during `PreCompact`.
3. Restore it during compact `SessionStart` context.
4. Validate current request and repository state before reuse.
5. Measure recovery against full-history/manual rediscovery baseline.

## 14. Work explicitly deferred

Defer until evidence demands it:

- LLM-based summarizer
- universal tokenizer
- vector database or embeddings
- new proxy or gateway
- model routing or fallback chains
- cross-provider tool-call translator
- daemon or watcher
- autonomous project execution
- automatic memory patch application
- Codex/OpenCode adapter
- hard-coded model capability catalog

Deletion, reuse, stdlib, and existing Claude Code lifecycle features outrank new systems.

## 15. Documentation reconciliation

This document supersedes direction claims in these files where they conflict:

- `docs/architecture/AKAR_OS.md`: gateway/model-routing and autonomous OS framing are not current commitments.
- `docs/architecture/INTELLIGENCE_ROADMAP.md`: L2 routing and L7 autonomous application are historical/speculative, not release targets.
- `docs/architecture/PRODUCT_ROADMAP.md`: already marked superseded.
- `docs/architecture/AKAR_V1_ARCHITECTURE_FREEZE_PROPOSAL.md`: remains useful for safety boundaries, but statements that AKAR never installs project-local Claude settings or hooks conflict with current `init --hooks` behavior.
- `docs/audits/AKAR_V0_43_NORTH_STAR_DRIFT_GAP_ASSESSMENT.md`: remains authoritative history and original doctrine; its “postpone token optimization” sequencing condition has now advanced because AI-facing delivery code exists. Measurement still comes before optimization.
- `README.md`: current statements about manual relay, hook installation, and settings modification require later factual update against actual behavior.

Historical audit reports remain evidence and must not be rewritten to match current direction.

## 16. Decision rules

Before accepting future work, ask in order:

1. Does it preserve original North Star outcomes?
2. Does it fix measured failure or close required evidence gap?
3. Can existing deterministic code or host-native lifecycle solve it?
4. Does it keep protected context exact?
5. Does it work at Level 0 without provider assumptions?
6. Can benefit be measured under fixed budget?
7. Is smaller deletion or consolidation sufficient?

If any safety, retention, or evidence answer fails, do not ship the optimization.

## 17. Primary references

### Project evidence

- `docs/audits/AKAR_V0_43_NORTH_STAR_DRIFT_GAP_ASSESSMENT.md`
- `docs/audits/AKAR_CURRENT_STATE.md`
- `docs/audits/benchmarks/BENCHMARK_REPORT.md`
- `docs/kernel/CONTEXT_BUDGET.md`
- `docs/architecture/AKAR_V1_ARCHITECTURE_FREEZE_PROPOSAL.md`

### Host and provider behavior

- [Claude Code hooks](https://code.claude.com/docs/en/hooks)
- [Claude Code context management](https://code.claude.com/docs/en/how-claude-code-works)
- [Claude Code memory](https://code.claude.com/docs/en/memory)
- [Claude Code LLM gateway](https://code.claude.com/docs/en/llm-gateway)
- [Anthropic token counting](https://platform.claude.com/docs/en/build-with-claude/token-counting)
- [Anthropic context windows](https://platform.claude.com/docs/en/build-with-claude/context-windows)
- [Anthropic prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching)
- [Anthropic OpenAI compatibility](https://platform.claude.com/docs/en/api/openai-sdk)
- [OpenAI token counting](https://developers.openai.com/api/docs/guides/token-counting)
- [OpenAI prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching)
- [Gemini token counting](https://ai.google.dev/gemini-api/docs/tokens)
- [Gemini OpenAI compatibility](https://ai.google.dev/gemini-api/docs/openai)
- [Amazon Bedrock CountTokens](https://docs.aws.amazon.com/bedrock/latest/userguide/count-tokens.html)
- [Cloudflare AI Gateway caching](https://developers.cloudflare.com/ai-gateway/features/caching/)
- [LiteLLM token usage](https://docs.litellm.ai/docs/completion/token_usage)

### Compression and evaluation evidence

- [LLMLingua](https://arxiv.org/abs/2310.05736)
- [LongLLMLingua](https://arxiv.org/abs/2310.06839)
- [Selective Context](https://arxiv.org/abs/2304.12102)
- [RECOMP](https://arxiv.org/abs/2310.04408)
- [Lost in the Middle](https://aclanthology.org/2024.tacl-1.9/)
- [LongBench](https://aclanthology.org/2024.acl-long.172/)
- [RULER](https://arxiv.org/abs/2404.06654)
