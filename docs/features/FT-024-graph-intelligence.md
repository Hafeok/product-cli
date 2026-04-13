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
- TC-157
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
- TC-009
- TC-010
domains: []
domains-acknowledged: {}
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
