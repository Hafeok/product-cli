## Overview

Product includes a benchmark suite that validates its core value proposition: LLM context assembled from the knowledge graph produces measurably better implementation results than naive approaches. The suite has two components — **performance benchmarks** that enforce timing invariants on graph operations, and **LLM quality benchmarks** that score context bundle effectiveness against rubric-based criteria. Together they ensure Product remains both fast and useful.

## Tutorial

### Running the performance benchmarks

The fastest way to see the benchmark suite in action is to run the built-in Criterion benchmarks:

```bash
cargo bench
```

This executes four benchmarks that measure core graph operations:

1. Parsing 200 feature/ADR/TC files
2. Betweenness centrality computation on 200 nodes
3. BFS traversal at depth 2 on 500 edges
4. Graph rebuild from scratch

Each benchmark prints timing statistics including mean, median, and standard deviation. Criterion also detects regressions automatically if previous results exist.

### Understanding the output

A typical run produces output like:

```
parse_200_files         time:   [142.3 ms 145.1 ms 148.2 ms]
centrality_200_nodes    time:   [61.2 ms 63.8 ms 66.1 ms]
bfs_depth2_500_edges    time:   [28.4 ms 30.1 ms 31.9 ms]
graph_rebuild           time:   [89.7 ms 92.3 ms 95.1 ms]
```

The three values in brackets are the lower bound, estimate, and upper bound of the confidence interval. If a previous baseline exists, Criterion also reports whether performance changed.

### Running the LLM quality benchmarks

The LLM benchmark runner is a separate binary in `benchmarks/runner/`. It measures whether Product's context bundles actually improve LLM output quality compared to naive context or no context at all.

1. Build the runner:

   ```bash
   cd benchmarks/runner
   cargo build --release
   ```

2. Run all benchmark tasks:

   ```bash
   ./target/release/runner
   ```

3. View the results in `benchmarks/results/latest/results.md`.

## How-to Guide

### Check whether performance invariants hold

1. Run `cargo bench`.
2. Compare the reported timings against these targets:

   | Operation | Target |
   |---|---|
   | Parse 200 files | < 200 ms |
   | Centrality on 200 nodes | < 100 ms |
   | BFS depth 2 on 500 edges | < 50 ms |

3. If any benchmark exceeds its target, investigate the relevant module (`parser.rs`, `graph.rs`, or `context.rs`).

### Detect performance regressions between commits

1. Run `cargo bench` on the baseline commit. Criterion saves results automatically.
2. Check out the new commit.
3. Run `cargo bench` again. Criterion compares against the saved baseline and reports any regressions with percentage change.

### Run a single LLM benchmark task

1. Navigate to `benchmarks/runner/`.
2. Run the runner targeting a specific task:

   ```bash
   ./target/release/runner --task task-001-raft-leader-election
   ```

3. The runner executes the task under three conditions (`none`, `naive`, `product`), scores the output against the rubric, and writes results to `benchmarks/results/`.

### Interpret LLM benchmark results

1. Open `benchmarks/results/latest/results.json`.
2. For each task, check two pass criteria:
   - `score(product) >= 0.80` — the absolute quality threshold.
   - `delta_product_vs_naive >= 0.15` — Product must add measurable value over naive context.
3. Both conditions must hold for a task to pass. A high product score where naive also scores high does not count.

### Add a new benchmark task

1. Create a directory under `benchmarks/tasks/` following the naming convention `task-NNN-description/`.
2. Write `prompt.md` — the implementation request, stated without embedded context.
3. Write `rubric.md` — binary-scored criteria with weights. Each criterion must be answerable with YES or NO.
4. Register the task's TC in `docs/tests/` with appropriate front-matter.
5. Run the benchmark runner to verify the task executes correctly under all three conditions.

## Reference

### Performance benchmarks

**Command:**

```bash
cargo bench
```

**Benchmark location:** `benches/graph_bench.rs`

**Timing invariants:**

| Benchmark | Operation | Target | Validates |
|---|---|---|---|
| `parse_200_files` | Parse 200 feature/ADR/TC files | < 200 ms | `parser.rs` performance |
| `centrality_200_nodes` | Betweenness centrality on 200 nodes | < 100 ms | `graph.rs` Brandes' algorithm |
| `bfs_depth2_500_edges` | BFS at depth 2 on 500 edges | < 50 ms | `graph.rs` / `context.rs` traversal |

**Target hardware:** Raspberry Pi 5. Timings are calibrated to this baseline — faster hardware will pass more easily.

### LLM quality benchmarks

**Runner location:** `benchmarks/runner/src/main.rs`

**Task directory structure:**

