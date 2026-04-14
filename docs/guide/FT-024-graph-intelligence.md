## Overview

Graph Intelligence provides structural analysis capabilities that go beyond simple navigation of the knowledge graph. It surfaces betweenness centrality rankings to identify the most architecturally important decisions, executes SPARQL 1.1 queries over the graph for ad-hoc exploration, and reports graph-wide statistics including artifact counts, link density, and formal block coverage. These capabilities turn the implicit structure of the knowledge graph into explicit, actionable signals for engineers and LLM agents.

## Tutorial

### Discovering your most important decisions

Every knowledge graph has a few foundational ADRs that connect many features together. Centrality analysis finds them automatically.

1. Run the centrality command to see the top-10 ADRs ranked by structural importance:

   ```bash
   product graph central
   ```

   You will see output like:

   ```
   Rank  ID       Centrality  Title
   1     ADR-001  0.847       Rust as Implementation Language
   2     ADR-002  0.731       openraft for Cluster Consensus
   3     ADR-006  0.612       Oxigraph for RDF Projection
   4     ADR-003  0.445       Event Log Schema
   5     ADR-009  0.201       CI Exit Codes
   ```

   ADRs with high centrality sit on many shortest paths between other nodes — they are structural bridges that an engineer must understand before working on most features.

2. Narrow or expand the list:

   ```bash
   product graph central --top 3    # just the top 3
   product graph central --all      # every ADR, ranked
   ```

### Querying the graph with SPARQL

SPARQL lets you ask arbitrary questions about the graph structure.

1. Find all ADRs linked to a specific feature:

   ```bash
   product graph query "SELECT ?adr WHERE { ft:FT-001 pm:implementedBy ?adr }"
   ```

2. Find features that have no test criteria:

   ```bash
   product graph query "SELECT ?f WHERE { ?f a pm:Feature . FILTER NOT EXISTS { ?f pm:validatedBy ?t } }"
   ```

3. Filter features by phase:

   ```bash
   product graph query "SELECT ?f WHERE { ?f pm:phase 1 }"
   ```

### Checking graph health at a glance

The stats command gives you a quick snapshot of your knowledge graph.

```bash
product graph stats
```

This prints artifact counts, link density, centrality summary (mean, max, min), and phi (formal block coverage) across test criteria.

## How-to Guide

### Identify which ADRs to read first for a new feature

1. Run `product graph central` to see the top-ranked ADRs.
2. Cross-reference with the feature's context bundle: `product context FT-XXX`.
3. ADRs in the bundle are ordered by centrality descending by default — read them top to bottom for the most efficient onboarding path.

### Find structural hubs in your architecture

1. Run `product graph stats`.
2. Look for the "Structural hubs" line, which lists all ADRs with centrality above 0.5:
   ```
   ADR centrality: mean=0.41, max=0.847 (ADR-001), min=0.003 (ADR-007)
   Structural hubs (centrality > 0.5): ADR-001, ADR-002, ADR-006
   ```
3. These hubs are the decisions most likely to cause widespread impact if changed. Use `product impact ADR-XXX` before modifying any of them.

### Run ad-hoc SPARQL queries for custom analysis

1. Ensure the graph's TTL export is current:
   ```bash
   product graph rebuild
   ```
2. Write a SPARQL 1.1 query using the graph's namespace prefixes (`ft:`, `adr:`, `tc:`, `pm:`).
3. Execute it:
   ```bash
   product graph query "SELECT ?feature ?adr WHERE { ?feature pm:implementedBy ?adr } ORDER BY ?feature"
   ```

### Use centrality data in TTL exports

1. Run `product graph rebuild` to regenerate `index.ttl`.
2. Centrality scores are included as properties on ADR nodes in the TTL output.
3. External tools that consume `index.ttl` can use these scores for their own ranking or visualization.

### Override ADR ordering in context bundles

By default, ADRs within context bundles are ordered by centrality descending — the most structurally important decisions appear first. To switch to ID-based ordering:

```bash
product context FT-XXX --order id
```

## Reference

### Commands

#### `product graph central`

Ranks ADRs by betweenness centrality using Brandes' algorithm.

