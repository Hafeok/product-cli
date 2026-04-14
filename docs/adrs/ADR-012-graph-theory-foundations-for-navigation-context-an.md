---
id: ADR-012
title: Graph Theory Foundations for Navigation, Context, and Impact Analysis
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:babc958120bef73567be0426c7048247619d7402482cbb8ad075ae90a8225e33
---

**Status:** Accepted

**Context:** The current graph model supports only fixed 1-hop traversals: a feature's direct ADRs, a feature's direct tests, an ADR's direct features. This is sufficient for simple lookups but fails for four real problems:

1. **Implementation ordering** — `product feature next` uses phase labels to determine what to implement next. Phase labels are human-assigned approximations of dependency order. A feature in phase 2 may depend on an incomplete feature in phase 1, but phase ordering cannot express or detect this. The correct implementation order is determined by the *dependency structure* of the feature graph, not by human-assigned integers.

2. **Context depth** — context bundles are assembled at exactly 1 hop from the seed feature. An agent implementing a feature that shares foundational ADRs with adjacent features has no way to discover that adjacency without querying each feature individually. Transitive context — the ADRs and tests of features this feature depends on — is often relevant but is currently invisible.

3. **Decision importance** — all ADRs in a context bundle are presented as equal. ADR-001 (Rust) is structurally foundational — it is linked to every feature. ADR-007 (checklist generation) is peripheral. An agent or engineer has no signal about which decisions to read first. This signal is latent in the graph structure but not surfaced.

4. **Change impact** — superseding or modifying an ADR has downstream consequences: features that must be re-evaluated, tests that may be invalidated, implementation work that may need to be revisited. Today the developer discovers these consequences by reading every linked file. A graph-reachability traversal can compute the full impact set in one operation.

**Decision:** Extend the graph model with four graph-theoretic capabilities:

1. **Topological sort** on a `depends-on` DAG of feature nodes — used for `product feature next` and dependency validation
2. **BFS to configurable depth** — used for `product context --depth N` to surface transitive context
3. **Betweenness centrality** on ADR nodes — used for `product graph central` to rank architectural decisions by structural importance
4. **Reverse-graph reachability** — used for `product impact` to compute the full affected set of any change

---

### Capability 1: Topological Sort, Feature Dependencies, and Phase Gates

**New edge type:** `depends-on` between Feature nodes. Declared in feature front-matter:

```yaml
---
id: FT-003
title: RDF Projection
depends-on: [FT-001, FT-002]
---
```

This edge means FT-003 cannot be correctly implemented until FT-001 and FT-002 are complete.

**Graph construction:** Feature nodes plus `depends-on` edges form a directed acyclic graph (DAG). Product validates this DAG on every invocation. A cycle (FT-001 depends-on FT-003 depends-on FT-001) is a hard error — exit code 1. Cycles represent contradictory dependency claims and cannot be resolved automatically.

**Topological sort:** Kahn's algorithm over the feature DAG produces a partial order of valid implementation sequences. `product feature next` applies a two-level gate to select the next feature:

```
for each feature F in topological order:
    if F.status == complete:                              skip
    if any depends_on predecessor is not complete:        skip
    if F.phase > 1 AND NOT phase_gate_satisfied(F.phase - 1):  skip
    return F   ← next feature to implement
```

**Phase gate (`phase_gate_satisfied(N)`):**

A phase gate is satisfied when all test criteria of type `exit-criteria` linked to features in phase N have `status: passing`. Not all features in the phase need to be complete — only the exit criteria must pass. This reflects the spec's definition of phase completion: a phase is done when its measurable exit conditions are met, not when every feature in it is perfect.

```rust
fn phase_gate_satisfied(phase: u32, graph: &Graph) -> bool {
    graph.features_in_phase(phase)
        .flat_map(|f| graph.tests_for_feature(f))
        .filter(|tc| tc.tc_type == TcType::ExitCriteria)
        .all(|tc| tc.status == TcStatus::Passing)
}
```

If no exit-criteria TCs exist for a phase, the gate is considered satisfied — a phase with no defined exit criteria is always open. This ensures backward compatibility during migration when TCs haven't been written yet.

**What `product feature next` reports when a phase gate blocks:**