```
benchmarks/tasks/task-NNN-description/
  prompt.md       # Implementation request (no embedded context)
  rubric.md       # Binary-scored criteria with weights
```

**Three test conditions per task:**

| Condition | Context provided |
|---|---|
| `none` | No context beyond the prompt |
| `naive` | Full PRD + ADRs concatenated |
| `product` | Output of `product context FT-XXX --depth 2` |

**Scoring:** Each rubric criterion is scored by a separate LLM call with a binary YES/NO question. Final score = sum of (satisfied criteria x weight) / sum of (all criteria x weight). Each condition runs N=5 times at temperature=0; the reported score is the mean.

**Pass thresholds:**

| Criterion | Threshold |
|---|---|
| Absolute quality | `score(product) >= 0.80` |
| Incremental value | `score(product) - score(naive) >= 0.15` |

**Result output location:** `benchmarks/results/<date>/`

**Result format (JSON):**

```json
{
  "run_date": "2026-04-11T09:00:00Z",
  "product_version": "0.1.0",
  "model": "claude-sonnet-4-6",
  "runs_per_condition": 5,
  "tasks": [
    {
      "id": "task-001-raft-leader-election",
      "tc": "TC-030",
      "conditions": {
        "none":    { "mean": 0.41, "scores": [0.39, 0.41, 0.44, 0.40, 0.41] },
        "naive":   { "mean": 0.63, "scores": [0.61, 0.65, 0.63, 0.62, 0.64] },
        "product": { "mean": 0.87, "scores": [0.85, 0.89, 0.86, 0.88, 0.87] }
      },
      "delta_product_vs_naive": 0.24,
      "pass": true
    }
  ],
  "aggregate": {
    "mean_product_score": 0.87,
    "mean_delta_vs_naive": 0.21,
    "tasks_passing": 3,
    "tasks_total": 3
  }
}
```

**A `results.md` human-readable summary** is also generated alongside the JSON.

### Initial task set

| TC | Task | Target feature | Key rubric dimension |
|---|---|---|---|
| TC-030 | Raft leader election | FT-001 | Architecture compliance (openraft, RDF) |
| TC-031 | Front-matter parser | FT-Product-001 | Robustness (no panics, error codes) |
| TC-032 | Context bundle assembly | FT-Product-002 | Correctness (depth, dedup, ordering) |

### Run cadence

LLM benchmarks are **not run in CI**. They are triggered manually:

- On release candidates
- After context bundle format changes
- Monthly for trend tracking

Performance benchmarks (`cargo bench`) can be run at any time and are suitable for CI.

## Explanation

### Why two kinds of benchmarks?

Product makes two claims: that its graph operations are fast enough for interactive use, and that its context bundles are meaningfully better than naive alternatives. These are independent claims requiring independent validation.

Performance benchmarks run in milliseconds and catch regressions on every commit. LLM benchmarks are expensive (multiple API calls per task) and slow, but they validate the product's fundamental value proposition. Neither can substitute for the other.

### Why binary rubric scoring?

Holistic judgments like "is this good Rust?" produce high variance between LLM scoring calls. Binary questions like "does it use openraft?" produce consistent scores. The rubric-based approach with weighted binary criteria reduces scorer variance and produces a number that can be tracked reliably over time (ADR-018).

### Why the delta threshold matters

The `delta_product_vs_naive >= 0.15` threshold is the most important pass criterion. A product score of 0.90 is meaningless if naive context also scores 0.87 — Product would be providing no incremental value over simply concatenating the PRD and ADRs. The delta threshold enforces that Product's graph-based context assembly earns its place in the workflow.

### Why N=5 at temperature=0?

Temperature=0 is not fully deterministic across all models, but variance is small enough that the mean of 5 runs produces a stable score. This is a deliberate variance-reduction choice that balances cost (5 API calls per condition per task) against statistical reliability (ADR-018).

### How benchmarks relate to graph capabilities

The performance benchmarks directly validate the algorithms specified in ADR-012: Brandes' algorithm for betweenness centrality, BFS for context depth traversal, and the file parser that feeds the graph. The timing targets are calibrated to ensure these operations remain interactive even on constrained hardware (Raspberry Pi 5).

The LLM benchmarks validate the end-to-end value chain: the graph model (ADR-012) feeds context assembly, which feeds context bundles, which feed an LLM. If any link in that chain degrades, the benchmark scores drop. A declining `delta_product_vs_naive` over releases is the primary signal that context bundle quality is regressing.

### Results as trend data

Benchmark results are committed to the repository under `benchmarks/results/`. The `latest` symlink always points to the most recent run. The trend across runs — not any single result — is the primary signal. A single benchmark failure may reflect model variance; a declining trend across releases indicates a real problem in context bundle quality.