```
product graph central [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--top N` | `10` | Number of top-ranked ADRs to display |
| `--all` | off | Display the full ranked list |

**Output format:**

```
Rank  ID       Centrality  Title
1     ADR-001  0.847       Rust as Implementation Language
...
```

Centrality values are normalized to the range [0.0, 1.0].

**Exit codes:** `0` on success, `1` on error.

#### `product graph query`

Executes a SPARQL 1.1 query against the knowledge graph using embedded Oxigraph (ADR-008).

```
product graph query "<SPARQL query string>"
```

Supports SELECT, CONSTRUCT, ASK, and DESCRIBE query forms. The graph is loaded from the in-memory representation on each invocation — no persistent store is used (ADR-003).

**Exit codes:** `0` on success, `1` on error (e.g., malformed SPARQL).

#### `product graph stats`

Prints aggregate statistics about the knowledge graph.

```
product graph stats
```

Output includes:
- Artifact counts (features, ADRs, test criteria)
- Link density
- Centrality summary: mean, max (with ID), min (with ID)
- Structural hubs (ADRs with centrality > 0.5)
- Phi (formal block coverage across test criteria)

**Exit codes:** `0` on success, `1` on error.

#### `product graph rebuild`

Regenerates `index.ttl` from the current file state. Centrality scores are included as properties on ADR nodes.

```
product graph rebuild
```

**Exit codes:** `0` on success, `1` on error.

### Context bundle ADR ordering

| `--order` value | Behaviour |
|-----------------|-----------|
| *(default)* | ADRs ordered by betweenness centrality, descending |
| `id` | ADRs ordered by ID, ascending |

### Performance characteristics

| Operation | Target | Graph size |
|-----------|--------|------------|
| Centrality computation | < 100 ms | 200 nodes |
| Impact analysis | < 50 ms | 200 nodes |

The graph is rebuilt from YAML front-matter on every invocation (ADR-003). For repositories with < 200 artifact files, full graph construction completes in < 50 ms.

### Exit codes

Graph intelligence commands follow the standard exit code scheme (ADR-009):

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | Error |
| `2` | Warnings only (for `graph check`) |

### Error output

All errors and warnings are written to stderr. Use `--format json` for machine-parseable structured output (ADR-013).

## Explanation

### Why betweenness centrality and not PageRank?

PageRank models random-walk importance — it assumes edges represent influence or endorsement. In the Product knowledge graph, edges are structural dependencies (`implementedBy`, `validatedBy`, `depends-on`), not endorsements. Betweenness centrality correctly models structural bridging: a node with high betweenness sits on many shortest paths between other nodes, making it a bottleneck that many traversals must pass through. This is exactly the property that makes an ADR "foundational" — it connects otherwise loosely related features (ADR-012).

### Why computed importance instead of manual tagging?

An alternative approach would be to add an `importance: foundational | standard | peripheral` field to ADR front-matter. This was rejected (ADR-012) because manual importance labels drift as the graph evolves. A decision that was peripheral when three features existed may become foundational when twenty features link to it. Centrality is computed from the current graph structure — it cannot drift out of sync because it is derived, not declared.

### Why SPARQL over a custom query language?

SPARQL 1.1 is a W3C standard with existing tooling, documentation, and user knowledge. A bespoke query language would require Product to own documentation and training for a capability that SPARQL already covers. Oxigraph (ADR-008) provides a Rust-native SPARQL 1.1 implementation with no FFI or external service dependencies, making it a natural fit for the single-binary constraint.

### Why no persistent graph store?

The graph is rebuilt from front-matter on every invocation (ADR-003). At the scale Product targets (< 200 artifact files), full graph construction including centrality computation completes in well under 100 ms. A persistent store would introduce a synchronization invariant — the stored graph must always match the files — that is impossible to enforce perfectly when files can be edited outside Product or changed by git operations. A derived graph is always correct by construction.

### How centrality integrates with context bundles

Context bundles order their ADR sections by centrality descending by default. This means an LLM agent reading a bundle top-to-bottom encounters the most structurally important decisions first, giving it architectural grounding before diving into feature-specific decisions. This ordering falls out naturally from the centrality computation and adds no maintenance burden (ADR-012).
