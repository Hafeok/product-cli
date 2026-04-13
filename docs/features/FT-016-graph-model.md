---
id: FT-016
title: Graph Model
phase: 1
status: in-progress
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
domains: []
domains-acknowledged: {}
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