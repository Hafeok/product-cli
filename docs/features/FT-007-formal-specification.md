---
id: FT-007
title: Formal Specification
phase: 1
status: complete
depends-on: []
adrs:
- ADR-004
tests:
- TC-011
- TC-012
domains: []
domains-acknowledged: {}
---

⟦Σ:Types⟧{
  Graph≜⟨nodes:Node+, edges:Edge*⟩
  CentralityScore≜Float
}

⟦Γ:Invariants⟧{
  ∀g:Graph, ∀n∈g.nodes: betweenness(g,n) ≥ 0.0 ∧ betweenness(g,n) ≤ 1.0
}

⟦Ε⟧⟨δ≜0.99;φ≜100;τ≜◊⁺⟩
```

**Benchmark example:**
```markdown
---
id: TC-030
title: LLM Context Quality — Raft Leader Election
type: benchmark
status: unimplemented
validates:
  features: [FT-001]
  adrs: [ADR-006, ADR-012]
phase: 3
benchmark:
  task: benchmarks/tasks/task-001-raft-leader-election
  rubric: benchmarks/tasks/task-001-raft-leader-election/rubric.md
  conditions: [none, naive, product]
  runs-per-condition: 5
  pass-threshold:
    product: 0.80
    delta-vs-naive: 0.15
---