```
product feature next

  Next candidate: FT-009 — Rate Limiting  [phase 2, planned]
  ✗ Phase 2 locked — Phase 1 exit criteria not all passing:

    TC-001  Binary compiles               [passing  ✓]
    TC-004  Two-node cluster forms        [passing  ✓]
    TC-007  Workload survives restart     [failing  ✗]
    TC-012  Volume allocation end-to-end  [unimplemented]

  Fix TC-007 and TC-012 to unlock Phase 2.
  To skip the gate:  product feature next --ignore-phase-gate
  To work on FT-009 directly:  product preflight FT-009
```

The `--ignore-phase-gate` flag bypasses the phase gate for the current invocation only. It does not suppress the warning. Explicit feature invocations (`product preflight FT-009`, `product context FT-009`) are always available regardless of phase gate state — the gate only applies to the automated `next` selection.

**Topological order vs. phase labels:** Phase labels carry human intent about grouping and milestones. Topological order carries structural truth about explicit dependency. The phase gate adds a third signal: phase completion readiness. All three are used together in `product feature next`. When they disagree (a phase-1 feature depends-on a phase-2 feature), `product graph check` reports W005.

**New command:** `product feature deps FT-003` — prints the full transitive dependency tree for a feature.

**`product status` with phase gate display:**

```
product status

Phase 1 — Cluster Foundation  [OPEN — exit criteria: 2/4 passing]
  FT-001  Cluster Foundation     complete
  FT-002  mTLS Node Comms        complete
  FT-003  Raft Consensus         in-progress
  FT-004  Block Storage          planned

Phase 2 — Products and IAM  [LOCKED — Phase 1 exit criteria: TC-007, TC-012 not passing]
  FT-005  Product Resource       planned
  FT-006  OIDC Provider          planned

Phase 3 — RDF and Event Store  [LOCKED — Phase 2 not yet open]
  FT-007  RDF Store              planned
```

`product status --phase 1` shows the full exit criteria detail for a single phase including which TCs are passing, failing, and unimplemented.

---

### Capability 2: BFS Context Assembly

**Current behaviour:** `product context FT-001` performs exactly 1-hop traversal:
```
FT-001 → {ADR-001, ADR-002} → (stop)
FT-001 → {TC-001, TC-002}   → (stop)
```

**New behaviour:** `product context FT-001 --depth N` performs BFS to depth N from the seed node, following all edge types in the traversal direction. Default depth is 1 (preserves current behaviour).

**Depth semantics:**

```
depth 1 (default):
  FT-001 → direct ADRs, direct tests

depth 2:
  FT-001 → direct ADRs → other features those ADRs apply to
  FT-001 → depends-on features → their ADRs and tests
  FT-001 → direct tests → (no outbound edges from tests)

depth 3:
  depth-2 nodes → their ADRs, tests, and dependencies
```

**Deduplication:** A node that appears multiple times in a BFS traversal (reachable via multiple paths) is included once in the bundle, at its first-encountered position. The bundle header `⟦Ω:Bundle⟧` lists all included artifact IDs so the agent sees the full manifest before reading content.

**Practical limit:** Depth ≥ 3 on a well-connected graph risks pulling in most of the repository. `product context --depth 3` emits a warning to stderr if the resulting bundle exceeds 50 nodes: "Bundle contains N artifacts at depth 3. Consider narrowing scope." The bundle is still produced — the warning does not block output.

**New flag on context command:**
```
product context FT-001 --depth 2     # transitive context
product context FT-001 --depth 1     # direct only (default)
```

---

### Capability 3: Betweenness Centrality

**Definition:** The betweenness centrality of a node v is the fraction of shortest paths between all pairs of nodes in the graph that pass through v. A node with high betweenness is a structural bridge — many other nodes depend on it to connect to each other.

**Application to ADRs:** ADRs that are linked to many features, and whose features are otherwise loosely connected, have high betweenness. These are the foundational decisions an engineer or agent must understand before working on any feature. ADRs that apply to a single isolated feature have low betweenness regardless of how important they feel to the author.

**Algorithm:** Brandes' algorithm. O(V·E) time complexity. On a repository with 200 nodes and 800 edges this completes in < 50ms.

**New command:**
```
product graph central                # top-10 ADRs by betweenness centrality
product graph central --top 5        # configurable N
product graph central --all          # full ranked list
```

