## Overview

Impact analysis lets you see the full blast radius of changing any artifact in the knowledge graph before you commit to the change. `product impact` performs reverse-graph reachability from a target node — traversing every edge type backwards — to report all features, ADRs, and test criteria that depend on it, grouped by hop distance. This is essential for assessing risk before superseding a decision, modifying a shared ADR, or understanding what breaks when a foundational feature changes.

## Tutorial

### See what depends on a decision

Suppose your repository has ADR-002 linked to several features and you are considering superseding it. Start by checking its impact:

```bash
product impact ADR-002
```

The output groups dependents by distance from the target:

```
Impact analysis: ADR-002 — openraft for Cluster Consensus

Direct dependents:
  Features:  FT-001 (in-progress), FT-004 (planned)
  Tests:     TC-002 (unimplemented), TC-003 (unimplemented), TC-007 (passing)

Transitive dependents (via feature dependencies):
  Features:  FT-007 (planned) — depends-on FT-001
  Tests:     TC-011 (unimplemented) — validates FT-007

Summary: 3 features, 4 tests affected. 1 passing test may be invalidated.
```

Read the output from top to bottom:

1. **Direct dependents** are one hop away — features governed by this ADR and tests that validate it.
2. **Transitive dependents** are further out — features that depend on directly-affected features, and their tests.
3. The **summary** highlights passing tests that may break, which are the highest-urgency items.

### Check impact on a feature

Features can be depended on by other features via `depends-on` edges:

```bash
product impact FT-001
```

This shows every feature that transitively depends on FT-001 completing, plus their associated tests and ADRs.

### Check impact on a test criterion

```bash
product impact TC-003
```

This reveals which features and ADRs are connected to a specific test, helping you understand the consequences of removing or rewriting it.

## How-to Guide

### Assess risk before superseding an ADR

1. Run impact analysis on the ADR you plan to supersede:
   ```bash
   product impact ADR-002
   ```
2. Review the summary line — note any passing tests that may be invalidated.
3. If the risk is acceptable, proceed with the supersession:
   ```bash
   product adr status ADR-002 superseded --by ADR-013
   ```
   Product automatically runs impact analysis and prints the summary before committing the status change, so you see the blast radius even if you skip step 1.

### Review impact before implementing a feature

1. Identify the ADRs linked to your feature:
   ```bash
   product context FT-006 --depth 1
   ```
2. For each ADR that you plan to modify or interpret differently, run:
   ```bash
   product impact ADR-012
   ```
3. Check whether other in-progress features share that ADR — coordinate with their implementers if so.

### Find all artifacts affected by a change at any depth

1. Run impact on the artifact you are changing:
   ```bash
   product impact ADR-002
   ```
2. The output already includes transitive dependents (not just direct ones). Reverse-graph BFS follows all five edge types, so no manual depth flag is needed — the full reachable set is always reported.

### Combine impact analysis with graph health checks

1. Run impact analysis to understand what a change affects:
   ```bash
   product impact ADR-005
   ```
2. After making your changes, verify the graph is still healthy:
   ```bash
   product graph check
   product gap check
   product drift check
   ```

## Reference

### Command syntax

```
product impact <ARTIFACT-ID>
```

| Argument | Required | Description |
|---|---|---|
| `ARTIFACT-ID` | Yes | Any valid artifact ID: `FT-XXX`, `ADR-XXX`, or `TC-XXX` |

### Output format

The output has three sections:

1. **Header** — the artifact ID and title.
2. **Direct dependents** — artifacts one reverse-hop from the target, grouped by type (Features, Tests, ADRs). Each entry shows the artifact ID, its current status in parentheses, and title context where relevant.
3. **Transitive dependents** — artifacts reachable through chains of reverse edges beyond the first hop. Each entry includes an annotation explaining the path (e.g., `depends-on FT-001`).
4. **Summary** — total counts of affected features and tests, with a callout for passing tests that may be invalidated.

### Edge types traversed

Impact analysis reverses all five edge types in the knowledge graph:

| Forward edge | From → To | Reverse meaning (used by impact) |
|---|---|---|
| `implementedBy` | Feature → ADR | ADR impacts the features it governs |
| `validatedBy` | Feature → TestCriterion | Test impacts the features it validates |
| `testedBy` | ADR → TestCriterion | Test impacts the ADRs it verifies |
| `supersedes` | ADR → ADR | Superseded ADR impacts its successor chain |
| `depends-on` | Feature → Feature | Feature impacts features that depend on it |

### Automatic impact on ADR supersession

When you run:

```bash
product adr status ADR-XXX superseded --by ADR-YYY
```

Product automatically executes impact analysis on `ADR-XXX` and prints the impact summary to stdout **before** committing the status change. This is not optional — it is built into the supersession workflow.

### Exit codes

`product impact` follows the standard Product exit code model defined in `error.rs`. A successful impact analysis exits with code 0 regardless of how many dependents are found. A non-zero exit indicates the artifact ID was not found or the graph could not be built.

## Explanation

### Why reverse-graph reachability?

The knowledge graph is a directed graph: features point to ADRs via `implementedBy`, features point to tests via `validatedBy`, and features point to other features via `depends-on`. These forward edges answer "what does this feature use?" Impact analysis needs the opposite question: "what uses this artifact?"

Reversing every edge and running BFS from the target node computes the complete set of artifacts that have a forward path *to* the target — everything that depends on it, directly or transitively. This is a standard graph algorithm (O(V+E) time complexity) that requires no additional data structures beyond the graph already in memory (ADR-003).

### Relationship to the derived graph model

Because the graph is rebuilt from front-matter on every CLI invocation (ADR-003), the reverse graph is always consistent with the current file state. There is no risk of a stale impact report caused by a cached graph that does not reflect recent file edits. The cost of rebuilding is negligible at Product's target scale (< 500 nodes).

### Relationship to context assembly

Impact analysis and context assembly (`product context`) are complementary traversals of the same graph:

- **Context assembly** follows edges *forward* from a seed node: "what does this feature need?" It answers the question an implementer asks before starting work.
- **Impact analysis** follows edges *backward* from a target node: "what depends on this artifact?" It answers the question an engineer asks before changing something.

Both use BFS. Context assembly is depth-limited (configurable via `--depth N`). Impact analysis traverses the full reverse-reachable set — there is no depth limit because the entire blast radius matters when assessing change risk.

### Design decisions

The graph-theoretic foundations for impact analysis are specified in ADR-012, which also covers topological sort, BFS context assembly, and betweenness centrality. All four capabilities share the same underlying graph model and were introduced together to avoid multiple graph-model migrations.

The choice to use an embedded, in-process graph (ADR-003) rather than an external store means impact analysis runs in the same process as every other command — no network calls, no service dependencies. The SPARQL projection via Oxigraph (ADR-008) provides an alternative query interface for the same graph, but `product impact` uses direct graph traversal rather than SPARQL because BFS on an adjacency list is simpler and faster than expressing reachability in SPARQL for this use case.
