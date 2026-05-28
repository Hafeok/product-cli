---
id: FT-016
title: Graph Model
phase: 1
status: complete
depends-on: []
adrs:
- ADR-003
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
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
---

Product builds an in-memory directed graph from front-matter on every invocation. The graph is also exportable as RDF Turtle via `product graph rebuild`.

### Edge Types

| Edge | From | To | Description |
|---|---|---|---|
| `implementedBy` | Feature | ADR | Feature is governed by this decision |
| `validatedBy` | Feature | TestCriterion | Feature is verified by this test |
| `testedBy` | ADR | TestCriterion | Decision is verified by this test |
| `supersedes` | ADR | ADR | This decision replaces another |
| `depends-on` | Feature | Feature | Implementation dependency — must complete before |

The reverse of every edge is implicit. Impact analysis (`product impact`) traverses the reverse graph to compute reachability.

### Graph Algorithms

| Algorithm | Applied to | Command | Purpose |
|---|---|---|---|
| Topological sort (Kahn's) | Feature `depends-on` DAG | `product feature next` | Correct implementation ordering |
| BFS to depth N | All edges | `product context --depth N` | Transitive context assembly |
| Betweenness centrality (Brandes') | ADR nodes | `product graph central` | Structural importance ranking |
| Reverse-graph BFS | All edges reversed | `product impact` | Change impact analysis |

### RDF Export

Product exports the knowledge graph as RDF Turtle. The ontology prefix is `pm:` (product-meta).

```turtle
@prefix pm: <https://product-meta/ontology#> .
@prefix ft: <https://product-meta/feature/> .
@prefix adr: <https://product-meta/adr/> .
@prefix tc: <https://product-meta/test/> .

ft:FT-001 a pm:Feature ;
    pm:title "Cluster Foundation" ;
    pm:phase 1 ;
    pm:status pm:InProgress ;
    pm:implementedBy adr:ADR-001 ;
    pm:implementedBy adr:ADR-002 ;
    pm:validatedBy tc:TC-001 ;
    pm:validatedBy tc:TC-002 .

ft:FT-003 a pm:Feature ;
    pm:dependsOn ft:FT-001 ;
    pm:dependsOn ft:FT-002 .

adr:ADR-002 a pm:ArchitecturalDecision ;
    pm:title "openraft for Cluster Consensus" ;
    pm:status pm:Accepted ;
    pm:betweennessCentrality 0.731 ;
    pm:appliesTo ft:FT-001 ;
    pm:testedBy tc:TC-002 .

tc:TC-002 a pm:TestCriterion ;
    pm:title "Raft Leader Election" ;
    pm:type pm:Scenario ;
    pm:status pm:Unimplemented ;
    pm:validates ft:FT-001 ;
    pm:validates adr:ADR-002 .
```

Betweenness centrality scores are written into the TTL export on `graph rebuild` so external SPARQL tools can query on them.

---

---

## Description

Product builds an in-memory directed knowledge graph from YAML front-matter on every invocation (ADR-003). The graph is never persisted — it is always correct by construction. Four graph-theoretic algorithms are layered on top (ADR-012): topological sort (Kahn's) for feature ordering, BFS to configurable depth for context assembly, Brandes' betweenness centrality for ADR importance ranking, and reverse-graph BFS for impact analysis. The graph is also exportable as RDF Turtle via `product graph rebuild`.

## Functional Specification

### Inputs

- All artifact files scanned from paths configured in `product.toml` (`docs/features/`, `docs/adrs/`, `docs/tests/`)
- Front-matter fields that declare edges: `adrs`, `tests` (Feature→ADR, Feature→TC), `validates.features`, `validates.adrs` (TC→Feature, TC→ADR), `supersedes` / `superseded-by` (ADR→ADR), `depends-on` (Feature→Feature)
- Command-specific arguments: `--depth N` for BFS, `--top N` for centrality, artifact ID for impact analysis

### Outputs

- In-memory `KnowledgeGraph` struct (nodes: `HashMap<id, Feature|Adr|TestCriterion|Dependency|Pattern>`, edges: `Vec<Edge>`, forward/reverse adjacency lists)
- `product graph rebuild`: RDF Turtle file at `index.ttl` with betweenness centrality scores written into the TTL export
- `product graph central`: ranked ADR list by betweenness centrality score
- `product impact ADR-XXX`: impact set report — direct and transitive dependents, summary of passing tests at risk
- `product context FT-XXX --depth N`: context bundle assembled via BFS to depth N
- `product feature next`: next feature in topological order respecting phase gates
- SPARQL query results via `product graph query` (embedded Oxigraph, ADR-008)

### State

Stateless between invocations. The graph is rebuilt from files on every command (ADR-003). `index.ttl` is an export snapshot — Product never reads it as input. If `index.ttl` is stale, `product graph rebuild` regenerates it.

### Behaviour

1. Scan artifact directories, parse YAML front-matter for all files; collect parse errors.
2. Build `KnowledgeGraph`: insert nodes, materialise directed edges from declared IDs, build forward and reverse adjacency lists.
3. Topological sort (Kahn's): validates the `depends-on` DAG; a cycle is E003 (hard error). Used by `product feature next` to determine correct implementation ordering and by `product checklist generate` for feature ordering.
4. BFS (depth N): traverses forward and reverse edges from a seed node up to depth N, deduplicating by first-encounter. Default depth is 1 (direct links only); `--depth 2` includes transitive context.
5. Betweenness centrality (Brandes' algorithm, O(V·E)): computed over ADR and Feature nodes; scores are included in `index.ttl` exports and used to order ADRs within context bundles.
6. Reverse-graph BFS: starting from any node, traverses reversed edges to compute the full reachable impact set. Used by `product impact`.
7. SPARQL queries: the in-memory graph is loaded into an embedded Oxigraph store on `graph query` invocations (ADR-008); SPARQL 1.1 SELECT/CONSTRUCT/ASK/DESCRIBE are supported.

### Invariants

- The graph is always rebuilt from files; it cannot be stale with respect to the current file state (ADR-003).
- A `depends-on` cycle is a hard error E003 — cycles cannot be resolved automatically (ADR-012).
- A `supersedes` cycle is a hard error E004.
- Phase label disagreement with topological order produces W005 (warning, not error).
- Depth ≥ 3 with > 50 nodes in the bundle emits a warning to stderr; the bundle is still produced (ADR-012).
- `index.ttl` is never read as a graph source by the CLI; it is write-only export (ADR-003).

### Error handling

- E001: malformed front-matter in any artifact file; reported with file path and line.
- E002: referenced artifact ID does not exist in the graph (broken link).
- E003: cycle in `depends-on` DAG.
- E004: cycle in ADR `supersedes` chain.
- E008: `schema-version` in `product.toml` exceeds the binary's supported version.
- SPARQL query errors from Oxigraph are surfaced with the query text and the error message.

### Boundaries

- The graph covers only artifacts in the configured scan paths; files outside those paths are not nodes.
- Betweenness centrality excludes Pattern nodes by default for backward-compatible output of `product graph central` (opt-in via `--include-patterns`).
- SPARQL is available via `product graph query` only; other commands use the typed in-memory model, not SPARQL.

## Out of scope

- Persistent graph storage (never implemented; ADR-003 explicitly rejects this).
- Graph mutation (the graph is read-only; mutations happen via artifact file writes through the command adapters).
- Full RDF reasoning or OWL inference (only SPARQL 1.1 query over asserted triples).