**Output format:**
```
Rank  ID       Centrality  Title
1     ADR-001  0.847       Rust as Implementation Language
2     ADR-002  0.731       openraft for Cluster Consensus
3     ADR-006  0.612       Oxigraph for RDF Projection
4     ADR-003  0.445       Event Log Schema
5     ADR-009  0.201       CI Exit Codes
```

**Integration with context bundles:** When `--depth 1` (default), ADRs in the bundle are ordered by betweenness centrality descending, not by ID ascending. An agent reading the bundle top-to-bottom encounters the most structurally important decisions first. ID-ascending order is available via `--order id`.

**`product graph stats` output** is extended with:
```
ADR centrality: mean=0.41, max=0.847 (ADR-001), min=0.003 (ADR-007)
Structural hubs (centrality > 0.5): ADR-001, ADR-002, ADR-006
```

---

### Capability 4: Reverse-Graph Reachability (Impact Analysis)

**Reverse graph:** For every directed edge A → B in the knowledge graph, the reverse graph contains edge B → A. BFS on the reverse graph from any node returns all nodes that have a path *to* that node in the forward graph — i.e., everything that depends on it.

**`product impact` command:**
```
product impact ADR-002               # what is affected if ADR-002 changes
product impact TC-003                # what depends on this test criterion
product impact FT-001                # what depends on this feature completing
```

**Impact set composition for an ADR:**

Starting from ADR-002 in the reverse graph:
- Features that `implementedBy` ADR-002 — must be re-evaluated
- Test criteria that `validates` ADR-002 — may be invalidated
- Features that `depends-on` features linked to ADR-002 — transitively affected

**Output:**
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

The summary line highlights passing tests that may be invalidated — these are the highest-urgency items when superseding a decision.

**Integration with ADR supersession:** When `product adr status ADR-002 superseded --by ADR-013` is run, Product automatically runs impact analysis and prints the impact summary before completing the status change. The developer sees the full blast radius before committing.

---

### Graph Model Update

The full edge type set after this ADR:

| Edge | From | To | Direction | Description |
|---|---|---|---|---|
| `implementedBy` | Feature | ADR | forward | Feature is governed by this decision |
| `validatedBy` | Feature | TestCriterion | forward | Feature is verified by this test |
| `testedBy` | ADR | TestCriterion | forward | Decision is verified by this test |
| `supersedes` | ADR | ADR | forward | This decision replaces another |
| `depends-on` | Feature | Feature | forward | Implementation dependency |

The reverse of every edge is implicit and traversed by impact analysis.

---

**Rationale:**
- Topological sort is the only correct solution to implementation ordering in a system with explicit dependencies. Phase labels cannot express partial order — two features in the same phase may have a dependency between them that phase numbers cannot represent
- BFS depth generalises context assembly without changing the default behaviour — existing workflows are unaffected unless `--depth N` is explicitly passed
- Betweenness centrality requires no human curation — the structural importance ranking falls out of the graph that already exists. It does not add any new maintenance burden
- Reverse-graph reachability is O(V+E) and trivially derived from the forward graph already in memory. The implementation cost is near zero; the operational value (knowing the blast radius of a change before making it) is high
- All four algorithms operate on graphs of the scale Product manages (< 500 nodes) in well under 100ms. There is no performance argument against any of them

**Rejected alternatives:**
- **PageRank for ADR importance** — PageRank models random-walk importance, which assumes edges represent influence or endorsement. Our edges are structural dependencies, not endorsements. Betweenness centrality correctly models structural bridging, which is the property we want.
- **Manual importance tagging on ADRs** — `importance: foundational | standard | peripheral` in front-matter. Requires human judgment and drifts over time as the graph evolves. Centrality is computed, not declared — it cannot drift.
- **Depth-limited context as default** — making depth-2 the default for `product context`. Rejected because depth-2 bundles are significantly larger and the use case (transitive context for an agent implementing a complex feature) is not the common case. Default depth-1 preserves current behaviour; opt-in depth-2 covers the complex case.
- **Full graph dump with relevance scoring** — send the entire graph to an LLM and let it select relevant nodes. Rejected because it defeats the purpose of Product: the whole point is to assemble targeted context cheaply and deterministically, not to add another LLM call to the pipeline.