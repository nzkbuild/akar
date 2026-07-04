# AKAR Product Roadmap

## Version Progression

### v0.1.0 — Foundation CLI (Done)
- Bootstrap sequence: init, mission, eval, doctor, status
- Task Contract schema defined
- Gateway routing (model profile + cost mode)
- Basic skill loading via Claude Code skill system
- EVENT_LOG.jsonl append-only telemetry

### v0.1.1 — Architecture Refinement (Current)
- OS framing adopted: AKAR Kernel authority hierarchy
- Skill Intelligence: role classification, conflict detection
- Request Intelligence: pressure modes, resume mechanism
- Learning Intelligence: failure taxonomy, learning patch format
- 12 architecture/policy docs published
- RFC lifecycle introduced

### v0.1.2 — Optimization
- Skill registry with runtime classification
- Context Pack: load/evict discipline enforced
- Request pressure adaptation live (not just documented)
- Learning patch application pipeline (observe → store → apply)
- Doctor recovery flows for top 5 failure classes

### v0.2.0 — First Stable Runtime
- All L3–L5 intelligence levels functional (not just specified)
- Eval suite covers skill conflicts, request spikes, verification gaps
- Runtime telemetry queryable locally
- Mission Compiler handles multi-step missions with branching
- NEXT_RUN.md resume mechanism tested end-to-end

### v0.3.0 — Expansion
- L6 Runtime Self-Optimization: AKAR tunes its own skill selection
- Multi-model routing (cost-aware fallback chains)
- Project memory scoped correctly across sessions
- Autopilot mode with guardrails (not just overnight runs)
- External eval harness for regression testing

### v0.5.0 — Production Candidate
- Battle-tested against 50+ real engineering missions
- Learning Intelligence has accumulated 20+ verified patches
- Doctor can recover from all documented failure classes
- Safety Policy formally audited
- Performance baseline: cost/mission, requests/mission tracked

### v1.0.0 — Battle-Tested Release
- L7 Self-Evolving Engineering OS: AKAR proposes its own RFC drafts
- Public API surface stable and documented
- Zero known verification trust gaps
- All v0.x RFCs either accepted, rejected, or deferred with rationale
- External adoption possible without deep internal knowledge

## Principles

- Each version must pass its own eval suite before the next begins.
- No version skipping. Optimization before expansion.
- RFCs gate all major design changes from v0.1.1 onward.
