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
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  observability: Benchmarks produce timing metrics and score comparisons but are not a runtime observability surface. ADR-018 (testing strategy) governs the benchmark approach; no dedicated observability ADR is needed.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
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

---

## Description

The benchmark suite validates Product's core value proposition empirically: that LLM context assembled from the knowledge graph produces better implementation results than naive approaches. It also validates timing invariants for graph algorithms. The suite runs via `cargo bench` and consists of four benchmarks in `benches/graph_bench.rs` covering graph construction, BFS, centrality, and file parsing. Task-based LLM benchmarks (TC-030, TC-031, TC-032) score agent outputs against rubric files to measure the quality delta between Product context bundles and naive approaches (ADR-018).

## Functional Specification

### Inputs

- **`cargo bench`**: runs all four benchmark harnesses in `benches/graph_bench.rs`
- **Benchmark task fixtures**: rubric files and golden result baselines in `benchmarks/`; each task has a fixed prompt and a structured rubric
- **Task descriptions** (TC-030, TC-031, TC-032): Raft leader election implementation, front-matter parser implementation, context bundle assembly
- **LLM benchmark runner** (`benchmarks/runner/`): executable that runs tasks against a configured model and scores outputs

### Outputs

- **`cargo bench` output**: timing results per benchmark in Criterion format to stdout; failures if any benchmark panics
- **Score reports**: per-task scores in the form `score(product)` and `delta_vs_naive`; written to stdout by the benchmark runner
- **Pass/fail determination**: each LLM task passes when `score(product) >= 0.80` and `delta_vs_naive >= 0.15`

### State

- Rubric files and golden baselines in `benchmarks/` are committed to the repository and are the stable reference for scoring.
- Timing invariant thresholds are encoded directly in benchmark assertions, not in `product.toml`.
- Benchmark results are not automatically committed to `metrics.jsonl` — they are run and interpreted manually or in CI.

### Behaviour

1. **Timing benchmarks** (`cargo bench`): four Criterion benchmarks exercise the graph algorithms on representative data sizes (200 files, 200 nodes, 500 edges). Each benchmark measures mean wall-clock time and asserts it is within the target:
   - Parse 200 files: < 200ms
   - Betweenness centrality on 200 nodes: < 100ms
   - BFS at depth 2 on 500 edges: < 50ms
2. **LLM task benchmarks**: the runner in `benchmarks/runner/` sends each task's context bundle (product-assembled) and a naive context (raw file dump) to a configured model. It scores each response against the rubric and computes `delta_vs_naive`.
3. **Scoring**: rubric files define weighted criteria for each task. The runner evaluates model output against each criterion and sums the weighted scores. `score(product)` is the mean across three rubric criteria; `delta_vs_naive` is the score difference.

### Invariants

- All timing benchmarks must complete within their target on a Raspberry Pi 5 (the reference hardware for performance validation).
- TC-030, TC-031, and TC-032 each require `score(product) >= 0.80` and `delta_vs_naive >= 0.15` to pass.
- The rubric files and golden baselines are never auto-modified by the benchmark runner — they are stable references.

### Error handling

- If any timing benchmark panics or exceeds its time limit, `cargo bench` exits non-zero. TC-180 treats this as a failure.
- If the LLM runner cannot reach the model API (network error, timeout), it exits non-zero with an error message; the task score is not recorded.
- Malformed rubric files cause the runner to exit 1 with a schema validation error.

### Boundaries

- The benchmark suite validates algorithmic performance and context quality. It does not validate CLI command correctness (that is the integration test suite's responsibility).
- LLM benchmark results depend on the configured model and its temperature. Results are not deterministic across model versions.
- The timing benchmarks use Criterion's statistical harness; wall-clock variance across runs is expected and Criterion reports confidence intervals accordingly.

## Out of scope

- Continuous benchmark recording in CI on every commit (benchmarks are run explicitly, not on every push)
- A/B testing across multiple models (the runner is configured for one model at a time)
- LLM benchmark results stored in `metrics.jsonl` (metrics tracks graph health metrics; benchmark scores are separate)
- Benchmarking CLI startup time or I/O throughput (only graph algorithm performance is benchmarked)
