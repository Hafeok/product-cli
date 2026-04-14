---
id: FT-030
title: Codebase Onboarding
phase: 5
status: complete
depends-on: []
adrs:
- ADR-027
tests:
- TC-168
- TC-169
- TC-170
- TC-171
- TC-172
- TC-173
- TC-174
- TC-175
- TC-176
- TC-177
- TC-178
- TC-356
- TC-357
- TC-358
- TC-359
- TC-360
- TC-361
- TC-362
- TC-363
- TC-364
- TC-365
- TC-366
- TC-367
- TC-368
domains: []
domains-acknowledged: {}
---

Codebase onboarding discovers load-bearing architectural decisions from an existing codebase and produces a minimum viable knowledge graph. See ADR-027 for the full specification.

### The Problem

Most codebases have no formal architecture documentation. The decisions are baked into patterns — error handling conventions, module boundaries, dependency choices — that were made over years but never written down. An agent (or new engineer) modifying this codebase has no way to know which patterns are load-bearing and which are incidental.

### The Three Failure Modes

Naive approaches to onboarding fail in predictable ways:

1. **The archaeology dump** — LLM scans the codebase, generates 40 ADRs with no rationale, no rejected alternatives, no evidence. The graph is populated but useless. *Avoided by: LLM proposes candidates, not ADRs. Human triage is required.*

2. **The perfectionism trap** — every ADR must be complete before proceeding. Onboarding takes six months. *Avoided by: the "enrich later" principle. Confirmed candidates with empty rationale are valid. Gap analysis drives incremental enrichment.*

3. **The wrong unit** — starting from directory structure produces ADRs that map to files, not decisions. *Avoided by: signal types that cross module boundaries by design.*

### The Three Phases

```bash
# Phase 1: Scan — LLM detects decision candidates from code patterns
product onboard scan ./src --output candidates.json

# Phase 2: Triage — team confirms, enriches, merges, or rejects
product onboard triage candidates.json --interactive

# Phase 3: Seed — confirmed candidates become ADR files + feature stubs
product onboard seed triaged.json

# Post-onboarding: gap analysis drives incremental growth
product gap check --all
```

### Signal Types

The scan prompt looks for six signal types — patterns that suggest deliberate architectural choices:

| Signal | What the LLM observes | Why it's load-bearing |
|---|---|---|
| **Consistency** | Same pattern repeated across the codebase | Violating it breaks an implicit contract |
| **Boundary** | Only certain modules access a resource | Violating it bypasses safety guarantees |
| **Constraint** | All X comes from Y, never from Z | Violating it breaks deployment/runtime assumptions |
| **Convention** | Different treatment for different categories | Violating it leaks internals or breaks APIs |
| **Absence** | Something is deliberately *not* used | Introducing it would conflict with the chosen approach |
| **Dependency** | A foundational dependency is pinned with explanation | Upgrading it would break an assumption |

None of these map to files or directories. They map to decisions that manifest *across* files.

### How It Differs from Migration (FT-020)

Migration converts **existing documents** (PRDs, ADR docs) into structured artifacts. The input is already prose that describes decisions.

Onboarding converts **existing code** into structured artifacts. The input is source files where decisions are implicit in patterns, not stated in prose. The LLM detects signals; the team provides meaning.

A team may use both: migrate existing docs with `product migrate`, then onboard the codebase with `product onboard` to find decisions that were never documented.

### Configuration

```toml
[onboard]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-candidates = 30             # upper bound — prevents archaeology dump
confidence-threshold = "low"    # include everything, let triage filter
chunk-strategy = "import-graph" # split large codebases by module clusters
evidence-validation = true      # post-validate cited files and lines exist
```

### Design Principles

1. **Find load-bearing walls, not everything.** The question is "what would break if an agent didn't know about it?" — not "document the architecture."
2. **LLM detects, humans decide.** The LLM proposes decision candidates with evidence. The team confirms and enriches. No ADR enters the graph without human triage.
3. **Good enough beats complete.** Confirmed candidates with empty rationale are valid. Gap analysis (FT-029) drives incremental enrichment. Onboarding that finishes in a day beats onboarding that takes six months.
4. **Evidence-grounded, not hallucinated.** Every candidate must cite specific files and line numbers. Post-validation catches fabricated evidence before triage.

### Exit Criterion

Onboarding is "done enough" when:

1. `product gap check --all` produces no G005 (architectural contradiction) — captured decisions are internally consistent
2. Every seeded ADR has evidence that post-validates (cited files and lines exist)
3. `product graph check` exits 0 or 2 (warnings only)

Coverage gaps are expected — they're tracked in `gaps.json` and addressed over time. **Onboarding finds the load-bearing walls. Gap analysis fills in the rest.**

---