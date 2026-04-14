## Overview

Product builds an in-memory directed graph from YAML front-matter on every CLI invocation. The graph links features, architectural decisions (ADRs), and test criteria through typed edges, enabling dependency ordering, transitive context assembly, structural importance ranking, and change impact analysis. The graph can also be exported as RDF Turtle for use with external SPARQL tooling. Because the graph is always derived from the current file state, it cannot become stale or diverge from the documents it represents (ADR-003).

## Tutorial

### Step 1: Inspect the graph

Run a health check to see the current state of your knowledge graph:

```bash
product graph check
```

If everything is consistent you will see exit code 0. Broken links produce exit code 1; warnings (orphaned artifacts, missing test coverage) produce exit code 2.

### Step 2: Explore feature dependencies

Create two features where one depends on the other. Given a feature file `docs/features/FT-002-rdf-projection.md` with front-matter:

```yaml
---
id: FT-002
title: RDF Projection
depends-on: [FT-001]
---
```

View the dependency tree:

```bash
product feature deps FT-002
```

This prints the full transitive dependency chain for FT-002.

### Step 3: Find the next feature to implement

```bash
product feature next
```

Product performs a topological sort on the `depends-on` DAG and returns the first feature whose predecessors are all complete and whose own status is not yet complete.

### Step 4: Assemble a context bundle

Get the direct context for a feature (depth 1, the default):

```bash
product context FT-001
```

Now get transitive context at depth 2, which follows edges through intermediate nodes:

```bash
product context FT-001 --depth 2
```

The depth-2 bundle includes ADRs and tests reachable through dependent features, giving an agent or engineer a broader picture of the implementation landscape.

### Step 5: Rank decisions by structural importance

```bash
product graph central
```

This computes betweenness centrality for all ADR nodes and prints the top 10 ranked by structural importance. ADRs that bridge many features rank highest.

### Step 6: Analyse change impact

Before modifying or superseding a decision, check the blast radius:

```bash
product impact ADR-002
```

This traverses the reverse graph from ADR-002 and reports all directly and transitively affected features and test criteria.

### Step 7: Export the graph as RDF Turtle

```bash
product graph rebuild
```

This writes `index.ttl` with the full graph in Turtle format, including betweenness centrality scores on ADR nodes. You can then query it with any SPARQL 1.1 tool.

## How-to Guide

### Find which features are blocked

Run topological sort and look for features whose predecessors are incomplete:

1. Run `product feature next` to see the highest-priority unblocked feature.
2. Run `product feature deps FT-XXX` on a specific feature to see its full dependency chain and identify which predecessors are incomplete.

### Get transitive context for a complex feature

1. Run `product context FT-XXX --depth 2` to include ADRs, tests, and features reachable within two hops.
2. If the bundle is very large (>50 nodes), Product warns on stderr. Consider narrowing scope or using `--depth 1`.

### Identify foundational decisions

1. Run `product graph central` for the top-10 ADRs by betweenness centrality.
2. Run `product graph central --top 5` to limit results.
3. Run `product graph central --all` for the full ranked list.

### Check the blast radius before superseding an ADR

1. Run `product impact ADR-XXX` to see all affected features and tests.
2. Pay attention to passing tests in the impact summary — these may be invalidated.
3. When you run `product adr status ADR-XXX superseded --by ADR-YYY`, Product automatically prints the impact summary before committing the status change.

### Query the graph with SPARQL

1. Run `product graph rebuild` to produce `index.ttl`.
2. Run a SPARQL query:

```bash
product graph query "SELECT ?adr WHERE { ft:FT-001 pm:implementedBy ?adr }"
```

3. Use the `pm:` ontology prefix for graph relationships and `ft:`, `adr:`, `tc:` prefixes for artifact IRIs.

### Detect dependency cycles

1. Run `product graph check`.
2. If a cycle exists in the `depends-on` DAG, Product exits with code 1 and names the features involved.

### Use graph health in CI

Add `product graph check` as a pipeline step:

- Exit code 0: clean graph, pipeline passes.
- Exit code 1: errors (broken links, cycles), pipeline fails.
- Exit code 2: warnings only (orphans, coverage gaps), pipeline passes unless you choose to be strict.

To fail only on hard errors and tolerate warnings:

```bash
product graph check || [ $? -eq 2 ]
```

## Reference

### Edge types

| Edge | From | To | Description |
|---|---|---|---|
| `implementedBy` | Feature | ADR | Feature is governed by this decision |
| `validatedBy` | Feature | TestCriterion | Feature is verified by this test |
| `testedBy` | ADR | TestCriterion | Decision is verified by this test |
| `supersedes` | ADR | ADR | This decision replaces another |
| `depends-on` | Feature | Feature | Implementation dependency |

The reverse of every edge is implicit and used by `product impact`.

### Graph algorithms

