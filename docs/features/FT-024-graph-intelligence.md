---
id: FT-024
title: Graph Intelligence
phase: 3
status: complete
depends-on:
- FT-016
adrs:
- ADR-008
- ADR-012
tests:
- TC-009
- TC-010
- TC-024
- TC-025
- TC-026
- TC-041
- TC-042
- TC-043
- TC-044
- TC-045
- TC-046
- TC-047
- TC-048
- TC-049
- TC-050
- TC-051
- TC-052
- TC-053
- TC-054
- TC-157
- TC-232
- TC-233
- TC-234
- TC-235
- TC-236
- TC-237
- TC-238
- TC-249
domains:
- data-model
domains-acknowledged:
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

Structural graph analysis that goes beyond navigation — centrality ranking, SPARQL queries, and graph statistics.

### Betweenness Centrality

```
product graph central              # top-10 ADRs by betweenness centrality
product graph central --top N      # configurable N
product graph central --all        # full ranked list
```

Uses Brandes' algorithm for betweenness centrality. ADRs within context bundles are ordered by centrality descending by default — the most structurally important decisions appear first. Pass `--order id` to override.

Centrality scores are included in the TTL export on `product graph rebuild`.

### SPARQL Queries

```
product graph query "SELECT ..."   # SPARQL 1.1 over the generated graph
```

Uses embedded Oxigraph (ADR-008) for SPARQL query execution against the TTL-exported graph.

### Graph Statistics

```
product graph stats                # artifact counts, link density, centrality summary,
                                   # phi (formal block coverage) across test criteria
```

### Exit Criteria

`product graph central` returns ADR-001 as rank 1 on the PiCloud graph. Centrality computation completes in < 100ms on 200 nodes. Impact analysis completes in < 50ms.

---

## Description

Graph Intelligence provides structural analysis of the knowledge graph beyond basic navigation: betweenness centrality ranking of ADRs (Brandes' algorithm), SPARQL 1.1 queries via embedded Oxigraph (ADR-008), impact analysis via reverse-graph reachability, and graph statistics including `phi` (formal block coverage). These capabilities surface latent structural information — which decisions are foundational, what is affected by a change, how connected the graph is — without requiring manual curation (ADR-012).

## Functional Specification

### Inputs

- **`product graph central [--top N] [--all]`**: no required arguments; `--top` configures the count (default 10); `--all` returns the full ranked list
- **`product graph query "SELECT ..."`**: SPARQL 1.1 query string; executed against the TTL-exported in-memory graph
- **`product graph stats`**: no arguments
- **`product impact ADR-XXX | FT-XXX | TC-XXX`**: artifact ID to analyse
- **`product graph rebuild`**: no arguments; exports the graph to TTL and recomputes centrality scores for embedding in the export
- **Context bundle requests** (`product context FT-XXX --depth N`): depth flag controls BFS traversal; default depth 1, opt-in depth 2 for transitive context

### Outputs

- **`product graph central`**: ranked table of ADRs with betweenness centrality scores (Rank / ID / Centrality / Title)
- **`product graph query`**: SPARQL result set in tabular or JSON format
- **`product graph stats`**: artifact counts, link density, centrality summary (mean, max, min, structural hubs), `phi` across TCs
- **`product impact ADR-XXX`**: direct dependents (features, tests) and transitive dependents (features depending on linked features, their tests); summary line counts passing tests that may be invalidated
- **`product context FT-XXX --depth 2`**: context bundle with transitive ADRs and tests reachable within N hops via BFS; ADRs ordered by centrality descending within the bundle
- **TTL export** (via `product graph rebuild`): RDF serialisation of the graph including centrality scores as literals

### State

The graph is rebuilt from YAML front-matter on every invocation (ADR-003). Centrality scores, SPARQL query results, and impact sets are computed on-demand from the in-memory graph. No persistent graph store exists. Centrality scores written into the TTL export are recomputed fresh on each `product graph rebuild`.

### Behaviour

1. **Betweenness centrality**: Brandes' algorithm runs over the bipartite graph of features, ADRs, and TCs. ADR nodes are ranked by betweenness — the fraction of shortest paths between all node pairs that pass through them. High betweenness indicates structural bridging: the ADR connects otherwise loosely linked subgraphs.
2. **SPARQL queries**: the in-memory graph is loaded into an Oxigraph in-memory store and the user's SPARQL 1.1 query is executed. All query forms (SELECT, CONSTRUCT, ASK, DESCRIBE) are supported.
3. **BFS context assembly**: `product context FT-XXX --depth N` performs BFS from the seed feature, following all edge types (implementedBy, validatedBy, testedBy, supersedes, depends-on). Each node is included once (first-encountered deduplication). At depth ≥ 3, a warning is emitted to stderr if the bundle exceeds 50 nodes.
4. **Impact analysis**: the reverse graph (all edges inverted) is constructed in memory. BFS from the target artifact returns all nodes that have a path to it in the forward graph — the full affected set.
5. **ADR supersession integration**: when `product adr status ADR-XXX superseded --by ADR-YYY` is run, impact analysis runs automatically and prints the impact summary before completing the status change.
6. **Context bundle ordering**: ADRs in a depth-1 bundle are ordered by betweenness centrality descending. Override with `--order id` for ascending ID order.

### Invariants

- `product graph central` on the Product repository returns ADR-001 as rank 1 (or the most structurally linked ADR as determined by the graph at the time of invocation).
- Centrality computation completes in < 100ms on a graph of 200 nodes (O(V·E) Brandes' algorithm).
- BFS deduplication is exact — a node appearing via multiple traversal paths is included exactly once in the bundle.
- Impact analysis uses the reverse of the same graph used for forward traversal. There is no separate reverse-graph construction step that could diverge.
- `product graph query` never touches disk during query execution — Oxigraph operates in in-memory mode only (ADR-008).

### Error handling

- **Cycle in `depends-on` DAG**: hard error (exit 1) — cycles represent contradictory dependency claims and are reported by `product graph check`.
- **Invalid SPARQL**: Oxigraph returns a parse error; `product graph query` prints it and exits 1.
- **Depth ≥ 3 bundle warning**: warning on stderr, bundle produced without blocking.
- **Empty impact set**: `product impact` exits 0 and prints "No dependents found" — not an error.

### Boundaries

- Graph Intelligence operates on the in-memory graph derived from YAML front-matter. It does not analyse source code, git history, or any files outside the Product-managed document directories.
- SPARQL queries execute read-only against the in-memory graph. `product graph query` provides no mutation capability.
- Centrality and impact scores are computed, not declared. They reflect the graph topology at invocation time and may change as the graph evolves.

## Out of scope

- Persistent graph storage (the graph is always rebuilt from front-matter per ADR-003)
- External SPARQL endpoints (Oxigraph is embedded, not a service)
- Graph visualisation (terminal output only; no web UI or graph rendering)
- PageRank or other centrality measures (betweenness centrality is the correct model for structural bridging)
- LLM-assisted graph analysis (graph algorithms are deterministic; LLM analysis is the domain of gap analysis and drift detection)
