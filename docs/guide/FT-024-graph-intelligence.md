It looks like the write permission is being blocked. Here's the complete documentation file for FT-024 — Graph Intelligence. You can save it to `docs/guide/FT-024-graph-intelligence.md`:

---

## Overview

Graph Intelligence provides structural analysis capabilities that go beyond simple navigation of the knowledge graph. It surfaces **betweenness centrality** rankings for ADRs, executes **SPARQL 1.1 queries** over the graph, computes **graph statistics**, and performs **impact analysis** to reveal the blast radius of any change. These capabilities help engineers and LLM agents identify the most structurally important decisions, understand transitive dependencies, and assess risk before modifying artifacts.

## Tutorial

This walkthrough introduces the three main graph intelligence commands. You will rank ADRs by structural importance, query the graph with SPARQL, and check the impact of changing a decision.

### Step 1: View graph statistics

Start by getting an overview of your knowledge graph:

```bash
product graph stats
```

Output includes artifact counts, link density, formal coverage percentage, centrality summary, and timing information:

```
Graph Statistics
================
  Features:      12
  ADRs:          26
  Tests:         104
  Total nodes:   142
  Total edges:   387
  Link density:  0.019
  Formal coverage (invariant/chaos): 78%

  Timing:
    Parse:      12.3ms
    Centrality: 4.1ms
    Total:      16.4ms

  ADR centrality: mean=0.041, max=0.847, min=0.003
  Structural hubs (>0.5): ADR-001, ADR-002, ADR-006
```

### Step 2: Rank ADRs by centrality

Find which decisions are structurally foundational:

```bash
product graph central
```

This prints the top 10 ADRs ranked by betweenness centrality — a measure of how many shortest paths pass through each ADR node:

```
RANK   ID         CENTRALITY   TITLE
------------------------------------------------------------
1      ADR-001    0.847        Rust as Implementation Language
2      ADR-002    0.731        openraft for Cluster Consensus
3      ADR-006    0.612        Oxigraph for RDF Projection
...
```

ADRs with high centrality are structural bridges — they connect otherwise loosely related features. Read these first when onboarding to a project.

### Step 3: Check impact before changing a decision

Before superseding or modifying an ADR, see what it affects:

```bash
product impact ADR-002
```

Output shows direct and transitive dependents:

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

### Step 4: Run a SPARQL query

Query the graph directly using SPARQL 1.1:

```bash
product graph query "SELECT ?adr WHERE { ft:FT-001 pm:implementedBy ?adr }"
```

This returns all ADRs linked to FT-001. The query runs against an in-memory Oxigraph store loaded from the knowledge graph.

## How-to Guide

### Find the most important ADRs in your project

1. Run `product graph central` to see the top 10 by betweenness centrality.
2. To see more or fewer results, use `--top N`:
   ```bash
   product graph central --top 5
   ```
3. To see the full ranked list of all ADRs:
   ```bash
   product graph central --all
   ```

### Assess the blast radius of an ADR change

1. Run `product impact ADR-XXX` with the ADR you plan to modify.
2. Review the direct and transitive dependents listed in the output.
3. Pay attention to passing tests that may be invalidated — these are highest-urgency items.
4. If using JSON output for scripting:
   ```bash
   product impact ADR-XXX --format json
   ```

### Automatically show impact when superseding an ADR

No extra step is needed. When you run:

```bash
product adr status ADR-002 superseded --by ADR-013
```

Product automatically runs impact analysis and prints the impact summary before committing the status change.

### Find features with no test criteria using SPARQL

```bash
product graph query "SELECT ?f WHERE { ?f a pm:Feature . FILTER NOT EXISTS { ?f pm:validatedBy ?tc } }"
```

### Filter features by phase using SPARQL

```bash
product graph query "SELECT ?f WHERE { ?f pm:phase 1 }"
```

### Get transitive context for a complex feature

Use the `--depth` flag on the context command to pull in transitive artifacts:

```bash
product context FT-001 --depth 2
```

At depth 2, the bundle includes artifacts reachable through intermediate nodes (e.g., ADRs shared with adjacent features, tests of dependent features). Default depth is 1 (direct links only).

### Control ADR ordering in context bundles

By default, ADRs in context bundles are ordered by betweenness centrality (most important first). To order by ID instead:

```bash
product context FT-001 --order id
```

## Reference

### `product graph central`

Ranks ADRs by betweenness centrality using Brandes' algorithm.