| Algorithm | Command | Purpose |
|---|---|---|
| Topological sort (Kahn's) | `product feature next`, `product feature deps` | Correct implementation ordering over the `depends-on` DAG |
| BFS to depth N | `product context --depth N` | Transitive context assembly |
| Betweenness centrality (Brandes') | `product graph central` | Structural importance ranking of ADR nodes |
| Reverse-graph BFS | `product impact` | Change impact analysis |

### Commands and flags

#### `product graph check`

Validates graph consistency. Reports broken links, cycles, orphans, and missing test coverage.

| Exit code | Meaning |
|---|---|
| 0 | Clean graph |
| 1 | Errors (broken links, cycles, malformed front-matter) |
| 2 | Warnings only (orphans, coverage gaps) |

#### `product graph rebuild`

Exports the knowledge graph as RDF Turtle to `index.ttl`. Betweenness centrality scores are included on ADR nodes.

#### `product graph central`

Ranks ADR nodes by betweenness centrality.

| Flag | Description |
|---|---|
| `--top N` | Show top N results (default: 10) |
| `--all` | Show all ADRs ranked |

Output columns: Rank, ID, Centrality, Title.

#### `product graph query "<SPARQL>"`

Executes a SPARQL 1.1 query against the in-memory graph (via embedded Oxigraph).

#### `product context FT-XXX`

Assembles a context bundle from the seed feature.

| Flag | Description |
|---|---|
| `--depth N` | BFS traversal depth (default: 1) |
| `--order id` | Sort ADRs by ID instead of centrality |

At depth 1, only direct ADRs and tests are included. At depth 2+, transitive neighbors are followed. A warning is emitted if the bundle exceeds 50 nodes.

Duplicate nodes reachable via multiple paths appear once, at their first-encountered position. The bundle header lists all included artifact IDs.

#### `product impact <ID>`

Computes reverse-graph reachability from the given artifact (feature, ADR, or test criterion). Reports direct and transitive dependents, with a summary highlighting passing tests that may be invalidated.

#### `product feature next`

Returns the first feature in topological order whose status is not complete and whose predecessors are all complete.

#### `product feature deps FT-XXX`

Prints the full transitive dependency tree for a feature.

### RDF ontology prefixes

| Prefix | IRI |
|---|---|
| `pm:` | `https://product-meta/ontology#` |
| `ft:` | `https://product-meta/feature/` |
| `adr:` | `https://product-meta/adr/` |
| `tc:` | `https://product-meta/test/` |

### Key properties in the Turtle export

| Property | Domain | Range | Description |
|---|---|---|---|
| `pm:title` | Any artifact | String | Artifact title |
| `pm:phase` | Feature | Integer | Phase label |
| `pm:status` | Any artifact | `pm:Status` | Current status |
| `pm:implementedBy` | Feature | ADR | Forward edge |
| `pm:validatedBy` | Feature | TestCriterion | Forward edge |
| `pm:testedBy` | ADR | TestCriterion | Forward edge |
| `pm:dependsOn` | Feature | Feature | Dependency edge |
| `pm:betweennessCentrality` | ADR | Float | Computed centrality score |

### `product graph stats` output

Extended with centrality summary:

```
ADR centrality: mean=0.41, max=0.847 (ADR-001), min=0.003 (ADR-007)
Structural hubs (centrality > 0.5): ADR-001, ADR-002, ADR-006
```

## Explanation

### Why the graph is rebuilt on every invocation

Product never persists the graph to disk (ADR-003). On every command, it reads all artifact files, parses their YAML front-matter, and constructs the graph in memory. This eliminates an entire class of bugs — stale caches, corrupted indexes, synchronization failures between the graph store and the files. At the scale Product targets (under 500 artifacts), full reconstruction takes well under 100ms.

The `index.ttl` file produced by `product graph rebuild` is an export artifact for external SPARQL tools. Product never reads it as input.

### Front-matter as the single source of truth

Every artifact declares its identity and outgoing edges in YAML front-matter (ADR-002). This means each file is self-describing — open any file and you immediately see its place in the graph. Adding a link between a feature and an ADR is a one-line front-matter change, visible in git diffs and code review.

### Topological sort vs. phase labels

Phase labels (`phase: 1`, `phase: 2`) express human intent about grouping and milestones. The `depends-on` edge expresses structural truth about what must be complete before implementation can begin. Topological sort on the dependency DAG produces the correct implementation order regardless of phase labels.

When they disagree — a phase-1 feature depending on a phase-2 feature — `product graph check` reports a warning. The developer decides whether to fix the dependency declaration or the phase label.

### Why betweenness centrality, not PageRank

Betweenness centrality measures how often a node lies on the shortest path between other nodes — it identifies structural bridges. An ADR with high betweenness connects otherwise loosely related features; understanding it is essential before working on any of those features.

PageRank models random-walk importance, which assumes edges represent influence or endorsement. Product's edges are structural dependencies, not endorsements. Betweenness centrality is the correct metric for identifying decisions that an engineer must read first (ADR-012).

Centrality is computed from the graph structure, not declared in front-matter. It cannot drift or become stale — it is always consistent with the current graph.

### Impact analysis and the reverse graph

For every forward edge A -> B, the reverse graph contains B -> A. BFS on the reverse graph from any node returns everything that depends on it. This is the foundation of `product impact`: given an ADR, find all features that reference it, all tests that validate it, and all features that transitively depend on those features.

The practical value is knowing the blast radius of a change before making it. When superseding an ADR, Product automatically runs impact analysis and shows the result, so the developer sees which passing tests may be invalidated and which features need re-evaluation.

### Embedded SPARQL via Oxigraph

Product uses Oxigraph as an embedded, in-process SPARQL 1.1 engine (ADR-008). There is no external service to run — Oxigraph is a Rust-native library that loads the graph from memory and executes queries without touching disk. This preserves Product's single-binary, no-external-dependencies design while providing the full SPARQL 1.1 query vocabulary (SELECT, CONSTRUCT, ASK, DESCRIBE).

### File write safety

Graph export and checklist generation write files to disk. All writes use atomic temp-file-plus-rename to prevent torn writes, and an advisory lock on `.product.lock` serializes concurrent Product invocations (ADR-015). Read-only commands never acquire the lock.
