---
id: FT-025
title: Benchmarks
phase: 3
status: complete
depends-on:
- FT-024
adrs:
- ADR-012
- ADR-018
tests:
- TC-180
domains:
- observability
domains-acknowledged:
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  observability: Benchmarks produce timing metrics and score comparisons but are not a runtime observability surface. ADR-018 (testing strategy) governs the benchmark approach; no dedicated observability ADR is needed.
---

Benchmark suite that validates the core value proposition: LLM context assembled from the knowledge graph produces better results than naive approaches.

### Benchmark Runner

A benchmark runner binary at `benchmarks/runner/` executes benchmark tasks and scores results against rubric files.

### Benchmark Tasks

Three benchmark tasks validate the quality of assembled context:

- **TC-030** — Raft election: can the LLM implement Raft leader election from the context bundle?
- **TC-031** — Front-matter parser: can the LLM implement a parser from the spec?
- **TC-032** — Context bundle assembly: can the LLM assemble a context bundle correctly?

Each task has a rubric file and golden result baseline in `benchmarks/`.

### Performance Invariants

The benchmark suite validates timing invariants:

| Operation | Target |
|---|---|
| Parse 200 files | < 200ms |
| Centrality on 200 nodes | < 100ms |
| BFS depth 2 on 500 edges | < 50ms |

### Exit Criteria

TC-030, TC-031, TC-032 each pass: `score(product) >= 0.80` and `delta_vs_naive >= 0.15`. Benchmark suite passes all timing invariants on a Raspberry Pi 5.