```
product graph central [OPTIONS]
```

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--top` | integer | `10` | Number of results to display |
| `--all` | flag | off | Show all ADRs (overrides `--top`) |

**Output columns:** `RANK`, `ID`, `CENTRALITY` (0.0–1.0), `TITLE`

Results are sorted by centrality descending. Centrality scores are also included in the TTL export produced by `product graph rebuild`.

**Performance:** Completes in < 100ms on graphs with 200 nodes.

---

### `product graph query`

Executes a SPARQL 1.1 query over the knowledge graph using embedded Oxigraph (ADR-008).

```
product graph query "<SPARQL query>"
```

| Argument | Required | Description |
|----------|----------|-------------|
| `query` | yes | A SPARQL 1.1 query string (SELECT, CONSTRUCT, ASK, or DESCRIBE) |

The graph is loaded from front-matter files into an in-memory Oxigraph store on each invocation (ADR-003). No persistent RDF store is used.

**Namespace prefixes available in queries:**

- `ft:` — feature nodes
- `adr:` — ADR nodes
- `tc:` — test criterion nodes
- `pm:` — Product model predicates (`implementedBy`, `validatedBy`, `phase`, etc.)

---

### `product graph stats`

Displays aggregate statistics about the knowledge graph.

```
product graph stats
```

No flags. Output includes:

| Section | Fields |
|---------|--------|
| Artifact counts | Features, ADRs, Tests, Total nodes, Total edges |
| Structure | Link density (edges / possible edges) |
| Coverage | Formal coverage % (invariant/chaos tests with formal blocks) |
| Timing | Parse time, Centrality computation time, Total time |
| Centrality summary | Mean, max, min centrality across ADRs |
| Structural hubs | ADRs with centrality > 0.5 |

---

### `product impact`

Computes the full set of artifacts affected by a change to any artifact.

```
product impact <ID>
```

| Argument | Required | Description |
|----------|----------|-------------|
| `ID` | yes | Any artifact ID (FT-XXX, ADR-XXX, or TC-XXX) |

| Flag | Type | Description |
|------|------|-------------|
| `--format json` | global | Output as structured JSON |

**Output sections:**

- **Direct dependents** — features and tests linked to the artifact
- **Transitive dependents** — artifacts reachable through `depends-on` chains
- **Summary** — total counts and a count of passing tests that may be invalidated

**JSON output structure:**

```json
{
  "seed": "ADR-002",
  "direct_features": ["FT-001", "FT-004"],
  "direct_tests": ["TC-002", "TC-003", "TC-007"],
  "transitive_features": ["FT-007"],
  "transitive_tests": ["TC-011"]
}
```

**Performance:** Completes in < 50ms on graphs with 200 nodes.

---

### `product context` (depth and ordering flags)

These flags on the context command are part of the graph intelligence feature:

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--depth` | integer | `1` | BFS traversal depth from seed node |
| `--order` | string | centrality | Set to `id` for ID-ascending ADR order |

At depth 1, only directly linked artifacts are included. At depth 2+, transitive artifacts (e.g., ADRs of dependent features) are included. Nodes reachable via multiple paths appear once (deduplicated at first encounter).

A warning is emitted to stderr if a depth >= 3 bundle exceeds 50 nodes.

---

### Exit codes

Graph intelligence commands follow the standard exit code scheme (ADR-009):

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error (broken links, cycles, artifact not found) |
| `2` | Warnings only |

## Explanation

### Why betweenness centrality, not PageRank?

PageRank models random-walk importance — it assumes edges represent influence or endorsement. In the Product knowledge graph, edges are structural dependencies (`implementedBy`, `depends-on`, `validatedBy`), not endorsements. Betweenness centrality measures how often a node sits on the shortest path between other nodes, which correctly identifies structural bridges — the decisions that connect otherwise loosely related parts of the architecture (ADR-012).

### Why is the graph rebuilt on every invocation?

Product never persists the graph (ADR-003). The graph is rebuilt from YAML front-matter on every command. At the scale Product targets (< 500 nodes), parsing completes in under 50ms. This eliminates an entire class of cache-invalidation bugs: the graph is always correct by construction, even when files are edited outside Product or changed by git operations.

### Why is centrality used for ADR ordering in bundles?

When an LLM agent receives a context bundle, it reads top-to-bottom. Placing the most structurally important ADRs first ensures the agent encounters foundational decisions before peripheral ones. This ordering is computed, not declared — it automatically adjusts as the graph evolves. Pass `--order id` to override when stable ordering is preferred (ADR-012).

### How does impact analysis work?

Impact analysis constructs the reverse of every directed edge in the knowledge graph, then performs BFS from the target artifact. This finds all nodes that have a path *to* the target in the forward graph — i.e., everything that depends on it. The operation is O(V+E) and trivially derived from the forward graph already in memory (ADR-012).

### SPARQL and Oxigraph

SPARQL queries execute against an in-memory Oxigraph store loaded fresh on each invocation. Oxigraph is a Rust-native SPARQL 1.1 engine — no external service, no persistent storage, no port to configure. The `index.ttl` file produced by `product graph rebuild` is an export artifact for external tooling; the CLI never reads from it (ADR-008, ADR-003).

---

The document is ~280 lines, covers all five Diataxis sections, and every command/flag is verified against the actual clap definitions in `src/main.rs`. Would you like me to try writing the file again, or would you prefer to save it manually?
